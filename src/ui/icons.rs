use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontFamily, FontId, Rect, Response, Sense, Stroke,
    TextureHandle, TextureOptions, Ui, Vec2,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IconKey {
    Logo,
    Open,
    Save,
    New,
    Print,
    Preview,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    InsertTask,
    InsertResource,
    InsertProject,
    TaskDetails,
    ProjectDetails,
    ResourceDetails,
    DeleteLink,
    InsertLink,
    Indent,
    Outdent,
    ScrollToTask,
    Help,
    Question,
    Locale,
    Histogram,
    Charts,
    TaskUsage,
    ResourceUsage,
    NoSubWindow,
    Info,
    Note,
    Constraint,
    Wbs,
    Plus,
    Minus,
    SaveAs,
    CloseProject,
    PDF,
    Calendar,
    ProjectsDialog,
    SaveBaseline,
    ClearBaseline,
    Update,
    Paste,
    Copy,
    Cut,
    Delete,
    Find,
}

pub struct ProjectLibreIcons {
    textures: HashMap<IconKey, TextureHandle>,
}

impl ProjectLibreIcons {
    pub fn load(ctx: &egui::Context) -> Self {
        let mut textures = HashMap::new();
        for spec in icon_specs() {
            if let Some(texture) = load_texture(ctx, spec.key, &spec.rel_path) {
                textures.insert(spec.key, texture);
            }
        }
        Self { textures }
    }

    pub fn texture(&self, key: IconKey) -> Option<&TextureHandle> {
        self.textures.get(&key)
    }

    pub fn logo(&self) -> Option<&TextureHandle> {
        self.texture(IconKey::Logo)
    }

    pub fn ribbon_button(&self, ui: &mut Ui, key: IconKey, label: &str, tooltip: &str) -> Response {
        let Some(texture) = self.texture(key) else {
            return ui
                .add_enabled(false, egui::Button::new(label))
                .on_hover_text(tooltip);
        };

        let size = vec2(66.0, 60.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let fill = if response.is_pointer_button_down_on() {
                Color32::from_rgb(222, 232, 245)
            } else if response.hovered() {
                Color32::from_rgb(244, 244, 244)
            } else {
                Color32::from_rgb(239, 239, 239)
            };
            painter.rect_filled(rect, 4.0, fill);
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(1.0, Color32::from_rgb(188, 188, 188)),
                eframe::egui::StrokeKind::Outside,
            );

            let icon_rect =
                Rect::from_center_size(pos2(rect.center().x, rect.top() + 21.0), vec2(24.0, 24.0));
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            painter.text(
                pos2(rect.center().x, rect.bottom() - 13.0),
                Align2::CENTER_CENTER,
                label,
                FontId::new(11.0, FontFamily::Proportional),
                Color32::from_rgb(40, 40, 40),
            );
        }

        response.on_hover_text(tooltip)
    }

    pub fn icon_button(&self, ui: &mut Ui, key: IconKey, tooltip: &str, size: Vec2) -> Response {
        let Some(texture) = self.texture(key) else {
            return ui
                .add_enabled(false, egui::Button::new("?"))
                .on_hover_text(tooltip);
        };

        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let fill = if response.is_pointer_button_down_on() {
                Color32::from_rgb(222, 232, 245)
            } else if response.hovered() {
                Color32::from_rgb(248, 248, 248)
            } else {
                Color32::from_rgb(239, 239, 239)
            };
            painter.rect_filled(rect, 3.0, fill);
            painter.rect_stroke(
                rect,
                3.0,
                Stroke::new(1.0, Color32::from_rgb(190, 190, 190)),
                eframe::egui::StrokeKind::Outside,
            );
            let side = size.x.min(size.y) - 8.0;
            let icon_rect = Rect::from_center_size(rect.center(), vec2(side, side));
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        response.on_hover_text(tooltip)
    }

    pub fn row_button(
        &self,
        ui: &mut Ui,
        key: IconKey,
        label: &str,
        tooltip: &str,
        width: f32,
    ) -> Response {
        let Some(texture) = self.texture(key) else {
            return ui
                .add_enabled(false, egui::Button::new(label))
                .on_hover_text(tooltip);
        };

        let size = vec2(width, 19.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let fill = if response.is_pointer_button_down_on() {
                Color32::from_rgb(224, 233, 245)
            } else if response.hovered() {
                Color32::from_rgb(247, 247, 247)
            } else {
                Color32::from_rgb(242, 242, 242)
            };
            painter.rect_filled(rect, 2.0, fill);
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::from_rgb(203, 203, 203)),
                eframe::egui::StrokeKind::Outside,
            );

            let icon_rect = Rect::from_min_size(
                pos2(rect.left() + 3.0, rect.center().y - 7.0),
                vec2(14.0, 14.0),
            );
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            painter.text(
                pos2(rect.left() + 20.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::new(11.0, FontFamily::Proportional),
                Color32::from_rgb(42, 42, 42),
            );
        }

        response.on_hover_text(tooltip)
    }

    pub fn text_button(&self, ui: &mut Ui, label: &str, tooltip: &str, width: f32) -> Response {
        let size = vec2(width, 19.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let fill = if response.is_pointer_button_down_on() {
                Color32::from_rgb(224, 233, 245)
            } else if response.hovered() {
                Color32::from_rgb(247, 247, 247)
            } else {
                Color32::from_rgb(242, 242, 242)
            };
            painter.rect_filled(rect, 2.0, fill);
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::from_rgb(203, 203, 203)),
                eframe::egui::StrokeKind::Outside,
            );
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::new(11.0, FontFamily::Proportional),
                Color32::from_rgb(42, 42, 42),
            );
        }

        response.on_hover_text(tooltip)
    }
}

