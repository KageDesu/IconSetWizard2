use crate::*;
use egui::{Align, Align2, CornerRadius, FontId, Layout, Margin, RichText};

const AUTHOR_LINKS: [(SocialIcon, &str); 3] = [
    (SocialIcon::Website, "https://kdworkshop.net/"),
    (
        SocialIcon::YouTube,
        "https://www.youtube.com/channel/UCA3R61ojF5vp5tGwJ1YqdgQ?",
    ),
    (SocialIcon::Boosty, "https://boosty.to/kagedesu"),
];

#[derive(Clone, Copy)]
enum SocialIcon {
    Website,
    YouTube,
    Boosty,
}

fn social_button(ui: &mut egui::Ui, symbol: SocialIcon, url: &str) -> egui::Response {
    let label = match symbol {
        SocialIcon::Website => crate::i18n::tr("Сайт автора"),
        SocialIcon::YouTube => "YouTube",
        SocialIcon::Boosty => "Boosty",
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    let (fill, border) = control_colors(&response, false, true);
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    ui.painter().rect(
        rect,
        10.0,
        fill,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    let center = rect.center();
    match symbol {
        SocialIcon::Website => {
            let stroke = Stroke::new(1.8, ACCENT);
            ui.painter().circle_stroke(center, 16.0, stroke);
            ui.painter().line_segment(
                [center - Vec2::new(16.0, 0.0), center + Vec2::new(16.0, 0.0)],
                stroke,
            );
            for offset in [-8.0, 8.0] {
                ui.painter().line_segment(
                    [
                        center + Vec2::new(-13.5, offset),
                        center + Vec2::new(13.5, offset),
                    ],
                    stroke,
                );
            }
            let points = (0..=32)
                .map(|i| {
                    let angle = i as f32 / 32.0 * std::f32::consts::TAU;
                    center + Vec2::new(7.0 * angle.cos(), 16.0 * angle.sin())
                })
                .collect();
            ui.painter().add(egui::Shape::line(points, stroke));
        }
        SocialIcon::YouTube => {
            ui.painter().rect_filled(
                Rect::from_center_size(center, Vec2::new(36.0, 26.0)),
                7.0,
                Color32::from_rgb(255, 0, 0),
            );
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    center + Vec2::new(-4.0, -7.0),
                    center + Vec2::new(8.0, 0.0),
                    center + Vec2::new(-4.0, 7.0),
                ],
                Color32::WHITE,
                Stroke::NONE,
            ));
        }
        SocialIcon::Boosty => {
            let orange = Color32::from_rgb(245, 115, 45);
            let point = |x, y| rect.min + Vec2::new(x, y);
            // Orange slanted b / lightning mark, drawn at any display scale.
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    point(26.0, 10.0),
                    point(35.0, 10.0),
                    point(25.0, 30.0),
                    point(17.0, 30.0),
                ],
                orange,
                Stroke::NONE,
            ));
            ui.painter().add(egui::Shape::line(
                vec![
                    point(29.0, 24.0),
                    point(36.0, 24.0),
                    point(38.0, 30.0),
                    point(31.0, 41.0),
                    point(22.0, 43.0),
                    point(17.0, 39.0),
                    point(20.0, 33.0),
                ],
                Stroke::new(6.0, orange),
            ));
        }
    }
    if response.clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
    }
    response.on_hover_text(label)
}

fn sidebar_scroll_style(ui: &mut egui::Ui) {
    // Solid bars reserve a gutter instead of growing over the controls on hover.
    let scroll = &mut ui.style_mut().spacing.scroll;
    *scroll = egui::style::ScrollStyle::solid();
    scroll.bar_width = 8.0;
    scroll.bar_inner_margin = 10.0;
}

const BG: Color32 = Color32::from_rgb(16, 20, 29);
const PANEL: Color32 = Color32::from_rgb(20, 26, 37);
const LINE: Color32 = Color32::from_rgb(39, 49, 66);
const TEXT: Color32 = Color32::from_rgb(231, 237, 246);
const MUTED: Color32 = Color32::from_rgb(140, 155, 176);
const ACCENT: Color32 = Color32::from_rgb(113, 199, 223);
const FILL: Color32 = Color32::from_rgb(29, 38, 53);
const HOVER: Color32 = Color32::from_rgb(48, 69, 89);
const PRESSED: Color32 = Color32::from_rgb(32, 55, 71);

fn control_colors(response: &egui::Response, primary: bool, enabled: bool) -> (Color32, Color32) {
    if !enabled {
        return (if primary { ACCENT } else { FILL }, LINE);
    }
    let fill = if response.is_pointer_button_down_on() {
        if primary {
            Color32::from_rgb(78, 162, 188)
        } else {
            PRESSED
        }
    } else if response.hovered() {
        if primary {
            Color32::from_rgb(153, 223, 243)
        } else {
            HOVER
        }
    } else if primary {
        ACCENT
    } else {
        FILL
    };
    let border = if response.hovered() || response.has_focus() || primary {
        ACCENT
    } else {
        LINE
    };
    (fill, border)
}

fn hover_outline(ui: &egui::Ui, rect: Rect, response: &egui::Response) {
    if response.hovered() || response.has_focus() || response.is_pointer_button_down_on() {
        ui.painter().rect_stroke(
            rect,
            3.0,
            Stroke::new(2.0, ACCENT),
            egui::StrokeKind::Inside,
        );
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
}

pub fn configure(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto-cjk".into(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/NotoSansCJKsc-Regular.otf"
        ))),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("noto-cjk".into());
    }
    ctx.set_fonts(fonts);
    ctx.set_theme(egui::ThemePreference::Dark);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = BG;
    visuals.faint_bg_color = FILL;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = Color32::from_rgb(32, 55, 71);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.corner_radius = CornerRadius::same(7);
        widget.bg_stroke = Stroke::new(1.0, Color32::from_rgb(53, 65, 84));
        widget.bg_fill = FILL;
        widget.weak_bg_fill = FILL;
    }
    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.weak_bg_fill = HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_fill = PRESSED;
    visuals.widgets.active.weak_bg_fill = PRESSED;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    ctx.set_visuals(visuals);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 10.0);
        style.spacing.button_padding = Vec2::new(10.0, 8.0);
        style.spacing.interact_size.y = 34.0;
        for (kind, size) in [
            (egui::TextStyle::Body, 13.0),
            (egui::TextStyle::Button, 12.0),
            (egui::TextStyle::Heading, 18.0),
            (egui::TextStyle::Small, 10.0),
        ] {
            style.text_styles.insert(kind, FontId::proportional(size));
        }
    });
}

#[derive(Clone, Copy)]
enum Icon {
    Grid,
    Open,
    Save,
    Export,
    Undo,
    Redo,
    Plus,
    Copy,
    Paste,
    Layers,
    Up,
    Down,
    Trash,
    Check,
}

