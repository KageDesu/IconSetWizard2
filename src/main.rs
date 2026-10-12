#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod document;
mod editor;
mod i18n;
mod library;
mod storage;
mod ui;
use document::{CELL, COLS, Document, Layer, MAX_ROWS, fit_cell, read_png};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};
use std::path::{Path, PathBuf};

struct Wizard {
    language: i18n::Language,
    doc: Document,
    source: Option<PathBuf>,
    png_source: Option<PathBuf>,
    document_open: bool,
    overwrite_png: bool,
    library_open: bool,
    about_open: bool,
    library_folder: PathBuf,
    packages: Vec<library::Package>,
    library_category: usize,
    library_textures: Vec<TextureHandle>,
    library_errors: Vec<String>,
    last_preview: Option<Layer>,
    selected: usize,
    layer: usize,
    draft: Layer,
    undo: Vec<Document>,
    redo: Vec<Document>,
    clipboard: Option<Vec<Layer>>,
    group_clipboard: Option<Vec<editor::TileCopy>>,
    selection: std::collections::BTreeSet<usize>,
    selection_anchor: usize,
    selection_drag: Option<usize>,
    selection_drag_base: std::collections::BTreeSet<usize>,
    preferences: storage::Preferences,
    library_query: String,
    library_favorites_only: bool,
    library_search_all: bool,
    library_texture_cache: std::collections::BTreeMap<(usize, usize), TextureHandle>,
    color_open: bool,
    color_from: [u8; 3],
    color_to: [u8; 3],
    color_tolerance: u8,
    color_all_layers: bool,
    storage_root: Option<PathBuf>,
    autosave_store: Option<storage::RecoveryStore>,
    autosave_enabled: bool,
    autosave_needed: bool,
    autosave_last: std::time::Instant,
    autosave_draft: Option<Layer>,
    recovery_candidates: Vec<PathBuf>,
    recovery_open: bool,
    restored_from: Option<PathBuf>,
    sheet: Option<TextureHandle>,
    preview: Option<TextureHandle>,
    refresh: bool,
    thumbnails: Vec<TextureHandle>,
    screenshot_frame: u32,
    zoom: f32,
    grid: bool,
    dirty: bool,
    status: String,
    confirm_close: bool,
    confirm_png_save: bool,
    allow_close: bool,
}

fn blank() -> Document {
    Document {
        version: 2,
        tiles: vec![vec![]; COLS * 8],
    }
}

fn draft_for(doc: &Document, selected: usize, layer: usize) -> Layer {
    doc.tiles[selected].get(layer).cloned().unwrap_or_else(|| {
        Layer::new(
            crate::i18n::tr("Слой").into(),
            image::RgbaImage::new(CELL, CELL),
        )
    })
}

