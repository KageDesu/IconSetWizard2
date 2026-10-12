use crate::*;
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TileCopy {
    pub x: usize,
    pub y: usize,
    pub layers: Vec<Layer>,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wizard() -> Wizard {
        Wizard::from_document(crate::blank(), None, String::new())
    }
    fn root(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("{name}-{nonce}"));
        std::fs::create_dir_all(&path).unwrap();
        path
    }
    fn layer(color: [u8; 4]) -> Layer {
        Layer::new(
            "test".into(),
            image::RgbaImage::from_pixel(32, 32, image::Rgba(color)),
        )
    }
    #[test]
    fn rectangle_ctrl_selection_group_copy_paste_and_bounds_are_atomic() {
        let mut w = wizard();
        w.doc.tiles[0] = vec![layer([255, 0, 0, 255])];
        w.doc.tiles[17] = vec![layer([0, 255, 0, 80])];
        w.sync();
        w.select_cell(0, false, false);
        w.select_cell(17, false, true);
        assert_eq!(w.selected_cells(), vec![0, 1, 16, 17]);
        let group = w.copy_group();
        w.select_cell(34, false, false);
        assert!(w.paste_group(group.clone()));
        assert_eq!(w.selected_cells(), vec![34, 35, 50, 51]);
        assert_eq!(w.doc.tiles[34][0].pixels.rgba[0], 255);
        assert_eq!(w.doc.tiles[51][0].pixels.rgba[3], 80);
        w.undo();
        assert!(w.doc.tiles[34].is_empty());
        assert!(w.doc.tiles[51].is_empty());
        w.select_cell(15, false, false);
        let before = w.doc.tiles.clone();
        assert!(!w.paste_group(group));
        assert!(w.doc.tiles == before);
        w.select_cell(1, false, false);
        w.select_cell(16, true, false);
        assert_eq!(w.selected_cells(), vec![1, 16]);
        let group = w.copy_group();
        w.select_cell(32, false, false);
        assert!(w.paste_group(group));
        assert_eq!(w.selected_cells(), vec![33, 48]); // A sparse group need not contain its bounding-box corner.
    }
    #[test]
    fn duplication_recolor_preserves_original_alpha_and_undo() {
        let mut w = wizard();
        w.doc.tiles[0] = vec![layer([255, 0, 0, 128])];
        w.sync();
        let original = w.doc.tiles[0][0].pixels.clone();
        w.duplicate_layer();
        assert_eq!(w.doc.tiles[0].len(), 2);
        assert!(Arc::ptr_eq(&original, &w.doc.tiles[0][1].pixels));
        w.color_from = [255, 0, 0];
        w.color_to = [0, 0, 255];
        w.replace_color();
        assert_eq!(&w.doc.tiles[0][1].pixels.rgba[..4], &[0, 0, 255, 128]);
        assert_eq!(&original.rgba[..4], &[255, 0, 0, 128]);
        assert_eq!(w.doc.tiles[0][0].pixels.rgba, original.rgba);
        w.undo();
        assert_eq!(w.doc.tiles[0][1].pixels.rgba, original.rgba);
        w.undo();
        assert_eq!(w.doc.tiles[0].len(), 1);
        w.doc.tiles[1] = vec![layer([253, 0, 0, 80]), layer([255, 0, 0, 0])];
        w.selection = BTreeSet::from([0, 1]);
        w.selected = 0;
        w.sync();
        w.color_all_layers = true;
        w.color_tolerance = 2;
        w.replace_color();
        assert_eq!(&w.doc.tiles[1][0].pixels.rgba[..4], &[0, 0, 255, 80]);
        assert_eq!(&w.doc.tiles[1][1].pixels.rgba[..4], &[255, 0, 0, 0]);
    }
    #[test]
    fn move_layers_within_and_between_cells_and_full_destination_rollback() {
        let mut w = wizard();
        w.doc.tiles[0] = vec![layer([255, 0, 0, 255]), layer([0, 0, 255, 255])];
        w.sync();
        assert!(w.move_layer(LayerDrag { tile: 0, layer: 0 }, 0, 2));
        assert_eq!(w.doc.tiles[0][1].pixels.rgba[0], 255);
        assert!(w.move_layer(LayerDrag { tile: 0, layer: 1 }, 1, 0));
        assert_eq!(w.doc.tiles[0].len(), 1);
        assert_eq!(w.doc.tiles[1][0].pixels.rgba[0], 255);
        w.undo();
        assert_eq!(w.doc.tiles[0].len(), 2);
        assert!(w.doc.tiles[1].is_empty());
        w.doc.tiles[2] = vec![layer([0, 1, 2, 255]); 128];
        let before = w.doc.tiles.clone();
        assert!(!w.move_layer(LayerDrag { tile: 0, layer: 0 }, 2, 128));
        assert!(w.doc.tiles == before);
    }
    #[test]
    fn exports_selected_cells_as_individual_exact_pngs() {
        let mut w = wizard();
        w.doc.tiles[0] = vec![layer([255, 0, 0, 128])];
        w.doc.tiles[17] = vec![layer([0, 0, 255, 255])];
        w.selection = BTreeSet::from([0, 17]);
        w.sync();
        let folder = root("export-selection");
        assert_eq!(w.export_cells_to(&folder).unwrap(), 2);
        for i in [0, 17] {
            let path = folder.join(format!("icon_{i:04}.png"));
            assert_eq!(read_png(&path).unwrap(), w.doc.render_tile(i));
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(folder).unwrap();
    }
    #[test]
    fn library_search_favorites_survive_restart_and_folder_relocation() {
        let mut w = wizard();
        w.load_library();
        let category = w
            .packages
            .iter()
            .position(|p| p.name == "MV Battlers")
            .unwrap();
        let folder = root("preferences");
        w.initialize_storage(folder.clone());
        w.library_query = "bAtTlErS #1".into();
        assert!(w.library_matches(category, 0));
        assert!(!w.library_matches(0, 0));
        w.toggle_favorite(category, 0);
        w.library_favorites_only = true;
        assert!(w.library_matches(category, 0));
        assert!(!w.library_matches(category, 1));
        let key = w.favorite_key(category, 0);
        let preferences = storage::load_preferences(&folder).unwrap();
        assert!(preferences.favorites.contains(&key));
        w.packages[category].path =
            Path::new("another-folder").join(w.packages[category].path.file_name().unwrap());
        assert_eq!(w.favorite_key(category, 0), key);
        drop(w);
        std::fs::remove_file(folder.join("preferences.json")).unwrap();
        let _ = std::fs::remove_dir(folder.join("recovery"));
        let _ = std::fs::remove_dir(folder);
    }
    #[test]
    fn autosave_recovers_layers_pending_draft_paths_and_skips_active_session() {
        let folder = root("recovery");
        let mut w = wizard();
        w.initialize_storage(folder.clone());
        w.insert_image(
            "red".into(),
            image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 80])),
            false,
        );
        w.png_source = Some(folder.join("original.png"));
        w.source = w.png_source.clone();
        w.draft.angle = 37.0;
        w.autosave_last = std::time::Instant::now() - std::time::Duration::from_secs(31);
        w.autosave_tick(&egui::Context::default());
        let path = w.autosave_store.as_ref().unwrap().path.clone();
        assert_eq!(w.doc.tiles[0][0].angle, 0.0);
        assert_eq!(
            storage::read_recovery(&path).unwrap().document.tiles[0][0].angle,
            37.0
        );
        assert!(storage::candidates(&folder).is_empty()); // A second window must not recover a live editor.
        drop(w);
        assert_eq!(storage::candidates(&folder), vec![path.clone()]);
        let mut recovered = wizard();
        recovered.initialize_storage(folder.clone());
        assert!(recovered.recovery_open);
        recovered.restore_recovery(&path).unwrap();
        assert!(recovered.dirty);
        assert_eq!(recovered.doc.tiles[0][0].angle, 37.0);
        assert_eq!(recovered.png_source, Some(folder.join("original.png")));
        assert_eq!(recovered.doc.tiles[0][0].pixels.rgba[3], 80);
        recovered.clear_recovery();
        drop(recovered);
        assert!(storage::candidates(&folder).is_empty());
        let _ = std::fs::remove_dir(folder.join("recovery"));
        let _ = std::fs::remove_dir(folder);
    }
}
#[derive(Clone, Copy)]
pub struct LayerDrag {
    pub tile: usize,
    pub layer: usize,
}