fn icon(ui: &egui::Ui, rect: Rect, kind: Icon, color: Color32) {
    let point =
        |x: f32, y: f32| rect.min + Vec2::new(x / 24.0 * rect.width(), y / 24.0 * rect.height());
    let line = |points: &[(f32, f32)]| {
        ui.painter().add(egui::Shape::line(
            points.iter().map(|&(x, y)| point(x, y)).collect(),
            Stroke::new(1.5, color),
        ))
    };
    let box_at = |a: (f32, f32), b: (f32, f32)| {
        ui.painter().rect_stroke(
            Rect::from_min_max(point(a.0, a.1), point(b.0, b.1)),
            1.0,
            Stroke::new(1.5, color),
            egui::StrokeKind::Inside,
        )
    };
    match kind {
        Icon::Grid => {
            box_at((3., 3.), (21., 21.));
            for v in [9., 15.] {
                line(&[(v, 3.), (v, 21.)]);
                line(&[(3., v), (21., v)]);
            }
        }
        Icon::Open => {
            line(&[
                (3., 7.),
                (9., 7.),
                (11., 9.),
                (21., 9.),
                (18., 20.),
                (3., 20.),
                (3., 4.),
                (9., 4.),
                (11., 7.),
            ]);
        }
        Icon::Save => {
            line(&[
                (4., 3.),
                (17., 3.),
                (20., 6.),
                (20., 21.),
                (4., 21.),
                (4., 3.),
            ]);
            box_at((7., 3.), (16., 10.));
            box_at((7., 13.), (17., 21.));
        }
        Icon::Export => {
            line(&[(12., 3.), (12., 15.)]);
            line(&[(7., 10.), (12., 15.), (17., 10.)]);
            line(&[(4., 15.), (4., 21.), (20., 21.), (20., 15.)]);
        }
        Icon::Undo | Icon::Redo => {
            let mirror = |x| {
                if matches!(kind, Icon::Redo) {
                    24. - x
                } else {
                    x
                }
            };
            line(&[(mirror(8.), 4.), (mirror(3.), 9.), (mirror(8.), 14.)]);
            line(&[
                (mirror(3.), 9.),
                (mirror(14.), 9.),
                (mirror(19.), 12.),
                (mirror(20.), 17.),
                (mirror(17.), 21.),
                (mirror(12.), 22.),
            ]);
        }
        Icon::Plus => {
            line(&[(12., 4.), (12., 20.)]);
            line(&[(4., 12.), (20., 12.)]);
        }
        Icon::Copy => {
            box_at((8., 8.), (21., 21.));
            line(&[(4., 16.), (2., 16.), (2., 2.), (16., 2.), (16., 4.)]);
        }
        Icon::Paste => {
            line(&[
                (8., 5.),
                (4., 5.),
                (4., 21.),
                (20., 21.),
                (20., 5.),
                (16., 5.),
            ]);
            box_at((8., 3.), (16., 8.));
        }
        Icon::Layers => {
            line(&[(12., 3.), (22., 8.), (12., 13.), (2., 8.), (12., 3.)]);
            line(&[(2., 12.), (12., 17.), (22., 12.)]);
            line(&[(2., 17.), (12., 22.), (22., 17.)]);
        }
        Icon::Up => {
            line(&[(6., 15.), (12., 9.), (18., 15.)]);
        }
        Icon::Down => {
            line(&[(6., 9.), (12., 15.), (18., 9.)]);
        }
        Icon::Trash => {
            line(&[(3., 6.), (21., 6.)]);
            line(&[(9., 6.), (9., 3.), (15., 3.), (15., 6.)]);
            line(&[(6., 6.), (7., 21.), (17., 21.), (18., 6.)]);
            line(&[(10., 10.), (10., 17.)]);
            line(&[(14., 10.), (14., 17.)]);
        }
        Icon::Check => {
            line(&[(4., 12.), (9., 17.), (20., 6.)]);
        }
    }
}

fn button(
    ui: &mut egui::Ui,
    symbol: Icon,
    text: &str,
    enabled: bool,
    primary: bool,
) -> egui::Response {
    let color = if primary {
        Color32::from_rgb(16, 39, 50)
    } else {
        TEXT
    };
    let galley = ui
        .painter()
        .layout_no_wrap(text.into(), FontId::proportional(12.0), color);
    let width = if text.is_empty() {
        36.0
    } else {
        galley.size().x + 49.0
    };
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, 36.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let opacity = if enabled { 1.0 } else { 0.35 };
    let (fill, border) = control_colors(&response, primary, enabled);
    ui.painter().rect(
        rect,
        7.0,
        fill.gamma_multiply(opacity),
        Stroke::new(1.0, border.gamma_multiply(opacity)),
        egui::StrokeKind::Inside,
    );
    icon(
        ui,
        Rect::from_min_size(rect.min + Vec2::new(10., 9.), Vec2::splat(18.)),
        symbol,
        color.gamma_multiply(opacity),
    );
    if !text.is_empty() {
        ui.painter().galley_with_override_text_color(
            rect.min + Vec2::new(35., (36. - galley.size().y) / 2.),
            galley,
            color.gamma_multiply(opacity),
        );
    }
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

fn eyebrow(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(10.0).color(MUTED).strong());
}
fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(RichText::new(text.into()).size(11.0).color(MUTED));
}
fn panel_frame(color: Color32, margin: i8) -> egui::Frame {
    egui::Frame::new()
        .fill(color)
        .inner_margin(Margin::same(margin))
        .stroke(Stroke::new(1.0, LINE))
}
fn divider(ui: &mut egui::Ui) {
    ui.add_space(5.0);
    ui.separator();
    ui.add_space(5.0);
}