impl Wizard {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let saved = if std::env::var_os("ISW_SCREENSHOT_PATH").is_none() {
            storage::load_preferences(&storage::root())
                .ok()
                .and_then(|p| p.language)
        } else {
            None
        };
        let language = std::env::var("ISW_LANGUAGE")
            .ok()
            .map(|locale| i18n::Language::from_locale(&locale))
            .unwrap_or_else(|| i18n::Language::startup(saved));
        i18n::set_language(language);
        ui::configure(&cc.egui_ctx);
        let argument = std::env::args_os().nth(1).map(PathBuf::from);
        let nearby = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("IconSet.png")));
        let path = argument
            .or_else(|| nearby.filter(|p| p.exists()))
            .or_else(|| {
                Path::new("IconSet.png")
                    .exists()
                    .then(|| PathBuf::from("IconSet.png"))
            });
        let (doc, source, status) = match path {
            Some(path) => match Document::from_png(&path) {
                Ok(doc) => (
                    doc,
                    Some(path),
                    crate::i18n::tr("Iconset открыт. Выберите ячейку для редактирования.").into(),
                ),
                Err(error) => (blank(), None, error),
            },
            None => (
                blank(),
                None,
                crate::i18n::tr("Откройте PNG или перетащите проект на окно.").into(),
            ),
        };
        let mut wizard = Self::from_document(doc, source, status);
        if std::env::var_os("ISW_SCREENSHOT_PATH").is_none() {
            wizard.initialize_storage(storage::root());
        }
        if std::env::var_os("ISW_SCREENSHOT_PATH").is_some() {
            wizard.about_open = std::env::var_os("ISW_SCREENSHOT_ABOUT").is_some();
            cc.egui_ctx
                .all_styles_mut(|style| style.animation_time = 0.0);
            wizard.selected = 1.min(wizard.doc.tiles.len() - 1);
            wizard.zoom = 1.0;
            wizard.sync();
            if std::env::var_os("ISW_SCREENSHOT_LIBRARY").is_some() {
                wizard.load_library();
                wizard.library_open = true;
                match std::env::var("ISW_SCREENSHOT_LIBRARY").as_deref() {
                    Ok("search") => wizard.library_query = "Battlers".into(),
                    Ok("favorites") => {
                        let category = wizard
                            .packages
                            .iter()
                            .position(|p| p.name == "MV Battlers")
                            .unwrap_or(0);
                        for index in [0, 1, 2] {
                            wizard.toggle_favorite(category, index);
                        }
                        wizard.library_favorites_only = true;
                    }
                    _ => {}
                }
            }
            if std::env::var_os("ISW_SCREENSHOT_EDITOR").is_some() {
                wizard.selection = std::collections::BTreeSet::from([1, 2, 17, 18]);
                wizard.color_open = true;
                if let Some(pixel) = wizard
                    .draft
                    .pixels
                    .rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .find(|p| p[3] > 0)
                {
                    wizard.color_from = [pixel[0], pixel[1], pixel[2]];
                }
            }
            if let Some(path) = std::env::var_os("ISW_SCREENSHOT_RECOVERY_PATH") {
                wizard.recovery_candidates = vec![PathBuf::from(path)];
                wizard.recovery_open = true;
            }
        }
        wizard
    }

    fn from_document(doc: Document, source: Option<PathBuf>, status: String) -> Self {
        let draft = draft_for(&doc, 0, 0);
        let png_source = source
            .clone()
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")));
        let document_open = source.is_some();
        let nearby = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("packages")));
        let library_folder = nearby
            .filter(|p| p.is_dir())
            .unwrap_or_else(|| PathBuf::from("packages"));
        Self {
            language: i18n::current(),
            doc,
            source,
            png_source,
            document_open,
            overwrite_png: false,
            library_open: false,
            about_open: false,
            library_folder,
            packages: vec![],
            library_category: 0,
            library_textures: vec![],
            library_errors: vec![],
            last_preview: None,
            selected: 0,
            layer: 0,
            draft,
            undo: vec![],
            redo: vec![],
            clipboard: None,
            group_clipboard: None,
            selection: std::collections::BTreeSet::from([0]),
            selection_anchor: 0,
            selection_drag: None,
            selection_drag_base: Default::default(),
            preferences: Default::default(),
            library_query: String::new(),
            library_favorites_only: false,
            library_search_all: true,
            library_texture_cache: Default::default(),
            color_open: false,
            color_from: [255, 0, 0],
            color_to: [0, 128, 255],
            color_tolerance: 0,
            color_all_layers: false,
            storage_root: None,
            autosave_store: None,
            autosave_enabled: true,
            autosave_needed: false,
            autosave_last: std::time::Instant::now(),
            autosave_draft: None,
            recovery_candidates: vec![],
            recovery_open: false,
            restored_from: None,
            sheet: None,
            preview: None,
            refresh: true,
            thumbnails: vec![],
            screenshot_frame: 0,
            zoom: 1.5,
            grid: true,
            dirty: false,
            status,
            confirm_close: false,
            confirm_png_save: false,
            allow_close: false,
        }
    }

    fn sync(&mut self) {
        self.selected = self.selected.min(self.doc.tiles.len() - 1);
        self.selection.retain(|i| *i < self.doc.tiles.len());
        self.layer = self
            .layer
            .min(self.doc.tiles[self.selected].len().saturating_sub(1));
        self.draft = draft_for(&self.doc, self.selected, self.layer);
        self.refresh = true;
    }

    fn change(&mut self, change: impl FnOnce(&mut Document)) -> bool {
        let before = self.doc.clone();
        change(&mut self.doc);
        if let Err(error) = self.doc.validate() {
            self.doc = before;
            self.status = error;
            self.sync();
            return false;
        }
        self.undo.push(before);
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
        self.autosave_needed = true;
        self.sync();
        self.document_open = true;
        true
    }

    fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.doc, previous));
            self.dirty = true;
            self.autosave_needed = true;
            self.sync();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.doc, next));
            self.dirty = true;
            self.autosave_needed = true;
            self.sync();
        }
    }

    fn may_replace(&self) -> bool {
        !(self.dirty || self.has_pending_draft())
            || rfd::MessageDialog::new()
                .set_title(crate::i18n::tr("Несохранённый проект"))
                .set_description(crate::i18n::tr(
                    "Отменить изменения и открыть другой документ?",
                ))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                == rfd::MessageDialogResult::Yes
    }

    fn open(&mut self, path: PathBuf) {
        let result = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("isw2"))
        {
            std::fs::metadata(&path)
                .map_err(|e| e.to_string())
                .and_then(|meta| {
                    if meta.len() > 128 * 1024 * 1024 {
                        return Err(crate::i18n::tr("Файл проекта слишком большой.").into());
                    }
                    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                    let doc: Document =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    doc.validate()?;
                    Ok(doc)
                })
        } else {
            Document::from_png(&path)
        };
        match result {
            Ok(doc) => {
                if !self.may_replace() {
                    return;
                }
                self.clear_recovery();
                self.doc = doc;
                self.png_source = Some(path.clone())
                    .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")));
                self.overwrite_png = false;
                self.document_open = true;
                self.source = Some(path);
                self.selected = 0;
                self.selection = std::collections::BTreeSet::from([0]);
                self.selection_anchor = 0;
                self.layer = 0;
                self.undo.clear();
                self.redo.clear();
                self.dirty = false;
                self.sync();
                self.status = crate::i18n::tr("Документ открыт.").into();
            }
            Err(error) => {
                self.status = crate::tf!(
                    "Ошибка открытия: {error}",
                    "Open error: {error}",
                    "打开错误：{error}"
                )
            }
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(crate::i18n::tr("Iconset / проект"), &["png", "isw2"])
            .pick_file()
        {
            self.open(path);
        }
    }

    fn save_project(&mut self) -> bool {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(crate::i18n::tr("Проект Wizard"), &["isw2"])
            .set_file_name("IconSet.isw2")
            .save_file()
        else {
            return false;
        };
        let path = path.with_extension("isw2");
        self.commit_draft();
        self.doc.version = 2;
        let result = serde_json::to_vec(&self.doc)
            .map_err(|e| e.to_string())
            .and_then(|bytes| atomic_write(&path, &bytes));
        match result {
            Ok(()) => {
                self.clear_recovery();
                self.dirty = false;
                self.source = Some(path.clone());
                self.status = crate::tf!(
                    "Проект сохранён: {}",
                    "Project saved: {}",
                    "项目已保存：{}",
                    path.display()
                );
                true
            }
            Err(error) => {
                self.status = crate::tf!(
                    "Ошибка сохранения: {error}",
                    "Save error: {error}",
                    "保存错误：{error}"
                );
                false
            }
        }
    }

    fn export(&mut self) {
        let path = if self.overwrite_png {
            self.png_source.clone()
        } else {
            rfd::FileDialog::new()
                .add_filter("PNG", &["png"])
                .set_file_name("IconSet-export.png")
                .save_file()
        };
        let Some(path) = path else {
            return;
        };
        let path = path.with_extension("png");
        self.write_png(&path);
    }

    fn save_current_png(&mut self) {
        if let Some(path) = self.png_source.clone() {
            self.write_png(&path);
        }
    }

    fn write_png(&mut self, path: &Path) {
        self.commit_draft();
        let mut bytes = std::io::Cursor::new(Vec::new());
        let result = self
            .doc
            .render()
            .write_to(&mut bytes, image::ImageFormat::Png)
            .map_err(|e| e.to_string())
            .and_then(|()| atomic_write(path, &bytes.into_inner()));
        self.status = match result {
            Ok(()) => crate::tf!(
                "PNG экспортирован: {}. Сохраните проект, чтобы сохранить слои.",
                "PNG exported: {}. Save the project to preserve layers.",
                "PNG已导出：{}。请保存项目以保留图层。",
                path.display()
            ),
            Err(error) => crate::tf!(
                "Ошибка экспорта: {error}",
                "Export error: {error}",
                "导出错误：{error}"
            ),
        };
    }

    fn copy_icon(&mut self, ctx: &egui::Context) {
        self.commit_draft();
        let tile = self.doc.tiles[self.selected].clone();
        let group = self.copy_group();
        // winit sends Paste only when the OS clipboard contains non-empty text.
        // Publish the actual layer data so native Ctrl+V also works between windows.
        if group.len() > 1 {
            if let Ok(json) = serde_json::to_string(&group) {
                ctx.copy_text(format!("ICONSET_WIZARD_GROUP_2:{json}"));
            }
        } else if let Ok(json) = serde_json::to_string(&tile) {
            ctx.copy_text(format!("ICONSET_WIZARD_2:{json}"));
        }
        self.group_clipboard = Some(group);
        self.clipboard = Some(tile);
        self.status = crate::i18n::tr("Иконка скопирована вместе со слоями.").into();
    }

    fn paste_icon(&mut self, text: Option<&str>) {
        if let Some(json) = text.and_then(|t| t.strip_prefix("ICONSET_WIZARD_GROUP_2:")) {
            if json.len() <= 128 * 1024 * 1024
                && let Ok(group) = serde_json::from_str::<Vec<editor::TileCopy>>(json)
            {
                self.paste_group(group);
            }
            return;
        }
        if text.is_none_or(|t| !t.starts_with("ICONSET_WIZARD_2:"))
            && let Some(group) = self.group_clipboard.clone()
            && group.len() > 1
        {
            self.paste_group(group);
            return;
        }
        let from_system = text.and_then(|text| {
            let json = text.strip_prefix("ICONSET_WIZARD_2:")?;
            if json.len() > 8 * 1024 * 1024 {
                return None;
            }
            let tile: Vec<Layer> = serde_json::from_str(json).ok()?;
            let mut doc = Document {
                version: 2,
                tiles: vec![vec![]; COLS],
            };
            doc.tiles[0] = tile.clone();
            doc.validate().ok()?;
            Some(tile)
        });
        if let Some(tile) = from_system.or_else(|| self.clipboard.clone()) {
            let cells = self.selected_cells();
            if self.change(|doc| {
                for selected in cells {
                    doc.tiles[selected] = tile.clone();
                }
            }) {
                self.status = crate::i18n::tr("Иконка вставлена вместе со слоями.").into();
            }
        }
    }

    fn import_layer(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(crate::i18n::tr("Слой PNG"), &["png"])
            .pick_file()
        else {
            return;
        };
        self.import_path(&path, false);
    }

    fn replace_layer(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(crate::i18n::tr("Слой PNG"), &["png"])
            .pick_file()
        {
            self.import_path(&path, true);
        }
    }

    fn import_path(&mut self, path: &Path, replace: bool) {
        match read_png(path) {
            Ok(image) => {
                let resized = image.width() != CELL || image.height() != CELL;
                let pixels = fit_cell(image);
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                if !self.insert_image(name, pixels, replace) {
                    return;
                }
                self.status = if resized {
                    crate::i18n::tr("PNG приведён к 32 × 32 без сглаживания.")
                } else if replace {
                    crate::i18n::tr("Изображение слоя заменено.")
                } else {
                    crate::i18n::tr("Слой добавлен.")
                }
                .into();
            }
            Err(error) => {
                self.status = crate::tf!(
                    "Ошибка импорта: {error}",
                    "Import error: {error}",
                    "导入错误：{error}"
                )
            }
        }
    }

    fn insert_image(&mut self, name: String, image: image::RgbaImage, replace: bool) -> bool {
        self.commit_draft();
        let (selected, index) = (self.selected, self.layer);
        let layer = Layer::new(name, image);
        let replace = replace && !self.doc.tiles[selected].is_empty();
        if !self.change(|doc| {
            if replace {
                doc.tiles[selected][index].pixels = layer.pixels;
            } else {
                doc.tiles[selected].push(layer);
            }
        }) {
            return false;
        }
        if !replace {
            self.layer = self.doc.tiles[selected].len() - 1;
        }
        self.sync();
        true
    }

    fn has_pending_draft(&self) -> bool {
        self.doc.tiles[self.selected]
            .get(self.layer)
            .is_some_and(|l| *l != self.draft)
    }

    fn commit_draft(&mut self) {
        if self.has_pending_draft() {
            self.apply_draft();
        }
    }

    fn drop_path(&mut self, path: PathBuf) {
        if self.document_open
            && path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("png"))
        {
            self.import_path(&path, false);
        } else {
            self.open(path);
        }
    }

    fn load_library(&mut self) {
        match library::load_folder(&self.library_folder) {
            Ok((packages, errors)) => {
                self.packages = packages;
                self.library_errors = errors;
                self.library_category = self
                    .library_category
                    .min(self.packages.len().saturating_sub(1));
                self.library_textures.clear();
                self.library_texture_cache.clear();
            }
            Err(error) => {
                self.packages.clear();
                self.library_textures.clear();
                self.library_texture_cache.clear();
                self.library_errors = vec![crate::tf!(
                    "Не удалось открыть папку: {error}",
                    "Could not open folder: {error}",
                    "无法打开文件夹：{error}"
                )];
            }
        }
    }

    fn textures(&mut self, ctx: &egui::Context) {
        let upload = |name: &str, image: image::RgbaImage| {
            ctx.load_texture(
                name,
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                ),
                egui::TextureOptions::NEAREST,
            )
        };
        if self.refresh {
            self.sheet = Some(upload("sheet", self.doc.render()));
            self.thumbnails = self.doc.tiles[self.selected]
                .iter()
                .enumerate()
                .map(|(index, layer)| {
                    let doc = Document {
                        version: 1,
                        tiles: vec![vec![layer.clone()]],
                    };
                    upload(&format!("layer-{index}"), doc.render_tile(0))
                })
                .collect();
        }
        if self.refresh || self.last_preview.as_ref() != Some(&self.draft) {
            let mut tile = self.doc.tiles[self.selected].clone();
            if let Some(layer) = tile.get_mut(self.layer) {
                *layer = self.draft.clone();
            }
            let preview = Document {
                version: 2,
                tiles: vec![tile],
            };
            self.preview = Some(upload("selected", preview.render_tile(0)));
            self.last_preview = Some(self.draft.clone());
        }
        self.refresh = false;
    }
}