impl Wizard {
    pub(super) fn selected_cells(&self) -> Vec<usize> {
        if self.selection.contains(&self.selected) {
            self.selection.iter().copied().collect()
        } else {
            vec![self.selected]
        }
    }
    pub(super) fn rectangle(&self, a: usize, b: usize) -> BTreeSet<usize> {
        let (x1, x2) = ((a % COLS).min(b % COLS), (a % COLS).max(b % COLS));
        let (y1, y2) = ((a / COLS).min(b / COLS), (a / COLS).max(b / COLS));
        (y1..=y2)
            .flat_map(|y| (x1..=x2).map(move |x| y * COLS + x))
            .filter(|i| *i < self.doc.tiles.len())
            .collect()
    }
    pub(super) fn select_cell(&mut self, index: usize, ctrl: bool, shift: bool) {
        self.commit_draft();
        if shift {
            self.selection = self.rectangle(self.selection_anchor, index);
        } else if ctrl {
            if !self.selection.remove(&index) {
                self.selection.insert(index);
            }
            if self.selection.is_empty() {
                self.selection.insert(index);
            }
            self.selection_anchor = index;
        } else {
            self.selection = BTreeSet::from([index]);
            self.selection_anchor = index;
        }
        self.selected = if self.selection.contains(&index) {
            index
        } else {
            *self.selection.first().unwrap()
        };
        self.layer = 0;
        self.sync();
    }
    pub(super) fn copy_group(&self) -> Vec<TileCopy> {
        let cells = self.selected_cells();
        let x = cells.iter().map(|i| i % COLS).min().unwrap();
        let y = cells.iter().map(|i| i / COLS).min().unwrap();
        cells
            .iter()
            .map(|i| TileCopy {
                x: i % COLS - x,
                y: i / COLS - y,
                layers: self.doc.tiles[*i].clone(),
            })
            .collect()
    }
    pub(super) fn paste_group(&mut self, group: Vec<TileCopy>) -> bool {
        if group.is_empty() || group.len() > self.doc.tiles.len() {
            return false;
        }
        let x = self.selected % COLS;
        let y = self.selected / COLS;
        let mut targets = BTreeSet::new();
        for tile in &group {
            if tile.x >= COLS - x || tile.y >= self.doc.tiles.len() / COLS - y {
                self.status = crate::i18n::tr(
                    "Группа не помещается в атлас. Выберите другую ячейку или добавьте ряд.",
                )
                .into();
                return false;
            }
            let index = (y + tile.y) * COLS + x + tile.x;
            if !targets.insert(index) {
                return false;
            }
        }
        if self.change(|doc| {
            for tile in group {
                doc.tiles[(y + tile.y) * COLS + x + tile.x] = tile.layers;
            }
        }) {
            self.selection = targets;
            if !self.selection.contains(&self.selected) {
                self.selected = *self.selection.first().unwrap();
                self.sync();
            }
            self.status = crate::i18n::tr("Группа иконок вставлена. Ctrl+Z — отменить.").into();
            true
        } else {
            false
        }
    }
    pub(super) fn delete_cells(&mut self) {
        let cells = self.selected_cells();
        self.change(|doc| {
            for index in cells {
                doc.tiles[index].clear();
            }
        });
    }
    pub(super) fn duplicate_layer(&mut self) {
        self.commit_draft();
        let (tile, index) = (self.selected, self.layer);
        let Some(mut layer) = self.doc.tiles[tile].get(index).cloned() else {
            return;
        };
        layer.name = crate::tf!("{} — копия", "{} — copy", "{} — 副本", layer.name);
        if self.change(|doc| doc.tiles[tile].insert(index + 1, layer)) {
            self.layer = index + 1;
            self.sync();
            self.status = crate::i18n::tr("Слой продублирован.").into();
        }
    }
    pub(super) fn move_layer(&mut self, from: LayerDrag, tile: usize, mut position: usize) -> bool {
        if tile >= self.doc.tiles.len()
            || from.tile >= self.doc.tiles.len()
            || from.layer >= self.doc.tiles[from.tile].len()
        {
            return false;
        }
        self.commit_draft();
        position = position.min(self.doc.tiles[tile].len());
        if from.tile == tile && (position == from.layer || position == from.layer + 1) {
            return false;
        }
        if from.tile == tile && position > from.layer {
            position -= 1;
        }
        if self.change(|doc| {
            let layer = doc.tiles[from.tile].remove(from.layer);
            doc.tiles[tile].insert(position, layer);
        }) {
            self.selected = tile;
            self.selection = BTreeSet::from([tile]);
            self.layer = position;
            self.sync();
            self.status = crate::i18n::tr("Слой перемещён. Ctrl+Z — отменить.").into();
            true
        } else {
            false
        }
    }
    pub(super) fn recolor_layer(
        layer: &mut Layer,
        from: [u8; 3],
        to: [u8; 3],
        tolerance: u8,
    ) -> usize {
        let count = layer
            .pixels
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] > 0 && (0..3).all(|c| p[c].abs_diff(from[c]) <= tolerance))
            .count();
        if count > 0 {
            for p in Arc::make_mut(&mut layer.pixels).rgba.as_chunks_mut::<4>().0 {
                if p[3] > 0 && (0..3).all(|c| p[c].abs_diff(from[c]) <= tolerance) {
                    p[..3].copy_from_slice(&to);
                }
            }
        }
        count
    }
    pub(super) fn replace_color(&mut self) {
        self.commit_draft();
        let (from, to, tolerance, all, index) = (
            self.color_from,
            self.color_to,
            self.color_tolerance,
            self.color_all_layers,
            self.layer,
        );
        let cells = if all {
            self.selected_cells()
        } else {
            vec![self.selected]
        };
        let mut count = 0;
        let mut candidate = self.doc.clone();
        for tile in cells {
            for (number, layer) in candidate.tiles[tile].iter_mut().enumerate() {
                if all || number == index {
                    count += Self::recolor_layer(layer, from, to, tolerance);
                }
            }
        }
        if count > 0 && from != to {
            self.change(|doc| *doc = candidate);
        }
        self.status = crate::tf!(
            "Замена цвета: {count} пикселей. Прозрачность сохранена.",
            "Color replacement: {count} pixels. Transparency preserved.",
            "颜色替换：{count}个像素。透明度已保留。"
        );
    }
    pub(super) fn export_cells_to(&mut self, folder: &Path) -> Result<usize, String> {
        self.commit_draft();
        let cells = self.selected_cells();
        for index in &cells {
            let mut bytes = std::io::Cursor::new(Vec::new());
            self.doc
                .render_tile(*index)
                .write_to(&mut bytes, image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;
            atomic_write(
                &folder.join(format!("icon_{index:04}.png")),
                &bytes.into_inner(),
            )?;
        }
        Ok(cells.len())
    }
    pub(super) fn export_selection(&mut self) {
        let Some(folder) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let exists = self
            .selected_cells()
            .iter()
            .any(|i| folder.join(format!("icon_{i:04}.png")).exists());
        if exists
            && rfd::MessageDialog::new()
                .set_title(crate::i18n::tr("Экспорт иконок"))
                .set_description(crate::i18n::tr(
                    "Некоторые icon_XXXX.png уже существуют. Перезаписать их?",
                ))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                != rfd::MessageDialogResult::Yes
        {
            return;
        }
        self.status = match self.export_cells_to(&folder) {
            Ok(count) => crate::tf!(
                "Экспортировано {count} отдельных PNG в {}",
                "Exported {count} individual PNGs to {}",
                "已将{count}个独立PNG导出至{}",
                folder.display()
            ),
            Err(e) => crate::tf!("Ошибка экспорта: {e}", "Export error: {e}", "导出错误：{e}"),
        };
    }
    pub(super) fn favorite_key(&self, category: usize, index: usize) -> String {
        format!(
            "{}:{index}",
            self.packages[category]
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
        )
    }
    pub(super) fn toggle_favorite(&mut self, category: usize, index: usize) {
        let key = self.favorite_key(category, index);
        if !self.preferences.favorites.remove(&key) {
            self.preferences.favorites.insert(key);
        }
        self.persist_preferences();
    }
    pub(super) fn persist_preferences(&mut self) {
        self.preferences.library_folder = Some(self.library_folder.clone());
        if let Some(root) = &self.storage_root
            && let Err(e) = storage::save_preferences(root, &self.preferences)
        {
            self.status = crate::tf!(
                "Не удалось сохранить избранное: {e}",
                "Could not save preferences: {e}",
                "无法保存设置：{e}"
            );
        }
    }
    pub(super) fn library_matches(&self, category: usize, index: usize) -> bool {
        if self.library_favorites_only
            && !self
                .preferences
                .favorites
                .contains(&self.favorite_key(category, index))
        {
            return false;
        }
        let p = &self.packages[category];
        let text = format!(
            "{} {} {} #{}",
            p.name,
            p.info,
            p.path.file_name().unwrap_or_default().to_string_lossy(),
            index + 1
        )
        .to_lowercase();
        self.library_query
            .to_lowercase()
            .split_whitespace()
            .all(|word| text.contains(word))
    }
    pub(super) fn initialize_storage(&mut self, root: PathBuf) {
        match storage::load_preferences(&root) {
            Ok(p) => {
                if std::env::var_os("ISW_LANGUAGE").is_none() {
                    self.language = crate::i18n::Language::startup(p.language);
                    crate::i18n::set_language(self.language);
                }
                self.preferences = p;
                if let Some(folder) = &self.preferences.library_folder
                    && folder.is_dir()
                {
                    self.library_folder = folder.clone();
                }
            }
            Err(e) => {
                self.status = crate::tf!(
                    "Ошибка настроек: {e}",
                    "Preferences error: {e}",
                    "设置错误：{e}"
                )
            }
        }
        self.recovery_candidates = storage::candidates(&root);
        self.recovery_open = !self.recovery_candidates.is_empty();
        match storage::RecoveryStore::new(&root) {
            Ok(store) => self.autosave_store = Some(store),
            Err(e) => {
                self.status = crate::tf!(
                    "Автосохранение недоступно: {e}",
                    "Autosave unavailable: {e}",
                    "自动保存不可用：{e}"
                )
            }
        }
        self.storage_root = Some(root);
    }
    pub(super) fn autosave_now(&mut self) -> Result<(), String> {
        let Some(store) = &self.autosave_store else {
            return Ok(());
        };
        let mut doc = self.doc.clone();
        doc.version = 2;
        if let Some(layer) = doc.tiles[self.selected].get_mut(self.layer) {
            *layer = self.draft.clone();
        }
        store.write(&storage::Recovery {
            version: 1,
            document: doc,
            source: self.source.clone(),
            png_source: self.png_source.clone(),
            selected: self.selected,
            layer: self.layer,
        })?;
        self.autosave_needed = false;
        self.autosave_draft = Some(self.draft.clone());
        self.autosave_last = std::time::Instant::now();
        Ok(())
    }
    pub(super) fn autosave_tick(&mut self, ctx: &egui::Context) {
        if self.autosave_enabled && self.autosave_store.is_some() {
            let pending = (self.has_pending_draft()
                || self
                    .autosave_store
                    .as_ref()
                    .is_some_and(|s| s.path.exists()))
                && self.autosave_draft.as_ref() != Some(&self.draft);
            if self.autosave_last.elapsed().as_secs() >= 30
                && (self.autosave_needed || pending)
                && let Err(e) = self.autosave_now()
            {
                self.status = crate::tf!(
                    "Ошибка автосохранения: {e}",
                    "Autosave error: {e}",
                    "自动保存错误：{e}"
                );
                self.autosave_last = std::time::Instant::now();
            }
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
    }
    pub(super) fn clear_recovery(&mut self) {
        if let Some(store) = &self.autosave_store {
            store.clear();
        }
        if let Some(path) = self.restored_from.take() {
            storage::remove_recovery(&path);
            self.recovery_candidates.retain(|p| p != &path);
        }
        self.autosave_needed = false;
        self.autosave_draft = None;
    }
    pub(super) fn restore_recovery(&mut self, path: &Path) -> Result<(), String> {
        let snapshot = storage::read_recovery(path)?;
        self.doc = snapshot.document;
        self.source = snapshot.source;
        self.png_source = snapshot.png_source;
        self.selected = snapshot.selected.min(self.doc.tiles.len() - 1);
        self.selection = BTreeSet::from([self.selected]);
        self.layer = snapshot.layer;
        self.undo.clear();
        self.redo.clear();
        self.dirty = true;
        self.document_open = true;
        self.sync();
        self.restored_from = Some(path.to_owned());
        self.autosave_needed = true;
        self.recovery_open = false;
        self.overwrite_png = false;
        self.selection_anchor = self.selected;
        self.status =
            crate::i18n::tr("Проект восстановлен. Сохраните его, чтобы сохранить изменения.")
                .into();
        self.autosave_now()?;
        Ok(())
    }
}