impl Wizard {
    fn filename(&self) -> String {
        self.source
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| crate::i18n::tr("Новый Iconset").into())
    }

    pub(super) fn apply_draft(&mut self) {
        if !self.has_pending_draft() {
            return;
        }
        let (selected, layer) = (self.selected, self.layer);
        let mut draft = self.draft.clone();
        draft.pixels = self.doc.tiles[selected][layer].pixels.clone();
        if self.change(|doc| doc.tiles[selected][layer] = draft) {
            self.status = crate::i18n::tr("Свойства применены. Ctrl+Z — отменить.").into();
        }
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        eyebrow(ui, crate::i18n::tr("ВЫБРАННАЯ ИКОНКА"));
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("#{}", self.selected))
                    .size(26.0)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                muted(ui, "32 × 32");
            });
        });
        ui.add_space(9.0);
        ui.vertical_centered(|ui| {
            if let Some(texture) = &self.preview {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(192.0), Sense::hover());
                checker(ui, rect, 8.0);
                ui.painter().image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                    Color32::WHITE,
                );
                ui.painter().rect_stroke(
                    rect,
                    8.0,
                    Stroke::new(1.0, Color32::from_rgb(69, 84, 107)),
                    egui::StrokeKind::Inside,
                );
            }
            muted(
                ui,
                crate::tf!(
                    "Колонка {} · Ряд {}",
                    "Column {} · Row {}",
                    "列{} · 行{}",
                    self.selected % COLS,
                    self.selected / COLS
                ),
            );
        });
        ui.add_space(6.0);
        muted(
            ui,
            crate::tf!(
                "Выбрано ячеек: {}",
                "Selected cells: {}",
                "所选单元格：{}",
                self.selected_cells().len()
            ),
        );
        ui.horizontal(|ui| {
            if button(ui, Icon::Copy, crate::i18n::tr("Копировать"), true, false)
                .on_hover_text("Ctrl+C")
                .clicked()
            {
                self.copy_icon(ui.ctx());
            }
            if button(
                ui,
                Icon::Paste,
                crate::i18n::tr("Вставить"),
                self.clipboard.is_some(),
                false,
            )
            .on_hover_text(crate::i18n::tr("Ctrl+V · заменяет текущую иконку"))
            .clicked()
            {
                self.paste_icon(None);
            }
        });
        ui.horizontal(|ui| {
            if ui.button(crate::i18n::tr("Экспорт ячеек")).clicked() {
                self.export_selection();
            }
            if ui.button(crate::i18n::tr("Очистить ячейки")).clicked() {
                self.delete_cells();
            }
        });
        divider(ui);
        ui.horizontal(|ui| {
            if ui.button(crate::i18n::tr("Библиотека")).clicked() {
                if self.packages.is_empty() {
                    self.load_library();
                }
                self.library_open = true;
            }
            if ui
                .add_enabled(
                    !self.doc.tiles[self.selected].is_empty(),
                    egui::Button::new(crate::i18n::tr("Заменить PNG")),
                )
                .on_hover_text(crate::i18n::tr(
                    "Заменить картинку выбранного слоя, сохранив его свойства",
                ))
                .clicked()
            {
                self.replace_layer();
            }
        });
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(crate::tf!(
                    "Слои  {}",
                    "Layers  {}",
                    "图层  {}",
                    self.doc.tiles[self.selected].len()
                ))
                .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if button(ui, Icon::Plus, "PNG", true, false)
                    .on_hover_text(crate::i18n::tr("Импортировать PNG в новый слой"))
                    .clicked()
                {
                    self.import_layer();
                }
            });
        });
        let count = self.doc.tiles[self.selected].len();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    count > 0,
                    egui::Button::new(crate::i18n::tr("Дубликат слоя")),
                )
                .clicked()
            {
                self.duplicate_layer();
            }
            if ui
                .add_enabled(
                    count > 0,
                    egui::Button::new(crate::i18n::tr("Замена цвета")),
                )
                .clicked()
            {
                self.color_open = true;
            }
        });
        if count == 0 {
            muted(ui, crate::i18n::tr("Пока пусто. Добавьте слой PNG."));
        }
        egui::ScrollArea::vertical()
            .id_salt("layers")
            .max_height(180.0)
            .show(ui, |ui| {
                for index in (0..count).rev() {
                    let layer = &self.doc.tiles[self.selected][index];
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 54.0),
                        Sense::click_and_drag(),
                    );
                    let selected = self.layer == index;
                    ui.painter().rect(
                        rect,
                        7.0,
                        if response.hovered() {
                            HOVER
                        } else if selected {
                            PRESSED
                        } else {
                            FILL
                        },
                        Stroke::new(
                            1.0,
                            if selected || response.hovered() {
                                ACCENT
                            } else {
                                LINE
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    let thumb =
                        Rect::from_min_size(rect.min + Vec2::new(9., 11.), Vec2::splat(32.));
                    checker(ui, thumb, 4.0);
                    if let Some(texture) = self.thumbnails.get(index) {
                        ui.painter().image(
                            texture.id(),
                            thumb,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                            Color32::WHITE,
                        );
                    }
                    let text_rect = Rect::from_min_max(
                        rect.min + Vec2::new(51., 9.),
                        rect.max - Vec2::new(15., 7.),
                    );
                    let painter = ui.painter().with_clip_rect(text_rect);
                    painter.text(
                        text_rect.min,
                        Align2::LEFT_TOP,
                        &layer.name,
                        FontId::proportional(12.0),
                        TEXT,
                    );
                    painter.text(
                        text_rect.min + Vec2::new(0., 22.),
                        Align2::LEFT_TOP,
                        if layer.visible {
                            crate::tf!(
                                "{}% непрозрачности",
                                "{}% opacity",
                                "{}%不透明度",
                                (f32::from(layer.opacity) * 100. / 255.).round()
                            )
                        } else {
                            crate::i18n::tr("Скрыт").into()
                        },
                        FontId::proportional(10.0),
                        MUTED,
                    );
                    if response.drag_started() {
                        response.dnd_set_drag_payload(editor::LayerDrag {
                            tile: self.selected,
                            layer: index,
                        });
                    }
                    let drop = response
                        .dnd_release_payload::<editor::LayerDrag>()
                        .map(|p| *p);
                    if response
                        .on_hover_text(crate::tf!(
                            "{} · перетащите для смены порядка или в другую ячейку",
                            "{} · drag to reorder or move to another cell",
                            "{} · 拖动以重新排序或移至其他单元格",
                            layer.name
                        ))
                        .clicked()
                    {
                        self.commit_draft();
                        self.layer = index;
                        self.draft = draft_for(&self.doc, self.selected, index);
                    }
                    if let Some(from) = drop {
                        let above = ui.ctx().input(|i| {
                            i.pointer
                                .latest_pos()
                                .is_some_and(|p| p.y < rect.center().y)
                        });
                        self.move_layer(from, self.selected, index + usize::from(above));
                        break;
                    }
                }
            });
        ui.horizontal(|ui| {
            let (s, l) = (self.selected, self.layer);
            if button(ui, Icon::Up, crate::i18n::tr("Выше"), l + 1 < count, false).clicked() {
                self.change(|doc| doc.tiles[s].swap(l, l + 1));
                self.layer = l + 1;
                self.sync();
            }
            if button(ui, Icon::Down, crate::i18n::tr("Ниже"), l > 0, false).clicked() {
                self.change(|doc| doc.tiles[s].swap(l, l - 1));
                self.layer = l - 1;
                self.sync();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if button(ui, Icon::Trash, "", count > 0, false)
                    .on_hover_text(crate::i18n::tr("Удалить слой"))
                    .clicked()
                {
                    self.change(|doc| {
                        doc.tiles[s].remove(l);
                    });
                    self.status = crate::i18n::tr("Слой удалён.").into();
                }
            });
        });
        if self.doc.tiles[self.selected].is_empty() {
            return;
        }
        divider(ui);
        ui.label(RichText::new(crate::i18n::tr("Свойства слоя")).strong());
        ui.add_space(4.0);
        muted(ui, crate::i18n::tr("Название"));
        ui.add(
            egui::TextEdit::singleline(&mut self.draft.name).desired_width(ui.available_width()),
        );
        ui.checkbox(&mut self.draft.visible, crate::i18n::tr("Видимый слой"));
        ui.horizontal(|ui| {
            muted(ui, crate::i18n::tr("Непрозрачность"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                muted(
                    ui,
                    format!("{}%", (f32::from(self.draft.opacity) * 100. / 255.).round()),
                );
            });
        });
        ui.style_mut().spacing.slider_width = ui.available_width() - 2.0;
        ui.add(egui::Slider::new(&mut self.draft.opacity, 0..=255).show_value(false));
        ui.columns(2, |columns| {
            muted(&mut columns[0], crate::i18n::tr("Смещение X"));
            columns[0].add(
                egui::DragValue::new(&mut self.draft.x)
                    .range(-32..=32)
                    .speed(1.0),
            );
            muted(&mut columns[1], crate::i18n::tr("Смещение Y"));
            columns[1].add(
                egui::DragValue::new(&mut self.draft.y)
                    .range(-32..=32)
                    .speed(1.0),
            );
        });
        // Legacy quarter turns are folded into the angle only when edited.
        ui.style_mut().spacing.slider_width = (ui.available_width() - 60.0).max(100.0);
        let mut angle = self.draft.angle + f32::from(self.draft.turns) * 90.0;
        muted(ui, crate::i18n::tr("Поворот, °"));
        if ui
            .add(egui::Slider::new(&mut angle, -180.0..=360.0).step_by(1.0))
            .changed()
        {
            self.draft.angle = angle;
            self.draft.turns = 0;
        }
        ui.horizontal(|ui| {
            for angle in [0.0, 90.0, 180.0, 270.0] {
                if ui.button(format!("{angle}°")).clicked() {
                    self.draft.angle = angle;
                    self.draft.turns = 0;
                }
            }
        });
        muted(ui, crate::i18n::tr("Масштаб содержимого слоя"));
        ui.add(egui::Slider::new(&mut self.draft.scale, 0.1..=4.0).step_by(0.05));
        muted(ui, crate::i18n::tr("Цветовой тон Hue, °"));
        ui.add(egui::Slider::new(&mut self.draft.hue, -180.0..=180.0).step_by(1.0));
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.draft.flip, crate::i18n::tr("↔ Отразить X"))
                .clicked()
            {
                self.draft.flip = !self.draft.flip;
            }
            if ui
                .selectable_label(self.draft.flip_y, crate::i18n::tr("↕ Отразить Y"))
                .clicked()
            {
                self.draft.flip_y = !self.draft.flip_y;
            }
        });
        ui.checkbox(&mut self.draft.smooth, crate::i18n::tr("Сглаживать преобразования"))
            .on_hover_text(crate::i18n::tr("Для пиксельной графики оставьте выключенным. Сглаживание учитывает прозрачность краёв."));
        if button(ui, Icon::Check, crate::i18n::tr("Применить"), true, true).clicked() {
            self.apply_draft();
        }
        if ui.button(crate::i18n::tr("Сбросить изменения")).clicked() {
            self.draft = draft_for(&self.doc, self.selected, self.layer);
        }
        muted(
            ui,
            crate::i18n::tr(
                "Предпросмотр виден сразу. «Применить» фиксирует одну операцию Undo. Сохранение и экспорт также применяют параметры.",
            ),
        );
    }

    fn document_panel(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().interact_size.y = 20.0;
        eyebrow(ui, crate::i18n::tr("ДОКУМЕНТ"));
        ui.heading("Iconset");
        ui.add_space(8.0);
        egui::Frame::new()
            .fill(Color32::from_rgb(26, 35, 49))
            .stroke(Stroke::new(1.0, Color32::from_rgb(49, 65, 88)))
            .corner_radius(9)
            .inner_margin(14)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
                icon(ui, rect, Icon::Grid, ACCENT);
                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(RichText::new(self.filename()).strong().size(12.0)).truncate(),
                )
                .on_hover_text(self.filename());
                muted(
                    ui,
                    format!("512 × {} px", self.doc.tiles.len() / COLS * CELL as usize),
                );
            });
        ui.add_space(10.0);
        for (name, value) in [
            (crate::i18n::tr("Иконок"), self.doc.tiles.len()),
            (crate::i18n::tr("Колонок"), COLS),
            (crate::i18n::tr("Рядов"), self.doc.tiles.len() / COLS),
        ] {
            ui.horizontal(|ui| {
                muted(ui, name);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(value.to_string());
                });
            });
        }
        ui.add_space(6.0);
        if button(
            ui,
            Icon::Plus,
            crate::i18n::tr("Добавить ряд"),
            self.doc.tiles.len() / COLS < MAX_ROWS,
            false,
        )
        .clicked()
        {
            self.change(|doc| doc.tiles.extend(vec![vec![]; COLS]));
            self.status = crate::i18n::tr("Пустой ряд добавлен.").into();
        }
        divider(ui);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
        icon(ui, rect, Icon::Layers, MUTED);
        muted(
            ui,
            crate::i18n::tr("Проект хранит слои.\nPNG — готовая картинка для игры."),
        );
        ui.checkbox(
            &mut self.autosave_enabled,
            crate::i18n::tr("Автосохранение (30 с)"),
        );
        if !self.recovery_candidates.is_empty()
            && ui.button(crate::i18n::tr("Восстановление…")).clicked()
        {
            self.recovery_open = true;
        }
        ui.add_enabled_ui(self.png_source.is_some(), |ui| {
            ui.checkbox(
                &mut self.overwrite_png,
                crate::i18n::tr("Перезаписывать исходный PNG"),
            )
            .on_hover_text(crate::i18n::tr(
                "При экспорте обновлять открытый PNG без диалога выбора файла",
            ));
        });
        if self.overwrite_png {
            muted(
                ui,
                crate::i18n::tr(
                    "Экспорт обновит исходный PNG. Проект со слоями сохраняется отдельно.",
                ),
            );
        }
        ui.add_space(30.0);
        eyebrow(ui, crate::i18n::tr("БЫСТРЫЕ ДЕЙСТВИЯ"));
        for (name, key) in [
            (crate::i18n::tr("Открыть"), "Ctrl O"),
            (crate::i18n::tr("Сохранить"), "Ctrl S"),
            (crate::i18n::tr("Отменить"), "Ctrl Z"),
            (crate::i18n::tr("Копировать"), "Ctrl C"),
            (crate::i18n::tr("Вставить"), "Ctrl V"),
        ] {
            ui.horizontal(|ui| {
                muted(ui, name);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    muted(ui, key);
                });
            });
        }
    }

    pub(super) fn draw_ui(&mut self, ui: &mut egui::Ui) {
        crate::i18n::set_language(self.language);
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested())
            && !(self.dirty || self.has_pending_draft())
        {
            self.clear_recovery();
        }
        if ctx.input(|i| i.viewport().close_requested())
            && (self.dirty || self.has_pending_draft())
            && !self.allow_close
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        if self.confirm_close {
            egui::Window::new(crate::i18n::tr("Несохранённый проект"))
                .collapsible(false)
                .resizable(false)
                .show(&ctx, |ui| {
                    ui.label(crate::i18n::tr(
                        "Сохранить проект со слоями перед закрытием?",
                    ));
                    ui.horizontal(|ui| {
                        if ui.button(crate::i18n::tr("Сохранить")).clicked() && self.save_project()
                        {
                            self.allow_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui
                            .button(crate::i18n::tr("Закрыть без сохранения"))
                            .clicked()
                        {
                            self.clear_recovery();
                            self.allow_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button(crate::i18n::tr("Отмена")).clicked() {
                            self.confirm_close = false;
                        }
                    });
                });
        }
        let dropped = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect::<Vec<_>>()
        });
        for path in dropped {
            self.drop_path(path);
        }
        if self.confirm_png_save {
            let modal = egui::Modal::new(egui::Id::new("confirm-save-tileset")).show(&ctx, |ui| {
                ui.set_max_width(460.0);
                ui.heading(crate::i18n::tr("Сохранить Tileset?"));
                ui.label(crate::i18n::tr(
                    "Перезаписать открытый PNG текущим Tileset?",
                ));
                if let Some(path) = &self.png_source {
                    ui.label(path.display().to_string());
                }
                ui.label(crate::i18n::tr(
                    "Изменения заменят содержимое файла. Слои сохраняются отдельно в проекте.",
                ));
                ui.horizontal(|ui| {
                    if ui.button(crate::i18n::tr("Да, перезаписать")).clicked() {
                        self.save_current_png();
                        self.confirm_png_save = false;
                    }
                    if ui.button(crate::i18n::tr("Отмена")).clicked() {
                        self.confirm_png_save = false;
                    }
                });
            });
            if modal.should_close() {
                self.confirm_png_save = false;
            }
        }
        if !ctx.text_edit_focused()
            && !self.confirm_png_save
            && !self.confirm_close
            && !self.about_open
            && !self.library_open
            && !self.color_open
            && !self.recovery_open
        {
            let events = ctx.input(|i| i.events.clone());
            let copy_event = events
                .iter()
                .any(|event| matches!(event, egui::Event::Copy));
            let cut_event = events.iter().any(|event| matches!(event, egui::Event::Cut));
            let paste_event = events.iter().find_map(|event| {
                if let egui::Event::Paste(text) = event {
                    Some(text.as_str())
                } else {
                    None
                }
            });
            if copy_event || cut_event {
                self.copy_icon(&ctx);
            }
            if cut_event {
                self.delete_cells();
            }
            if let Some(text) = paste_event {
                self.paste_icon(Some(text));
            }
            for (key, action) in [
                (egui::Key::O, 0),
                (egui::Key::S, 1),
                (egui::Key::Z, 2),
                (egui::Key::Y, 3),
                (egui::Key::C, 4),
                (egui::Key::V, 5),
                (egui::Key::X, 6),
                (egui::Key::A, 7),
            ] {
                if ctx.input(|i| i.modifiers.command && i.key_pressed(key)) {
                    match action {
                        0 => self.open_dialog(),
                        1 => {
                            self.save_project();
                        }
                        2 => {
                            if ctx.input(|i| i.modifiers.shift) {
                                self.redo();
                            } else {
                                self.undo();
                            }
                        }
                        3 => self.redo(),
                        4 if !copy_event => self.copy_icon(&ctx),
                        5 if paste_event.is_none() => self.paste_icon(None),
                        6 if !cut_event => {
                            self.copy_icon(&ctx);
                            self.delete_cells();
                        }
                        7 => {
                            self.selection = (0..self.doc.tiles.len()).collect();
                        }
                        _ => {}
                    }
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
                self.delete_cells();
            }
        }
        egui::Panel::top("header")
            .exact_size(76.0)
            .frame(panel_frame(PANEL, 16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(crate::i18n::tr("О программе")).clicked() {
                            self.about_open = true;
                        }
                        ui.menu_button(crate::i18n::tr("Язык"), |ui| {
                            for language in crate::i18n::Language::ALL {
                                if ui
                                    .selectable_label(self.language == language, language.name())
                                    .clicked()
                                {
                                    self.language = language;
                                    crate::i18n::set_language(language);
                                    self.preferences.language = Some(language);
                                    self.status = crate::i18n::tr("Язык изменён.").into();
                                    self.persist_preferences();
                                    // Package diagnostics are generated in the active language.
                                    if !self.packages.is_empty() || !self.library_errors.is_empty()
                                    {
                                        self.load_library();
                                    }
                                    ui.close();
                                }
                            }
                        });

                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(42.0), Sense::hover());
                            ui.painter().rect(
                                rect,
                                12.,
                                Color32::from_rgb(33, 56, 72),
                                Stroke::new(1., Color32::from_rgb(57, 85, 101)),
                                egui::StrokeKind::Inside,
                            );
                            icon(ui, rect.shrink(9.0), Icon::Grid, ACCENT);
                            ui.vertical(|ui| {
                                ui.label(RichText::new("IconSet Wizard 2").size(20.0).strong());
                                muted(ui, crate::i18n::tr("Мастерская игровых иконок"));
                            });
                            ui.add_space(24.0);
                            ui.label(RichText::new(self.filename()).size(13.0));
                            let (dot, _) = ui.allocate_exact_size(Vec2::splat(6.0), Sense::hover());
                            ui.painter().circle_filled(
                                dot.center(),
                                3.0,
                                if self.dirty {
                                    Color32::from_rgb(227, 178, 108)
                                } else {
                                    Color32::from_rgb(120, 186, 161)
                                },
                            );
                            muted(
                                ui,
                                if self.dirty {
                                    crate::i18n::tr("Изменён")
                                } else {
                                    crate::i18n::tr("Сохранён")
                                },
                            );
                        });
                    });
                });
            });
        egui::Panel::top("toolbar")
            .exact_size(61.0)
            .frame(panel_frame(PANEL, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if button(ui, Icon::Open, crate::i18n::tr("Открыть"), true, false)
                        .on_hover_text("Ctrl+O")
                        .clicked()
                    {
                        self.open_dialog();
                    }
                    if button(
                        ui,
                        Icon::Save,
                        crate::i18n::tr("Сохранить проект"),
                        true,
                        false,
                    )
                    .on_hover_text("Ctrl+S")
                    .clicked()
                    {
                        self.save_project();
                    }
                    if button(ui, Icon::Export, crate::i18n::tr("Экспорт PNG"), true, true)
                        .clicked()
                    {
                        self.export();
                    }
                    if button(
                        ui,
                        Icon::Save,
                        crate::i18n::tr("Сохранить Tileset"),
                        self.png_source.is_some(),
                        false,
                    )
                    .on_hover_text(crate::i18n::tr(
                        "Перезаписать открытый PNG с подтверждением",
                    ))
                    .clicked()
                    {
                        self.confirm_png_save = true;
                    }
                    ui.add_space(10.0);
                    if button(
                        ui,
                        Icon::Undo,
                        crate::i18n::tr("Отменить"),
                        !self.undo.is_empty(),
                        false,
                    )
                    .on_hover_text("Ctrl+Z")
                    .clicked()
                    {
                        self.undo();
                    }
                    if button(
                        ui,
                        Icon::Redo,
                        crate::i18n::tr("Повторить"),
                        !self.redo.is_empty(),
                        false,
                    )
                    .on_hover_text("Ctrl+Y")
                    .clicked()
                    {
                        self.redo();
                    }
                });
            });
        egui::Panel::bottom("status")
            .exact_size(34.0)
            .frame(panel_frame(Color32::from_rgb(23, 33, 47), 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot, _) = ui.allocate_exact_size(Vec2::splat(6.0), Sense::hover());
                    ui.painter()
                        .circle_filled(dot.center(), 3., Color32::from_rgb(120, 186, 161));
                    ui.add(
                        egui::Label::new(RichText::new(&self.status).size(11.0).color(MUTED))
                            .truncate(),
                    )
                    .on_hover_text(&self.status);
                });
            });
        self.textures(&ctx);
        egui::Panel::left("document")
            .exact_size(210.0)
            .resizable(false)
            .frame(panel_frame(PANEL, 18))
            .show(ui, |ui| {
                sidebar_scroll_style(ui);
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .id_salt("document-scroll")
                    .show(ui, |ui| self.document_panel(ui));
            });
        egui::Panel::right("inspector")
            .exact_size(330.0)
            .resizable(false)
            .frame(panel_frame(PANEL, 20))
            .show(ui, |ui| {
                sidebar_scroll_style(ui);
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .id_salt("inspector-scroll")
                    .show(ui, |ui| self.inspector(ui));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(20))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(crate::i18n::tr("Полотно"));
                        muted(
                            ui,
                            crate::i18n::tr(
                                "Ctrl+клик — группа · Shift+клик / протягивание — прямоугольник",
                            ),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        egui::ComboBox::from_id_salt("zoom")
                            .width(65.0)
                            .selected_text(format!("{}%", (self.zoom * 100.) as i32))
                            .show_ui(ui, |ui| {
                                for zoom in [0.5, 1., 1.5, 2., 3., 4.] {
                                    ui.selectable_value(
                                        &mut self.zoom,
                                        zoom,
                                        format!("{}%", (zoom * 100.) as i32),
                                    );
                                }
                            });
                        if button(ui, Icon::Grid, "", true, self.grid)
                            .on_hover_text(crate::i18n::tr("Показать сетку"))
                            .clicked()
                        {
                            self.grid = !self.grid;
                        }
                    });
                });
                divider(ui);
                let height = (ui.available_height() - 26.0).max(40.0);
                egui::ScrollArea::both()
                    .id_salt("atlas")
                    .max_height(height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let cell = CELL as f32 * self.zoom;
                        let size = Vec2::new(
                            COLS as f32 * cell,
                            (self.doc.tiles.len() / COLS) as f32 * cell,
                        );
                        let (rect, response) =
                            ui.allocate_exact_size(size, Sense::click_and_drag());
                        checker(ui, rect, 8.0);
                        if let Some(texture) = &self.sheet {
                            ui.painter().image(
                                texture.id(),
                                rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                                Color32::WHITE,
                            );
                        }
                        if self.grid {
                            let stroke = Stroke::new(1.0, Color32::from_black_alpha(100));
                            for x in 0..=COLS {
                                let x = rect.left() + x as f32 * cell;
                                ui.painter().line_segment(
                                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                                    stroke,
                                );
                            }
                            let clip = ui.clip_rect().intersect(rect);
                            let first = ((clip.top() - rect.top()) / cell).floor().max(0.) as usize;
                            let last = (((clip.bottom() - rect.top()) / cell).ceil() as usize)
                                .min(self.doc.tiles.len() / COLS);
                            for y in first..=last {
                                let y = rect.top() + y as f32 * cell;
                                ui.painter().line_segment(
                                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                                    stroke,
                                );
                            }
                        }
                        let selection = Rect::from_min_size(
                            rect.min
                                + Vec2::new(
                                    (self.selected % COLS) as f32 * cell,
                                    (self.selected / COLS) as f32 * cell,
                                ),
                            Vec2::splat(cell),
                        );
                        if let Some(preview) = &self.preview {
                            // Cover the committed tile before drawing the draft, including transparent pixels.
                            checker(ui, selection, 8.0);
                            ui.painter().image(
                                preview.id(),
                                selection,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                                Color32::WHITE,
                            );
                        }
                        ui.painter().rect_stroke(
                            selection,
                            0.0,
                            Stroke::new(2.0, ACCENT),
                            egui::StrokeKind::Inside,
                        );
                        for index in self.selected_cells() {
                            let cell_rect = Rect::from_min_size(
                                rect.min
                                    + Vec2::new(
                                        (index % COLS) as f32 * cell,
                                        (index / COLS) as f32 * cell,
                                    ),
                                Vec2::splat(cell),
                            );
                            ui.painter().rect_stroke(
                                cell_rect,
                                0.0,
                                Stroke::new(2.0, ACCENT),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if response.clicked()
                            && let Some(pos) = response.interact_pointer_pos()
                        {
                            let relative = pos - rect.min;
                            let index =
                                (relative.y / cell) as usize * COLS + (relative.x / cell) as usize;
                            if index < self.doc.tiles.len() {
                                response.request_focus();
                                let modifiers = ui.ctx().input(|i| i.modifiers);
                                self.select_cell(index, modifiers.command, modifiers.shift);
                            }
                        }
                        let hovered = ui
                            .ctx()
                            .input(|i| i.pointer.latest_pos())
                            .filter(|p| rect.contains(*p))
                            .map(|pos| {
                                let relative = pos - rect.min;
                                (relative.y / cell) as usize * COLS + (relative.x / cell) as usize
                            })
                            .filter(|i| *i < self.doc.tiles.len());
                        if let Some(from) = response.dnd_release_payload::<editor::LayerDrag>()
                            && let Some(target) = hovered
                        {
                            self.move_layer(*from, target, self.doc.tiles[target].len());
                        }
                        if response.drag_started()
                            && egui::DragAndDrop::payload::<editor::LayerDrag>(ui.ctx()).is_none()
                        {
                            let origin = ui
                                .ctx()
                                .input(|i| i.pointer.press_origin())
                                .filter(|p| rect.contains(*p))
                                .map(|p| {
                                    let p = p - rect.min;
                                    (p.y / cell) as usize * COLS + (p.x / cell) as usize
                                });
                            self.commit_draft();
                            self.selection_drag = origin;
                            if let Some(origin) = origin {
                                self.selection_anchor = origin;
                            }
                            self.selection_drag_base = if ui.ctx().input(|i| i.modifiers.command) {
                                self.selection.clone()
                            } else {
                                Default::default()
                            };
                        }
                        if response.dragged()
                            && let (Some(start), Some(end)) = (self.selection_drag, hovered)
                        {
                            self.selection = self
                                .rectangle(start, end)
                                .union(&self.selection_drag_base)
                                .copied()
                                .collect();
                            self.selected = end;
                            self.layer = 0;
                            self.sync();
                        }
                        if response.drag_stopped() {
                            self.selection_drag = None;
                        }
                        if let Some(pos) = response.hover_pos() {
                            let relative = pos - rect.min;
                            let index =
                                (relative.y / cell) as usize * COLS + (relative.x / cell) as usize;
                            response.on_hover_text(crate::tf!(
                                "Иконка #{index}",
                                "Icon #{index}",
                                "图标#{index}"
                            ));
                        }
                    });
                muted(
                    ui,
                    crate::tf!(
                        "Иконка #{}  ·  Чёткие пиксели без сглаживания",
                        "Icon #{}  ·  Crisp pixels without smoothing",
                        "图标#{}  ·  清晰像素，无平滑处理",
                        self.selected
                    ),
                );
            });
        if self.refresh || self.last_preview.as_ref() != Some(&self.draft) {
            ctx.request_repaint();
        }
        self.draw_about(&ctx);
        self.draw_library(&ctx);
        self.draw_color_replacement(&ctx);
        self.draw_recovery(&ctx);
        self.autosave_tick(&ctx);
        self.capture_if_requested(&ctx);
    }

    fn draw_about(&mut self, ctx: &egui::Context) {
        if !self.about_open {
            return;
        }
        let mut open = true;
        let mut close = false;
        egui::Window::new(crate::i18n::tr("О программе"))
            .id(egui::Id::new("about-program"))
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.heading("IconSet Wizard 2");
                    let version = env!("CARGO_PKG_VERSION");
                    let version = version.strip_suffix(".0").unwrap_or(version);
                    muted(ui, format!("{} {version}", crate::i18n::tr("Версия")));
                    ui.add_space(8.0);
                    ui.label(format!("{}: Phoenix KageDesu", crate::i18n::tr("Автор")));
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        let width = 3.0 * 56.0 + 2.0 * ui.spacing().item_spacing.x;
                        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
                        for (symbol, url) in AUTHOR_LINKS {
                            social_button(ui, symbol, url);
                        }
                    });
                    ui.add_space(18.0);
                    close = ui.button(crate::i18n::tr("Закрыть")).clicked();
                    ui.add_space(8.0);
                });
            });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        self.about_open = open && !close;
    }

    fn draw_library(&mut self, ctx: &egui::Context) {
        if !self.library_open {
            return;
        }
        let mut open = self.library_open;
        let mut chosen = None;
        let mut favorite_change = None;
        egui::Window::new(crate::i18n::tr("Библиотека иконок"))
            .open(&mut open)
            .default_size([780.0, 520.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button(crate::i18n::tr("Выбрать папку пакетов"))
                        .clicked()
                        && let Some(folder) = rfd::FileDialog::new()
                            .set_directory(&self.library_folder)
                            .pick_folder()
                    {
                        self.library_folder = folder;
                        self.load_library();
                        self.persist_preferences();
                    }
                    if ui.button(crate::i18n::tr("Обновить")).clicked() {
                        self.load_library();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(crate::i18n::tr("Поиск:"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.library_query)
                            .hint_text(crate::i18n::tr("Пакет, описание или номер иконки"))
                            .desired_width(300.0),
                    );
                    ui.checkbox(
                        &mut self.library_favorites_only,
                        crate::i18n::tr("Только избранное"),
                    );
                    ui.checkbox(&mut self.library_search_all, crate::i18n::tr("Все пакеты"));
                });
                muted(
                    ui,
                    crate::i18n::tr("Звёздочка на миниатюре — добавить или убрать из избранного"),
                );
                muted(ui, self.library_folder.display().to_string());
                if !self.library_errors.is_empty() {
                    ui.collapsing(
                        crate::tf!(
                            "Ошибки загрузки: {}",
                            "Loading errors: {}",
                            "加载错误：{}",
                            self.library_errors.len()
                        ),
                        |ui| {
                            for error in &self.library_errors {
                                ui.label(error);
                            }
                        },
                    );
                }
                if self.packages.is_empty() {
                    ui.label(crate::i18n::tr(
                        "В выбранной папке нет доступных пакетов .iwpk.",
                    ));
                    return;
                }
                if !self.library_query.trim().is_empty() || self.library_favorites_only {
                    self.draw_library_results(ui, ctx, &mut chosen, &mut favorite_change);
                    return;
                }
                divider(ui);
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(175.0);
                        egui::ScrollArea::vertical()
                            .id_salt("library-categories")
                            .max_height(370.0)
                            .show(ui, |ui| {
                                ui.set_width(175.0);
                                for (index, package) in self.packages.iter().enumerate() {
                                    if ui
                                        .selectable_label(
                                            self.library_category == index,
                                            &package.name,
                                        )
                                        .on_hover_text(package.path.display().to_string())
                                        .clicked()
                                    {
                                        self.library_category = index;
                                        self.library_textures.clear();
                                    }
                                }
                            });
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        let package = &self.packages[self.library_category];
                        ui.heading(&package.name);
                        ui.label(&package.info);
                        muted(
                            ui,
                            crate::tf!(
                                "{} иконок · щёлкните для добавления в выбранную ячейку",
                                "{} icons · click to add to the selected cell",
                                "{}个图标 · 单击以添加到所选单元格",
                                package.items.len()
                            ),
                        );
                        if self.library_textures.is_empty() {
                            self.library_textures = package
                                .items
                                .iter()
                                .enumerate()
                                .map(|(index, image)| {
                                    ctx.load_texture(
                                        format!("library-{}-{index}", self.library_category),
                                        egui::ColorImage::from_rgba_unmultiplied(
                                            [32, 32],
                                            image.as_raw(),
                                        ),
                                        egui::TextureOptions::NEAREST,
                                    )
                                })
                                .collect();
                        }
                        let columns = ((ui.available_width() / 56.0) as usize).clamp(1, 12);
                        egui::ScrollArea::vertical()
                            .id_salt("library-items")
                            .max_height(340.0)
                            .show(ui, |ui| {
                                egui::Grid::new("library-grid").spacing([6.0, 6.0]).show(
                                    ui,
                                    |ui| {
                                        for (index, texture) in
                                            self.library_textures.iter().enumerate()
                                        {
                                            let (rect, response) = ui.allocate_exact_size(
                                                Vec2::splat(48.0),
                                                Sense::click(),
                                            );
                                            checker(ui, rect, 6.0);
                                            ui.painter().image(
                                                texture.id(),
                                                rect,
                                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                                                Color32::WHITE,
                                            );
                                            hover_outline(ui, rect, &response);
                                            let key =
                                                self.favorite_key(self.library_category, index);
                                            let star_rect = Rect::from_min_size(
                                                rect.right_top() - Vec2::new(16.0, 0.0),
                                                Vec2::splat(16.0),
                                            );
                                            let starred = self.preferences.favorites.contains(&key);
                                            ui.painter().rect_filled(
                                                star_rect,
                                                3.0,
                                                Color32::from_black_alpha(180),
                                            );
                                            ui.painter().text(
                                                star_rect.center(),
                                                Align2::CENTER_CENTER,
                                                if starred { "★" } else { "☆" },
                                                FontId::proportional(15.0),
                                                if starred { Color32::YELLOW } else { TEXT },
                                            );
                                            let star_response = ui
                                                .interact(
                                                    star_rect,
                                                    egui::Id::new((
                                                        "library-star",
                                                        self.library_category,
                                                        index,
                                                    )),
                                                    Sense::click(),
                                                )
                                                .on_hover_text(crate::i18n::tr("Избранное"));
                                            hover_outline(ui, star_rect, &star_response);
                                            if star_response.clicked() {
                                                favorite_change =
                                                    Some((self.library_category, index));
                                            }
                                            let on_star = ui.ctx().input(|i| {
                                                i.pointer
                                                    .latest_pos()
                                                    .is_some_and(|p| star_rect.contains(p))
                                            });
                                            if response
                                                .on_hover_text(format!(
                                                    "{} #{}",
                                                    package.name,
                                                    index + 1
                                                ))
                                                .clicked()
                                                && !on_star
                                            {
                                                chosen = Some((
                                                    format!("{} #{}", package.name, index + 1),
                                                    package.items[index].clone(),
                                                ));
                                            }
                                            if (index + 1).is_multiple_of(columns) {
                                                ui.end_row();
                                            }
                                        }
                                    },
                                );
                            });
                    });
                });
            });
        self.library_open = open;
        if let Some((category, index)) = favorite_change {
            self.toggle_favorite(category, index);
        }
        if let Some((name, image)) = chosen
            && self.insert_image(name, image, false)
        {
            self.status = crate::i18n::tr("Иконка из библиотеки добавлена новым слоем.").into();
            self.library_open = false;
        }
    }

    fn draw_library_results(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        chosen: &mut Option<(String, image::RgbaImage)>,
        favorite: &mut Option<(usize, usize)>,
    ) {
        let matches = (0..self.packages.len())
            .filter(|c| {
                self.library_search_all
                    || self.library_favorites_only
                    || *c == self.library_category
            })
            .flat_map(|c| (0..self.packages[c].items.len()).map(move |i| (c, i)))
            .filter(|(c, i)| self.library_matches(*c, *i))
            .collect::<Vec<_>>();
        ui.label(crate::tf!(
            "Найдено: {}",
            "Found: {}",
            "找到：{}",
            matches.len()
        ));
        let columns = ((ui.available_width() / 64.0) as usize).clamp(1, 12);
        egui::ScrollArea::vertical()
            .id_salt("library-filtered")
            .max_height(360.0)
            .show(ui, |ui| {
                egui::Grid::new("library-filtered-grid")
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        for (number, (category, index)) in matches.iter().copied().enumerate() {
                            let texture = self
                                .library_texture_cache
                                .entry((category, index))
                                .or_insert_with(|| {
                                    ctx.load_texture(
                                        format!("filtered-{category}-{index}"),
                                        egui::ColorImage::from_rgba_unmultiplied(
                                            [32, 32],
                                            self.packages[category].items[index].as_raw(),
                                        ),
                                        egui::TextureOptions::NEAREST,
                                    )
                                })
                                .id();
                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::splat(48.0), Sense::click());
                            checker(ui, rect, 6.0);
                            ui.painter().image(
                                texture,
                                rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                                Color32::WHITE,
                            );
                            hover_outline(ui, rect, &response);
                            let star_rect = Rect::from_min_size(
                                rect.right_top() - Vec2::new(16.0, 0.0),
                                Vec2::splat(16.0),
                            );
                            let starred = self
                                .preferences
                                .favorites
                                .contains(&self.favorite_key(category, index));
                            ui.painter().rect_filled(
                                star_rect,
                                3.0,
                                Color32::from_black_alpha(180),
                            );
                            ui.painter().text(
                                star_rect.center(),
                                Align2::CENTER_CENTER,
                                if starred { "★" } else { "☆" },
                                FontId::proportional(15.0),
                                if starred { Color32::YELLOW } else { TEXT },
                            );
                            let star_response = ui
                                .interact(
                                    star_rect,
                                    egui::Id::new(("filtered-star", category, index)),
                                    Sense::click(),
                                )
                                .on_hover_text(crate::i18n::tr("Избранное"));
                            hover_outline(ui, star_rect, &star_response);
                            if star_response.clicked() {
                                *favorite = Some((category, index));
                            }
                            let on_star = ctx.input(|i| {
                                i.pointer
                                    .latest_pos()
                                    .is_some_and(|p| star_rect.contains(p))
                            });
                            if response
                                .on_hover_text(format!(
                                    "{} #{}",
                                    self.packages[category].name,
                                    index + 1
                                ))
                                .clicked()
                                && !on_star
                            {
                                *chosen = Some((
                                    format!("{} #{}", self.packages[category].name, index + 1),
                                    self.packages[category].items[index].clone(),
                                ));
                            }
                            if (number + 1).is_multiple_of(columns) {
                                ui.end_row();
                            }
                        }
                    });
            });
    }

    fn draw_color_replacement(&mut self, ctx: &egui::Context) {
        if !self.color_open {
            return;
        }
        let mut open = true;
        egui::Window::new(crate::i18n::tr("Замена цвета")).open(&mut open).resizable(false).show(ctx,|ui| {
            ui.set_max_width(420.0);
            ui.horizontal(|ui| {
                ui.label(crate::i18n::tr("Из:"));ui.color_edit_button_srgb(&mut self.color_from);
                ui.label(crate::i18n::tr("В:"));ui.color_edit_button_srgb(&mut self.color_to);
            });
            let palette=self.doc.tiles[self.selected].get(self.layer).map(|layer| {
                layer.pixels.rgba.as_chunks::<4>().0.iter().filter(|p|p[3]>0).map(|p|[p[0],p[1],p[2]]).collect::<std::collections::BTreeSet<_>>()
            }).unwrap_or_default();
            ui.label(crate::i18n::tr("Выберите исходный цвет из палитры слоя:"));
            ui.horizontal_wrapped(|ui| {
                for color in palette.iter().take(128) {
                    let (rect,response)=ui.allocate_exact_size(Vec2::splat(18.0),Sense::click());
                    ui.painter().rect_filled(rect,2.0,Color32::from_rgb(color[0],color[1],color[2]));
                    hover_outline(ui, rect, &response);
                    if response.on_hover_text(format!("RGB {}, {}, {}",color[0],color[1],color[2])).clicked() {self.color_from = *color;}
                }
            });
            ui.add(egui::Slider::new(&mut self.color_tolerance,0..=64).text(crate::i18n::tr("Допуск")));
            ui.checkbox(&mut self.color_all_layers,crate::i18n::tr("Все слои выделенных ячеек"));
            muted(ui,crate::i18n::tr("Без этой опции изменяется только текущий слой. Альфа сохраняется; цвет выбирается до Hue."));
            let mut preview=self.doc.clone();
            if let Some(layer)=preview.tiles[self.selected].get_mut(self.layer) {*layer=self.draft.clone();}
            for (index,layer) in preview.tiles[self.selected].iter_mut().enumerate() {
                if self.color_all_layers || index==self.layer {Self::recolor_layer(layer,self.color_from,self.color_to,self.color_tolerance);}
            }
            let image=preview.render_tile(self.selected);
            let texture=ctx.load_texture("color-preview",egui::ColorImage::from_rgba_unmultiplied([32,32],image.as_raw()),egui::TextureOptions::NEAREST);
            ui.label(crate::i18n::tr("Предпросмотр результата:"));ui.image((texture.id(),Vec2::splat(128.0)));
            if ui.button(crate::i18n::tr("Заменить цвет")).clicked() {self.replace_color();self.color_open=false;}
        });
        if !open {
            self.color_open = false;
        }
    }

    fn draw_recovery(&mut self, ctx: &egui::Context) {
        if !self.recovery_open {
            return;
        }
        let mut action = None;
        egui::Modal::new(egui::Id::new("recover-project")).show(ctx,|ui| {
            ui.set_max_width(520.0);ui.heading(crate::i18n::tr("Найдены автосохранения"));
            ui.label(crate::i18n::tr("Можно восстановить проект со слоями и неприменёнными параметрами. Исходные PNG не изменяются."));
            egui::ScrollArea::vertical().max_height(250.0).show(ui,|ui| {
                for (index,path) in self.recovery_candidates.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} · {} ({:.1} MB)",index+1,storage::recovery_description(path),std::fs::metadata(path).map(|m|m.len()).unwrap_or(0) as f64/1048576.0)).on_hover_text(path.display().to_string());
                        if ui.button(crate::i18n::tr("Восстановить")).clicked() {action=Some((path.clone(),true));}
                        if ui.button(crate::i18n::tr("Удалить копию")).clicked() {action=Some((path.clone(),false));}
                    });
                }
            });
            if ui.button(crate::i18n::tr("Позже")).clicked() {self.recovery_open=false;}
        });
        if let Some((path, restore)) = action {
            if restore {
                if self.may_replace()
                    && let Err(e) = self.restore_recovery(&path)
                {
                    self.status = crate::tf!(
                        "Ошибка восстановления: {e}",
                        "Recovery error: {e}",
                        "恢复错误：{e}"
                    );
                }
            } else {
                storage::remove_recovery(&path);
                self.recovery_candidates.retain(|p| p != &path);
                self.recovery_open = !self.recovery_candidates.is_empty();
            }
        }
    }

    fn capture_if_requested(&mut self, ctx: &egui::Context) {
        let Some(path) = std::env::var_os("ISW_SCREENSHOT_PATH") else {
            return;
        };
        self.screenshot_frame += 1;
        if self.screenshot_frame == 12 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        let image = ctx.input(|i| {
            i.events.iter().find_map(|event| {
                if let egui::Event::Screenshot { image, .. } = event {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });
        if let Some(image) = image {
            let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
            if let Some(bitmap) =
                image::RgbaImage::from_raw(image.width() as u32, image.height() as u32, bytes)
            {
                let _ = bitmap.save(PathBuf::from(path));
            }
            self.allow_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if self.screenshot_frame < 14 {
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod social_link_tests {
    use super::*;

    #[test]
    fn clicking_each_social_icon_emits_the_browser_command() {
        for (symbol, url) in AUTHOR_LINKS {
            let ctx = egui::Context::default();
            configure(&ctx);
            let mut button_rect = Rect::NOTHING;
            let mut draw = |events| {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 100.0))),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        button_rect = social_button(ui, symbol, url).rect;
                    },
                );
                output.textures_delta.clear();
                (output, button_rect)
            };
            let (output, rect) = draw(vec![]);
            assert!(output.platform_output.commands.is_empty());
            // URLs stay out of rendered text; each icon opens exactly one URL.
            for shape in &output.shapes {
                if let egui::Shape::Text(text) = &shape.shape {
                    assert!(!text.galley.text().contains("https://"));
                }
            }
            let pos = rect.center();
            for pressed in [true, false] {
                let (output, _) = draw(vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ]);
                let opened = output
                    .platform_output
                    .commands
                    .iter()
                    .filter_map(|command| {
                        if let egui::OutputCommand::OpenUrl(open) = command {
                            Some(open)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if pressed {
                    assert!(opened.is_empty());
                } else {
                    assert_eq!(opened.len(), 1);
                    assert_eq!(opened[0].url, url);
                    assert!(opened[0].new_tab);
                }
            }
        }
    }
}

#[cfg(test)]
mod hover_tests {
    use super::*;

    #[test]
    fn rendered_buttons_highlight_on_hover_and_press_but_disabled_buttons_do_not() {
        for kind in 0..4 {
            for enabled in [true, false] {
                if kind == 3 && !enabled {
                    continue;
                }
                let ctx = egui::Context::default();
                configure(&ctx);
                ctx.all_styles_mut(|style| style.animation_time = 0.0);
                let mut rect = Rect::NOTHING;
                let mut draw = |events| {
                    let mut output = ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(Rect::from_min_size(
                                Pos2::ZERO,
                                Vec2::new(400.0, 120.0),
                            )),
                            events,
                            ..Default::default()
                        },
                        |ui| {
                            rect = match kind {
                                0 => ui.add_enabled(enabled, egui::Button::new("Action")).rect,
                                1 | 2 => {
                                    button(ui, Icon::Export, "Action", enabled, kind == 2).rect
                                }
                                _ => social_button(ui, SocialIcon::Website, AUTHOR_LINKS[0].1).rect,
                            };
                        },
                    );
                    output.textures_delta.clear();
                    let fill = output
                        .shapes
                        .iter()
                        .find_map(|shape| {
                            if let egui::Shape::Rect(paint) = &shape.shape {
                                ((paint.rect.center() - rect.center()).length() < 1.0)
                                    .then_some(paint.fill)
                            } else {
                                None
                            }
                        })
                        .expect("button background is rendered");
                    (fill, rect)
                };
                let (normal, rect) = draw(vec![egui::Event::PointerMoved(Pos2::new(390.0, 110.0))]);
                let pos = rect.center();
                draw(vec![egui::Event::PointerMoved(pos)]);
                let (hovered, _) = draw(vec![egui::Event::PointerMoved(pos)]);
                if enabled {
                    assert_ne!(
                        normal, hovered,
                        "kind {kind}: hover must change the visible fill"
                    );
                    let (pressed, _) = draw(vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    }]);
                    assert_ne!(
                        hovered, pressed,
                        "kind {kind}: press must differ from hover"
                    );
                } else {
                    assert_eq!(normal, hovered, "disabled buttons must keep their fill");
                }
            }
        }
    }
}