fn checker(ui: &egui::Ui, rect: Rect, step: f32) {
    let clip = ui.clip_rect().intersect(rect);
    let min_x = ((clip.min.x - rect.min.x) / step).floor().max(0.0) as i32;
    let min_y = ((clip.min.y - rect.min.y) / step).floor().max(0.0) as i32;
    let max_x = ((clip.max.x - rect.min.x) / step).ceil() as i32;
    let max_y = ((clip.max.y - rect.min.y) / step).ceil() as i32;
    let painter = ui.painter().with_clip_rect(clip);
    for y in min_y..max_y {
        for x in min_x..max_x {
            let color = if (x + y) % 2 == 0 {
                Color32::from_rgb(38, 48, 61)
            } else {
                Color32::from_rgb(49, 60, 76)
            };
            painter.rect_filled(
                Rect::from_min_size(
                    rect.min + Vec2::new(x as f32 * step, y as f32 * step),
                    Vec2::splat(step),
                ),
                0.0,
                color,
            );
        }
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    // Same-directory temporary file: a failed write never truncates the existing document.
    let filename = path
        .file_name()
        .ok_or(crate::i18n::tr("Недопустимое имя файла"))?
        .to_string_lossy();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let temp = path.with_file_name(format!(".{filename}.{}.{nonce}.tmp", std::process::id()));
    let mut created = false;
    let result = (|| {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        created = true;
        file.write_all(bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        std::fs::rename(&temp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() && created {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

impl eframe::App for Wizard {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw_ui(ui);
    }
}

fn main() -> eframe::Result {
    let width = std::env::var("ISW_SCREENSHOT_WIDTH")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1280.0);
    let height = std::env::var("ISW_SCREENSHOT_HEIGHT")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(860.0);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_min_inner_size([1050.0, 720.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "IconSet Wizard 2",
        options,
        Box::new(|cc| Ok(Box::new(Wizard::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_png_exports_identically_and_history_restores_pixels() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("IconSet.png");
        let input = read_png(&path).unwrap();
        let doc = Document::from_png(&path).unwrap();
        assert_eq!(doc.render(), input);
        let mut wizard = Wizard::from_document(doc, Some(path), String::new());
        wizard.change(|doc| doc.tiles[0][0].x = 3);
        wizard.undo();
        assert_eq!(wizard.doc.render(), input);
        wizard.redo();
        assert_eq!(wizard.doc.tiles[0][0].x, 3);
        wizard.undo();
        wizard.change(|doc| doc.tiles.extend(vec![vec![]; COLS]));
        assert!(wizard.redo.is_empty());
        assert_eq!(wizard.doc.render().height(), input.height() + CELL);
    }

    #[test]
    fn language_menu_switches_immediately_and_persists_without_editing_document() {
        i18n::set_language(i18n::Language::English);
        let ctx = egui::Context::default();
        ui::configure(&ctx);
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("language-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let mut wizard = Wizard::from_document(blank(), None, String::new());
        wizard.storage_root = Some(folder.clone());
        let original = wizard.doc.clone();
        let draw = |wizard: &mut Wizard, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1050.0, 720.0))),
                    events,
                    ..Default::default()
                },
                |ui| wizard.draw_ui(ui),
            );
            output.textures_delta.clear();
            output
        };
        let position = |output: &egui::FullOutput, label: &str| {
            output
                .shapes
                .iter()
                .find_map(|shape| {
                    if let egui::Shape::Text(text) = &shape.shape {
                        (text.galley.text() == label).then_some(text.pos + Vec2::splat(5.0))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| panic!("Missing UI label: {label}"))
        };
        for language in [
            i18n::Language::Chinese,
            i18n::Language::Russian,
            i18n::Language::English,
        ] {
            let output = draw(&mut wizard, vec![]);
            let pos = position(&output, i18n::tr("Язык"));
            for pressed in [true, false] {
                draw(
                    &mut wizard,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                );
            }
            let output = draw(&mut wizard, vec![]);
            let pos = position(&output, language.name());
            for pressed in [true, false] {
                draw(
                    &mut wizard,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                );
            }
            assert_eq!(wizard.language, language);
            assert_eq!(
                storage::load_preferences(&folder).unwrap().language,
                Some(language)
            );
            let output = draw(&mut wizard, vec![]);
            position(&output, i18n::tr("Открыть"));
            assert!(wizard.doc.tiles == original.tiles);
            assert!(!wizard.dirty);
        }
        // Verify every translated Chinese glyph, including the language menu name.
        let catalog: std::collections::BTreeMap<String, [String; 2]> =
            serde_json::from_str(include_str!("locales.json")).unwrap();
        ctx.fonts_mut(|fonts| {
            let font = egui::FontId::proportional(13.0);
            for text in catalog.values().map(|v| v[1].as_str()).chain(["简体中文"]) {
                for ch in text
                    .chars()
                    .filter(|c| *c >= '\u{4e00}' && *c <= '\u{9fff}')
                {
                    assert!(fonts.has_glyph(&font, ch), "Missing Chinese glyph: {ch}");
                }
            }
        });
        wizard.preferences.language = Some(i18n::Language::Chinese);
        wizard.persist_preferences();
        let mut restarted = Wizard::from_document(blank(), None, String::new());
        restarted.initialize_storage(folder.clone());
        assert_eq!(restarted.language, i18n::Language::Chinese);
        i18n::set_language(i18n::Language::English);
        drop(restarted);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn interface_renders_at_minimum_and_default_size() {
        let ctx = egui::Context::default();
        ui::configure(&ctx);
        let doc =
            Document::from_png(&Path::new(env!("CARGO_MANIFEST_DIR")).join("IconSet.png")).unwrap();
        let mut wizard = Wizard::from_document(doc, None, String::new());
        wizard.selected = 1;
        wizard.sync();
        wizard.draft.turns = 3;
        for size in [Vec2::new(1050.0, 720.0), Vec2::new(1280.0, 860.0)] {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| wizard.draw_ui(ui));
            assert!(!output.shapes.is_empty());
            output.textures_delta.clear();
        }
    }

    #[test]
    fn interface_actions_preserve_pixels_and_support_undo() {
        let ctx = egui::Context::default();
        ui::configure(&ctx);
        let doc =
            Document::from_png(&Path::new(env!("CARGO_MANIFEST_DIR")).join("IconSet.png")).unwrap();
        let original = doc.tiles[1][0].pixels.clone();
        let mut wizard = Wizard::from_document(doc, None, String::new());
        wizard.selected = 1;
        wizard.sync();
        wizard.draft.x = 3;
        let size = Vec2::new(1280.0, 1800.0);
        let mut draw = |events: Vec<egui::Event>| {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                    events,
                    ..Default::default()
                },
                |ui| wizard.draw_ui(ui),
            )
        };
        let mut output = draw(vec![]);
        let button_pos = |output: &egui::FullOutput, label: &str| {
            output
                .shapes
                .iter()
                .find_map(|shape| {
                    if let egui::Shape::Text(text) = &shape.shape {
                        (text.galley.text() == label).then_some(text.pos + Vec2::splat(5.0))
                    } else {
                        None
                    }
                })
                .expect("Action button must be rendered")
        };
        let apply = button_pos(&output, crate::i18n::tr("Применить"));
        output.textures_delta.clear();
        for pressed in [true, false] {
            let mut output = draw(vec![
                egui::Event::PointerMoved(apply),
                egui::Event::PointerButton {
                    pos: apply,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ]);
            output.textures_delta.clear();
        }
        let mut output = draw(vec![]);
        let undo = button_pos(&output, crate::i18n::tr("Отменить"));
        output.textures_delta.clear();
        assert_eq!(wizard.doc.tiles[1][0].x, 3);
        assert_eq!(wizard.doc.tiles[1][0].pixels.rgba, original.rgba);
        for pressed in [true, false] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                    events: vec![
                        egui::Event::PointerMoved(undo),
                        egui::Event::PointerButton {
                            pos: undo,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                    ..Default::default()
                },
                |ui| wizard.draw_ui(ui),
            );
            output.textures_delta.clear();
        }
        assert_eq!(wizard.doc.tiles[1][0].x, 0);
        assert_eq!(wizard.doc.tiles[1][0].pixels.rgba, original.rgba);
    }

    #[test]
    fn atomic_write_replaces_existing_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/atomic-write-test.isw2");
        atomic_write(&path, b"old").unwrap();
        atomic_write(&path, b"new").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn drop_opens_first_atlas_then_imports_png_and_replacement_preserves_properties() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut wizard = Wizard::from_document(blank(), None, String::new());
        wizard.drop_path(root.join("IconSet.png"));
        assert!(wizard.document_open);
        let source = wizard.png_source.clone();
        let count = wizard.doc.tiles[0].len();
        let path = root.join("target/drop-layer-test.PNG");
        image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255]))
            .save_with_format(&path, image::ImageFormat::Png)
            .unwrap();
        wizard.drop_path(path.clone());
        assert_eq!(wizard.doc.tiles[0].len(), count + 1);
        assert_eq!(wizard.png_source, source);
        wizard.draft.angle = 37.0;
        wizard.draft.hue = 120.0;
        wizard.draft.scale = 1.5;
        wizard.commit_draft();
        let before = wizard.doc.tiles[0][wizard.layer].clone();
        let replacement = image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 0, 255, 255]));
        assert!(wizard.insert_image("replacement".into(), replacement, true));
        let layer = &wizard.doc.tiles[0][wizard.layer];
        assert_eq!(layer.name, before.name);
        assert_eq!(layer.angle, 37.0);
        assert_eq!(layer.hue, 120.0);
        assert_eq!(layer.scale, 1.5);
        assert_eq!(layer.pixels.rgba[2], 255);
        wizard.undo();
        assert!(wizard.doc.tiles[0][wizard.layer] == before);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn preview_is_temporary_library_draws_and_all_parameters_undo_together() {
        let ctx = egui::Context::default();
        ui::configure(&ctx);
        let mut wizard = Wizard::from_document(
            Document::from_png(&Path::new(env!("CARGO_MANIFEST_DIR")).join("IconSet.png")).unwrap(),
            None,
            String::new(),
        );
        let original = wizard.doc.tiles[1][0].clone();
        wizard.selected = 1;
        wizard.sync();
        wizard.draft.angle = 33.0;
        wizard.draft.hue = 97.0;
        wizard.draft.scale = 1.5;
        wizard.draft.flip_y = true;
        wizard.load_library();
        wizard.library_open = true;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 1000.0))),
                ..Default::default()
            },
            |ui| wizard.draw_ui(ui),
        );
        assert!(!output.shapes.is_empty());
        output.textures_delta.clear();
        assert!(!wizard.library_textures.is_empty());
        assert!(wizard.has_pending_draft());
        assert!(wizard.doc.tiles[1][0] == original);
        assert!(wizard.undo.is_empty());
        assert!(wizard.preview.is_some());
        wizard.commit_draft();
        assert_eq!(wizard.undo.len(), 1);
        assert_eq!(wizard.doc.tiles[1][0].angle, 33.0);
        assert_eq!(wizard.doc.tiles[1][0].pixels.rgba, original.pixels.rgba);
        wizard.undo();
        assert!(wizard.doc.tiles[1][0] == original);
    }

    #[test]
    fn library_click_inserts_selected_icon_and_can_be_undone() {
        let ctx = egui::Context::default();
        ui::configure(&ctx);
        ctx.all_styles_mut(|style| style.animation_time = 0.0);
        let mut wizard = Wizard::from_document(blank(), None, String::new());
        wizard.load_library();
        wizard.library_open = true;
        let input = || egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 1000.0))),
            ..Default::default()
        };
        // Let the floating window finish its first sizing pass.
        let mut first = ctx.run_ui(input(), |ui| wizard.draw_ui(ui));
        first.textures_delta.clear();
        let mut output = ctx.run_ui(input(), |ui| wizard.draw_ui(ui));
        let texture = wizard.library_textures[0].id();
        let pos = output
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Mesh(mesh) = &shape.shape
                    && mesh.texture_id == texture
                {
                    return Some(mesh.calc_bounds().center());
                }
                None
            })
            .expect("First library icon must be visible");
        output.textures_delta.clear();
        let expected = wizard.packages[0].items[0].as_raw().clone();
        for pressed in [true, false] {
            let mut events = input();
            events.events = vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ];
            let mut output = ctx.run_ui(events, |ui| wizard.draw_ui(ui));
            output.textures_delta.clear();
        }
        assert_eq!(wizard.doc.tiles[0].len(), 1);
        assert_eq!(wizard.doc.tiles[0][0].pixels.rgba, expected);
        assert!(!wizard.library_open);
        wizard.undo();
        assert!(wizard.doc.tiles[0].is_empty());
    }

    fn shortcut_frame(
        wizard: &mut Wizard,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1050.0, 720.0))),
                events,
                ..Default::default()
            },
            |ui| wizard.draw_ui(ui),
        );
        output.textures_delta.clear();
        output
    }

    fn click_test(wizard: &mut Wizard, ctx: &egui::Context, pos: Pos2) {
        for pressed in [true, false] {
            shortcut_frame(
                wizard,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
        }
    }

    fn label_position(output: &egui::FullOutput, label: &str) -> Pos2 {
        output
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape
                    && text.galley.text() == label
                {
                    return Some(text.pos + Vec2::splat(5.0));
                }
                None
            })
            .unwrap_or_else(|| panic!("Missing button: {label}"))
    }

    #[test]
    fn native_copy_paste_work_with_canvas_focus_and_preserve_all_layers() {
        let ctx = egui::Context::default();
        ui::configure(&ctx);
        let mut doc = blank();
        doc.tiles[0] = vec![Layer::new(
            "red".into(),
            image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255])),
        )];
        doc.tiles[0][0].hue = 90.0;
        let original = doc.tiles[0].clone();
        let mut wizard = Wizard::from_document(doc, None, String::new());
        let output = shortcut_frame(&mut wizard, &ctx, vec![]);
        let sheet = wizard.sheet.as_ref().unwrap().id();
        let atlas = output
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Mesh(mesh) = &shape.shape
                    && mesh.texture_id == sheet
                {
                    return Some(mesh.calc_bounds());
                }
                None
            })
            .unwrap();
        let cell = CELL as f32 * wizard.zoom;
        click_test(&mut wizard, &ctx, atlas.min + Vec2::splat(cell / 2.0));
        assert!(ctx.egui_wants_keyboard_input()); // This used to suppress all shortcuts.
        assert!(!ctx.text_edit_focused());
        let output = shortcut_frame(&mut wizard, &ctx, vec![egui::Event::Copy]);
        let text = output
            .platform_output
            .commands
            .iter()
            .find_map(|command| {
                if let egui::OutputCommand::CopyText(text) = command {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .expect("Native clipboard must receive layer data");
        click_test(
            &mut wizard,
            &ctx,
            atlas.min + Vec2::new(cell * 1.5, cell / 2.0),
        );
        assert_eq!(wizard.selected, 1);
        let modifiers = egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        };
        shortcut_frame(
            &mut wizard,
            &ctx,
            vec![
                egui::Event::Paste(text),
                egui::Event::Key {
                    key: egui::Key::V,
                    physical_key: Some(egui::Key::V),
                    pressed: true,
                    repeat: false,
                    modifiers,
                },
            ],
        );
        assert!(wizard.doc.tiles[1] == original);
        assert_eq!(wizard.undo.len(), 1); // No duplicate operation if both event kinds are present.
        wizard.undo();
        assert!(wizard.doc.tiles[1].is_empty());
        let id = egui::Id::new("text-focus-test");
        let mut text = String::new();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut text).id(id))
                .request_focus();
        });
        output.textures_delta.clear();
        assert!(ctx.text_edit_focused());
        shortcut_frame(
            &mut wizard,
            &ctx,
            vec![egui::Event::Paste("ordinary text".into())],
        );
        assert!(wizard.doc.tiles[1].is_empty());
    }

    #[test]
    fn save_tileset_requires_confirmation_and_cancel_preserves_png() {
        let ctx = egui::Context::default();
        ui::configure(&ctx);
        ctx.all_styles_mut(|style| style.animation_time = 0.0);
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("target/save-tileset-confirm-test.png");
        let doc = blank();
        doc.render().save(&path).unwrap();
        let original_bytes = std::fs::read(&path).unwrap();
        let mut wizard = Wizard::from_document(doc, Some(path.clone()), String::new());
        wizard.insert_image(
            "new".into(),
            image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255])),
            false,
        );
        for confirm in [false, true] {
            let output = shortcut_frame(&mut wizard, &ctx, vec![]);
            let save = label_position(&output, crate::i18n::tr("Сохранить Tileset"));
            click_test(&mut wizard, &ctx, save);
            assert!(wizard.confirm_png_save);
            assert_eq!(std::fs::read(&path).unwrap(), original_bytes);
            shortcut_frame(&mut wizard, &ctx, vec![]);
            let output = shortcut_frame(&mut wizard, &ctx, vec![]);
            let action = label_position(
                &output,
                if confirm {
                    crate::i18n::tr("Да, перезаписать")
                } else {
                    crate::i18n::tr("Отмена")
                },
            );
            click_test(&mut wizard, &ctx, action);
            assert!(!wizard.confirm_png_save);
            if !confirm {
                assert_eq!(std::fs::read(&path).unwrap(), original_bytes);
            }
        }
        assert_eq!(read_png(&path).unwrap(), wizard.doc.render());
        assert!(wizard.dirty); // Flattening PNG does not save the layered project.
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn mouse_rectangle_and_layer_drag_drop_work_in_real_ui() {
        let ctx = egui::Context::default();
        ui::configure(&ctx);
        let mut w = Wizard::from_document(blank(), None, String::new());
        w.doc.tiles[0] = vec![
            Layer::new(
                "red".into(),
                image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255])),
            ),
            Layer::new(
                "blue".into(),
                image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 0, 255, 255])),
            ),
        ];
        w.sync();
        let frame = |w: &mut Wizard, events: Vec<egui::Event>| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 1800.0))),
                    events,
                    ..Default::default()
                },
                |ui| w.draw_ui(ui),
            );
            output.textures_delta.clear();
            output
        };
        let mesh_rect = |output: &egui::FullOutput, id| {
            output
                .shapes
                .iter()
                .find_map(|s| {
                    if let egui::Shape::Mesh(mesh) = &s.shape
                        && mesh.texture_id == id
                    {
                        Some(mesh.calc_bounds())
                    } else {
                        None
                    }
                })
                .unwrap()
        };
        let pointer = |pos, pressed| {
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ]
        };
        let output = frame(&mut w, vec![]);
        let source = mesh_rect(&output, w.thumbnails[0].id()).center();
        let target = mesh_rect(&output, w.thumbnails[1].id()).center() - Vec2::new(0.0, 10.0);
        frame(&mut w, pointer(source, true));
        frame(&mut w, vec![egui::Event::PointerMoved(target)]);
        frame(&mut w, pointer(target, false));
        assert_eq!(w.doc.tiles[0][1].name, "red");
        let output = frame(&mut w, vec![]);
        let source = mesh_rect(&output, w.thumbnails[1].id()).center();
        let atlas = mesh_rect(&output, w.sheet.as_ref().unwrap().id());
        let cell = CELL as f32 * w.zoom;
        let target = atlas.min + Vec2::new(cell * 1.5, cell / 2.0);
        frame(&mut w, pointer(source, true));
        frame(&mut w, vec![egui::Event::PointerMoved(target)]);
        frame(&mut w, pointer(target, false));
        assert_eq!(w.doc.tiles[1][0].name, "red");
        assert_eq!(w.doc.tiles[0].len(), 1);
        let start = atlas.min + Vec2::splat(cell / 2.0);
        let end = atlas.min + Vec2::splat(cell * 1.5);
        frame(&mut w, pointer(start, true));
        frame(&mut w, vec![egui::Event::PointerMoved(end)]);
        frame(&mut w, pointer(end, false));
        assert_eq!(w.selected_cells(), vec![0, 1, 16, 17]);
    }
}