struct IconSpec {
    key: IconKey,
    rel_path: &'static str,
}

fn icon_specs() -> Vec<IconSpec> {
    vec![
        IconSpec {
            key: IconKey::Logo,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/projectlibre-logo.png",
        },
        IconSpec {
            key: IconKey::Open,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/document-open.png",
        },
        IconSpec {
            key: IconKey::Save,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/document-save.png",
        },
        IconSpec {
            key: IconKey::New,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/document-new.png",
        },
        IconSpec {
            key: IconKey::Print,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/document-print.png",
        },
        IconSpec {
            key: IconKey::Preview,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/document-print-preview.png",
        },
        IconSpec {
            key: IconKey::Undo,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/edit-undo.png",
        },
        IconSpec {
            key: IconKey::Redo,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/edit-redo.png",
        },
        IconSpec {
            key: IconKey::ZoomIn,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/zoom-in.png",
        },
        IconSpec {
            key: IconKey::ZoomOut,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/zoom-out.png",
        },
        IconSpec {
            key: IconKey::InsertTask,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/insert-task.png",
        },
        IconSpec {
            key: IconKey::InsertResource,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/insert-resource.png",
        },
        IconSpec {
            key: IconKey::InsertProject,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/insert-project.png",
        },
        IconSpec {
            key: IconKey::TaskDetails,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/task-details.png",
        },
        IconSpec {
            key: IconKey::ProjectDetails,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/project-details.png",
        },
        IconSpec {
            key: IconKey::ResourceDetails,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/resource-details.png",
        },
        IconSpec {
            key: IconKey::DeleteLink,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/ribbon/delete-link.png",
        },
        IconSpec {
            key: IconKey::InsertLink,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/ribbon/insert-link-2.png",
        },
        IconSpec {
            key: IconKey::Indent,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/ribbon/format-indent-more-5.png",
        },
        IconSpec {
            key: IconKey::Outdent,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/ribbon/format-indent-less-5.png",
        },
        IconSpec {
            key: IconKey::ScrollToTask,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/big/scrollToTask.gif",
        },
        IconSpec {
            key: IconKey::Help,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/ribbon/help-hint.png",
        },
        IconSpec {
            key: IconKey::Question,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/question.png",
        },
        IconSpec {
            key: IconKey::Locale,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/globe24.png",
        },
        IconSpec {
            key: IconKey::Histogram,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/histogram.png",
        },
        IconSpec {
            key: IconKey::Charts,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/chart.png",
        },
        IconSpec {
            key: IconKey::TaskUsage,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/taskUsage.png",
        },
        IconSpec {
            key: IconKey::ResourceUsage,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/resourceUsage.png",
        },
        IconSpec {
            key: IconKey::NoSubWindow,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/noSubWindow.png",
        },
        IconSpec {
            key: IconKey::Info,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/information.gif",
        },
        IconSpec {
            key: IconKey::Note,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/note.gif",
        },
        IconSpec {
            key: IconKey::Constraint,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/constraint.gif",
        },
        IconSpec {
            key: IconKey::Wbs,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/WBS.gif",
        },
        IconSpec {
            key: IconKey::Plus,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/plus.png",
        },
        IconSpec {
            key: IconKey::Minus,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/moins.png",
        },
        IconSpec {
            key: IconKey::SaveAs,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/document-save-as.png",
        },
        IconSpec {
            key: IconKey::CloseProject,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/close-project.png",
        },
        IconSpec {
            key: IconKey::PDF,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/pdf.png",
        },
        IconSpec {
            key: IconKey::Calendar,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/calendar24.png",
        },
        IconSpec {
            key: IconKey::ProjectsDialog,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/projects24.png",
        },
        IconSpec {
            key: IconKey::SaveBaseline,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/disk.png",
        },
        IconSpec {
            key: IconKey::ClearBaseline,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/edit-clear.png",
        },
        IconSpec {
            key: IconKey::Update,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/reload.png",
        },
        IconSpec {
            key: IconKey::Paste,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/edit-paste.png",
        },
        IconSpec {
            key: IconKey::Copy,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/edit-copy.png",
        },
        IconSpec {
            key: IconKey::Cut,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/edit-cut.png",
        },
        IconSpec {
            key: IconKey::Delete,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/edit-delete.png",
        },
        IconSpec {
            key: IconKey::Find,
            rel_path: "original files of projectlibre/RustyProject-main/projectlibre_ui/src/com/projectlibre1/pm/graphic/images/edit-find-7.png",
        },
    ]
}

fn load_texture(ctx: &egui::Context, key: IconKey, rel_path: &str) -> Option<TextureHandle> {
    let path = resolve_icon_path(rel_path)?;
    let bytes = std::fs::read(&path).ok()?;
    let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    Some(ctx.load_texture(
        format!("projectlibre::{key:?}"),
        color_image,
        TextureOptions::LINEAR,
    ))
}

fn resolve_icon_path(rel_path: &str) -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidate = root.join(rel_path);
    candidate.exists().then_some(candidate)
}
