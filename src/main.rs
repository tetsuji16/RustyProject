mod history;
mod model;
mod mpp_import;
mod pod_export;
mod pod_import;
mod project_file;
mod schedule;
mod ui;

use chrono::{Duration, NaiveDate};
use eframe::egui::{
    self, pos2, vec2, Color32, FontData, FontDefinitions, FontFamily, Pos2, Rect, Stroke,
};
use history::UndoRedo;
use model::{EditCommand, ProjectSnapshot, TaskSnapshot};
use mpp_import::load_mpp;
use pod_export::{save_pod, save_xml};
use pod_import::{load_pod, load_xml};
use project_file::{load as load_project, save as save_project, ProjectDocument};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use ui::icons::{IconKey, ProjectLibreIcons};

const APP_NAME: &str = "ProjectLibre";
const VIEW_WIDTH: f32 = 2048.0;
const VIEW_HEIGHT: f32 = 1222.0;
const SAMPLE_MPP_PATH: &str = "sample data/Commercial construction project plan.mpp";
const SAMPLE_XML_PATH: &str = "sample data/Commercial construction project plan.xml";
const SAMPLE_POD_PATH: &str = "sample data/Commercial construction project plan.pod";

const ROW_H: f32 = 31.0;
const LEFT_TABLE_W: f32 = crate::ui::gantt_view::LEFT_TABLE_W;
const LEFT_TABLE_MIN_W: f32 = 540.0;
const LEFT_TABLE_MAX_W: f32 = 860.0;
const SPLITTER_W: f32 = 6.0;
const DAY_W: f32 = crate::ui::gantt_view::DAY_W;

fn main() -> eframe::Result<()> {
    let startup_path = std::env::args().nth(1).map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([VIEW_WIDTH, VIEW_HEIGHT])
            .with_title(APP_NAME),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(GanttApp::new(cc, startup_path.clone())))),
    )
}

struct GanttApp {
    snapshot: ProjectSnapshot,
    selected_task_id: usize,
    history: UndoRedo<ProjectDocument>,
    drag: Option<DragState>,
    collapsed_summaries: HashSet<usize>,
    day_width: f32,
    left_table_width: f32,
    project_path_input: String,
    status_message: String,
    active_tab: TopTab,
    icons: ProjectLibreIcons,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TopTab {
    File,
    Task,
    Resource,
    View,
}

impl GanttApp {
    fn new(cc: &eframe::CreationContext<'_>, startup_path: Option<PathBuf>) -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(240, 240, 240);
        visuals.widgets.noninteractive.bg_stroke =
            Stroke::new(1.0, Color32::from_rgb(185, 185, 185));
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(248, 248, 248);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(205, 205, 205));
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(252, 252, 252);
        visuals.widgets.active.bg_fill = Color32::from_rgb(225, 236, 248);
        cc.egui_ctx.set_visuals(visuals);
        let icons = ProjectLibreIcons::load(&cc.egui_ctx);
        install_japanese_fonts(&cc.egui_ctx);

        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = vec2(0.0, 0.0);
        style.spacing.window_margin = egui::Margin::same(0);
        cc.egui_ctx.set_style(style);

        let bundled_sample = bundled_sample_path();
        let snapshot = ProjectSnapshot::sample();
        let selected_task_id = snapshot.tasks.first().map(|task| task.number).unwrap_or(0);

        let mut app = Self {
            selected_task_id,
            history: UndoRedo::default(),
            drag: None,
            collapsed_summaries: HashSet::new(),
            day_width: DAY_W,
            left_table_width: LEFT_TABLE_W,
            project_path_input: bundled_sample
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "project.xml".to_string()),
            status_message: String::from("Ready"),
            active_tab: TopTab::File,
            icons,
            snapshot,
        };
        if let Some(path) = startup_path.or(bundled_sample) {
            app.load_project_from_path(path.to_string_lossy().as_ref());
        }
        app
    }
}

fn install_japanese_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let candidates = [
        r"C:\Windows\Fonts\NotoSansJP-VF.ttf",
        r"C:\Windows\Fonts\meiryo.ttc",
        r"C:\Windows\Fonts\msgothic.ttc",
    ];
    let mut loaded = false;

    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("japanese".to_string(), FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "japanese".to_string());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "japanese".to_string());
            loaded = true;
            break;
        }
    }

    if loaded {
        ctx.set_fonts(fonts);
    }
}

impl eframe::App for GanttApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));
        self.handle_shortcuts(ctx);
        self.draw_chrome(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(248, 248, 248)))
            .show(ctx, |ui| {
                let visible_rows = crate::ui::gantt_view::build_visible_rows(
                    &self.snapshot.tasks,
                    &self.collapsed_summaries,
                );
                let rect = ui.max_rect();
                let chart = crate::ui::gantt_view::TimelineGeometry::new(
                    rect,
                    &self.snapshot,
                    self.day_width,
                    self.left_table_width,
                );

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let content_size = vec2(
                            crate::ui::gantt_view::content_width(&chart, self.left_table_width),
                            crate::ui::gantt_view::content_height(visible_rows.len()),
                        );
                        let (content_rect, _) =
                            ui.allocate_exact_size(content_size, egui::Sense::hover());
                        let splitter_rect = Rect::from_min_max(
                            pos2(
                                content_rect.left() + self.left_table_width,
                                content_rect.top(),
                            ),
                            pos2(
                                content_rect.left() + self.left_table_width + SPLITTER_W,
                                content_rect.bottom(),
                            ),
                        );
                        let splitter_response = ui.interact(
                            splitter_rect,
                            ui.id().with("splitter"),
                            egui::Sense::drag(),
                        );
                        if splitter_response.dragged() {
                            self.left_table_width = (self.left_table_width
                                + splitter_response.drag_delta().x)
                                .clamp(LEFT_TABLE_MIN_W, LEFT_TABLE_MAX_W);
                        }
                        if splitter_response.hovered() || splitter_response.dragged() {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }

                        let painter = ui.painter_at(content_rect);
                        self.handle_pointer(
                            ctx,
                            content_rect,
                            self.left_table_width,
                            &chart,
                            &visible_rows,
                        );
                        let drag_preview = self.drag.as_ref().map(DragState::preview);

                        crate::ui::gantt_view::draw_workspace(
                            &painter,
                            content_rect,
                            &chart,
                            self.snapshot.status_date,
                            &self.snapshot.tasks,
                            &visible_rows,
                            self.selected_task_id,
                            &self.collapsed_summaries,
                            self.left_table_width,
                            &self.icons,
                            drag_preview.as_ref(),
                        );
                    });
            });
    }
}

impl GanttApp {
    fn draw_chrome(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("chrome")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(239, 239, 239))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(180, 180, 180))),
            )
            .show(ctx, |ui| {
                ui.set_min_height(164.0);
                ui.add_space(2.0);
                self.draw_top_bar(ui);
                ui.add_space(1.0);
                self.draw_tab_row(ui);
                self.draw_ribbon_row(ui);
            });
    }

    fn draw_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            self.draw_logo(ui);
            ui.add_space(6.0);

            self.draw_quick_access_bar(ui);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.draw_language_selector(ui);
                ui.add_space(8.0);
                self.draw_project_selector(ui);
            });
        });
    }

    fn draw_quick_access_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_rgb(227, 227, 227))
            .stroke(Stroke::new(1.0, Color32::from_rgb(174, 174, 174)))
            .corner_radius(egui::CornerRadius::same(14))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(2.0);
                    if self
                        .icons
                        .icon_button(
                            ui,
                            IconKey::TopBarSaveProject,
                            "Save project",
                            vec2(26.0, 26.0),
                        )
                        .clicked()
                    {
                        self.save_project_to_entry_or_dialog();
                    }
                    if self
                        .icons
                        .icon_button(ui, IconKey::TopBarUndo, "Undo", vec2(26.0, 26.0))
                        .clicked()
                    {
                        self.undo();
                    }
                    if self
                        .icons
                        .icon_button(ui, IconKey::TopBarRedo, "Redo", vec2(26.0, 26.0))
                        .clicked()
                    {
                        self.redo();
                    }
                    ui.add_space(2.0);
                });
            });
    }

    fn draw_project_selector(&mut self, ui: &mut egui::Ui) {
        let display_name = self.project_display_name();
        let mut selected = 0usize;
        let response = egui::ComboBox::from_id_salt("project_selector")
            .width(184.0)
            .selected_text(display_name.clone())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, 0, display_name.clone());
            })
            .response;
        let _ = response.on_hover_text(self.project_path_input.clone());
    }

    fn project_display_name(&self) -> String {
        let path = self.project_path_input.trim();
        if path.is_empty() {
            return "project.json".to_string();
        }
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or_else(|| path.to_string())
    }

    fn window_title(&self) -> String {
        let path = self.project_display_name();
        if path.is_empty() {
            APP_NAME.to_string()
        } else {
            format!("{APP_NAME} - {path} *")
        }
    }

    fn draw_project_views(&mut self, ui: &mut egui::Ui) {
        let mut style = ui.style().as_ref().clone();
        style.spacing.item_spacing = vec2(2.0, 0.0);
        ui.set_style(style);
        ui.horizontal(|ui| {
            let _ = self
                .icons
                .icon_button(ui, IconKey::Histogram, "Histogram", vec2(18.0, 18.0));
            let _ = self
                .icons
                .icon_button(ui, IconKey::Charts, "Charts", vec2(18.0, 18.0));
            let _ = self
                .icons
                .icon_button(ui, IconKey::TaskUsage, "Task Usage", vec2(18.0, 18.0));
            let _ = self.icons.icon_button(
                ui,
                IconKey::ResourceUsage,
                "Resource Usage",
                vec2(18.0, 18.0),
            );
            let _ =
                self.icons
                    .icon_button(ui, IconKey::NoSubWindow, "No sub window", vec2(18.0, 18.0));
        });
    }

    fn draw_language_selector(&mut self, ui: &mut egui::Ui) {
        let _ = self
            .icons
            .icon_button(ui, IconKey::Locale, "Locale", vec2(26.0, 26.0));
    }

    fn draw_help_button(&mut self, ui: &mut egui::Ui) {
        let _ = self
            .icons
            .icon_button(ui, IconKey::Help, "Help", vec2(26.0, 26.0));
    }

    fn draw_clipboard_band(&mut self, ui: &mut egui::Ui) {
        self.draw_band(ui, "クリップボード", 104.0, |ui, this| {
            ui.horizontal_top(|ui| {
                let _ = this
                    .icons
                    .large_ribbon_button(ui, IconKey::Paste, "貼り付け", "Paste");
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    let _ = this
                        .icons
                        .row_button(ui, IconKey::Copy, "コピー", "Copy", 92.0);
                    let _ = this
                        .icons
                        .row_button(ui, IconKey::Cut, "切り取り", "Cut", 92.0);
                });
            });
        });
    }

    fn draw_tab_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            for tab in [
                (TopTab::File, "ファイル"),
                (TopTab::Task, "タスク"),
                (TopTab::Resource, "リソース"),
                (TopTab::View, "ビュー"),
            ] {
                let selected = self.active_tab == tab.0;
                let mut button = egui::Button::new(tab.1).min_size(vec2(50.0, 22.0));
                if selected {
                    button = button.fill(Color32::from_rgb(241, 241, 241));
                }
                if ui.add(button).clicked() {
                    self.active_tab = tab.0;
                }
            }
            let end_group_width = 18.0 * 5.0 + 2.0 * 4.0 + 8.0 + 26.0;
            let spacer = (ui.available_width() - end_group_width).max(0.0);
            ui.add_space(spacer);
            ui.horizontal(|ui| {
                self.draw_project_views(ui);
                ui.add_space(8.0);
                self.draw_help_button(ui);
            });
        });
    }

    fn draw_ribbon_row(&mut self, ui: &mut egui::Ui) {
        match self.active_tab {
            TopTab::File => self.draw_file_tab(ui),
            TopTab::Task => self.draw_task_tab(ui),
            TopTab::Resource => self.draw_resource_tab(ui),
            TopTab::View => self.draw_view_tab(ui),
        }
    }

    fn draw_file_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            self.draw_band(ui, "ファイル", 214.0, |ui, this| {
                ui.horizontal(|ui| {
                    if this
                        .icons
                        .ribbon_button(ui, IconKey::Save, "保存", "Save project")
                        .clicked()
                    {
                        this.save_project_to_entry_or_dialog();
                    }
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        if this
                            .icons
                            .row_button(ui, IconKey::Open, "開く", "Open project", 112.0)
                            .clicked()
                        {
                            this.load_project_from_dialog();
                        }
                        let _ =
                            this.icons
                                .row_button(ui, IconKey::New, "新規", "New project", 112.0);
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::SaveAs,
                            "名前を付けて保存",
                            "Save as",
                            112.0,
                        );
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::CloseProject,
                            "閉じる",
                            "Close project",
                            92.0,
                        );
                    });
                });
            });
            self.draw_band(ui, "印刷", 118.0, |ui, this| {
                ui.vertical(|ui| {
                    ui.add_space(2.0);
                    let _ = this
                        .icons
                        .row_button(ui, IconKey::Print, "印刷", "Print", 96.0);
                    let _ = this.icons.row_button(
                        ui,
                        IconKey::Preview,
                        "プレビュー",
                        "Print preview",
                        96.0,
                    );
                    let _ = this
                        .icons
                        .row_button(ui, IconKey::PDF, "PDF", "Export PDF", 96.0);
                });
            });
            self.draw_project_band(ui);
        });
    }

    fn draw_project_band(&mut self, ui: &mut egui::Ui) {
        self.draw_band(ui, "プロジェクト", 360.0, |ui, this| {
            ui.horizontal_top(|ui| {
                let _ = this.icons.large_ribbon_button(
                    ui,
                    IconKey::Projects,
                    "プロジェクト",
                    "Project",
                );
                ui.add_space(5.0);

                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                    ui.vertical(|ui| {
                        let row_width = 112.0 + 6.0 + 144.0;

                        ui.allocate_ui_with_layout(
                            vec2(row_width, 18.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let _ = this.icons.row_button(
                                    ui,
                                    IconKey::Info,
                                    "情報",
                                    "Project information",
                                    112.0,
                                );
                                ui.add_space(6.0);
                                let _ = this.icons.text_button(
                                    ui,
                                    "ベースラインの保存",
                                    "Save baseline",
                                    144.0,
                                );
                            },
                        );

                        ui.allocate_ui_with_layout(
                            vec2(row_width, 18.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let _ = this.icons.row_button(
                                    ui,
                                    IconKey::Calendar,
                                    "カレンダー",
                                    "Change working time",
                                    112.0,
                                );
                                ui.add_space(6.0);
                                let _ = this.icons.text_button(
                                    ui,
                                    "ベースラインのクリア",
                                    "Clear baseline",
                                    144.0,
                                );
                            },
                        );

                        ui.allocate_ui_with_layout(
                            vec2(row_width, 18.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let _ = this.icons.row_button(
                                    ui,
                                    IconKey::ProjectsDialog,
                                    "プロジェクト ダイアログ",
                                    "Projects dialog",
                                    112.0,
                                );
                                ui.add_space(6.0);
                                let _ = this.icons.text_button(ui, "更新", "Update project", 144.0);
                            },
                        );
                    });
                });
            });
        });
    }

    fn draw_task_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            self.draw_task_views_zoom_band(ui);
            self.draw_clipboard_band(ui);
            self.draw_band(ui, "タスク", 468.0, |ui, this| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        if this
                            .icons
                            .row_button(ui, IconKey::InsertTask, "挿入", "Insert task", 88.0)
                            .clicked()
                        {
                            this.add_task_relative(false);
                        }
                        if this
                            .icons
                            .row_button(ui, IconKey::Delete, "削除", "Delete", 88.0)
                            .clicked()
                        {
                            this.delete_selected_task();
                        }
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        if this
                            .icons
                            .row_button(ui, IconKey::Indent, "レベル下げ", "Indent", 88.0)
                            .clicked()
                        {
                            this.adjust_selected_indent(1);
                        }
                        if this
                            .icons
                            .row_button(ui, IconKey::Outdent, "レベル上げ", "Outdent", 88.0)
                            .clicked()
                        {
                            this.adjust_selected_indent(-1);
                        }
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        let _ =
                            this.icons
                                .row_button(ui, IconKey::InsertLink, "リンク", "Link", 88.0);
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::DeleteLink,
                            "リンク解除",
                            "Unlink",
                            88.0,
                        );
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        let _ =
                            this.icons
                                .row_button(ui, IconKey::Info, "情報", "Information", 88.0);
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::Calendar,
                            "カレンダー",
                            "Change working time",
                            88.0,
                        );
                        let _ = this
                            .icons
                            .row_button(ui, IconKey::Note, "メモ", "Notes", 88.0);
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::AssignResources,
                            "リソース割当",
                            "Assign resources",
                            88.0,
                        );
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::SaveBaseline,
                            "ベースラインの保存",
                            "Save baseline",
                            88.0,
                        );
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::ClearBaseline,
                            "ベースラインのクリア",
                            "Clear baseline",
                            88.0,
                        );
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        let _ = this
                            .icons
                            .row_button(ui, IconKey::Find, "検索", "Find", 88.0);
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::ScrollToTask,
                            "タスクへスクロール",
                            "Scroll to task",
                            88.0,
                        );
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::Update,
                            "更新",
                            "Update tasks",
                            88.0,
                        );
                    });
                });
            });
        });
    }

    fn draw_resource_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            self.draw_resource_views_zoom_band(ui);
            self.draw_clipboard_band(ui);
            self.draw_band(ui, "リソース", 270.0, |ui, this| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::InsertResource,
                            "挿入",
                            "Insert resource",
                            88.0,
                        );
                        let _ = this
                            .icons
                            .row_button(ui, IconKey::Delete, "削除", "Delete", 88.0);
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::Indent,
                            "レベル下げ",
                            "Indent",
                            88.0,
                        );
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::Outdent,
                            "レベル上げ",
                            "Outdent",
                            88.0,
                        );
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        let _ =
                            this.icons
                                .row_button(ui, IconKey::Info, "情報", "Information", 88.0);
                        let _ = this.icons.row_button(
                            ui,
                            IconKey::Calendar,
                            "カレンダー",
                            "Change working time",
                            88.0,
                        );
                        let _ = this
                            .icons
                            .row_button(ui, IconKey::Note, "メモ", "Notes", 88.0);
                        let _ = this
                            .icons
                            .row_button(ui, IconKey::Find, "検索", "Find", 88.0);
                    });
                });
            });
        });
    }

    fn draw_task_views_zoom_band(&mut self, ui: &mut egui::Ui) {
        self.draw_band(ui, "ビュー", 470.0, |ui, this| {
            ui.horizontal_top(|ui| {
                let _ = this
                    .icons
                    .large_ribbon_button(ui, IconKey::Gantt, "ガント", "Gantt");
                ui.add_space(4.0);
                let _ =
                    this.icons
                        .row_button(ui, IconKey::Network, "ネットワーク", "Network", 88.0);
                ui.add_space(4.0);
                let _ = this.icons.row_button(ui, IconKey::Wbs, "WBS", "WBS", 88.0);
                ui.add_space(4.0);
                let _ = this.icons.row_button(
                    ui,
                    IconKey::TaskUsageDetail,
                    "タスク配分状況",
                    "Task usage",
                    124.0,
                );
                ui.add_space(4.0);
                let _ = this
                    .icons
                    .row_button(ui, IconKey::ZoomIn, "ズーム イン", "Zoom in", 88.0);
                ui.add_space(4.0);
                let _ =
                    this.icons
                        .row_button(ui, IconKey::ZoomOut, "ズーム アウト", "Zoom out", 88.0);
            });
        });
    }

    fn draw_resource_views_zoom_band(&mut self, ui: &mut egui::Ui) {
        self.draw_band(ui, "ビュー", 480.0, |ui, this| {
            ui.horizontal_top(|ui| {
                let _ =
                    this.icons
                        .large_ribbon_button(ui, IconKey::Resources, "リソース", "Resources");
                ui.add_space(4.0);
                let _ = this.icons.row_button(ui, IconKey::Rbs, "RBS", "RBS", 88.0);
                ui.add_space(4.0);
                let _ = this.icons.row_button(
                    ui,
                    IconKey::ResourceUsageDetail,
                    "リソース配分状況",
                    "Resource usage",
                    124.0,
                );
                ui.add_space(4.0);
                let _ = this
                    .icons
                    .row_button(ui, IconKey::ZoomIn, "ズーム イン", "Zoom in", 88.0);
                ui.add_space(4.0);
                let _ =
                    this.icons
                        .row_button(ui, IconKey::ZoomOut, "ズーム アウト", "Zoom out", 88.0);
            });
        });
    }

    fn draw_task_views_band(&mut self, ui: &mut egui::Ui) {
        self.draw_band(ui, "タスク ビュー", 384.0, |ui, this| {
            ui.horizontal_top(|ui| {
                let _ = this
                    .icons
                    .large_ribbon_button(ui, IconKey::Gantt, "ガント", "Gantt");
                ui.add_space(4.0);
                let _ =
                    this.icons
                        .row_button(ui, IconKey::Network, "ネットワーク", "Network", 88.0);
                ui.add_space(4.0);
                let _ = this.icons.row_button(ui, IconKey::Wbs, "WBS", "WBS", 88.0);
                ui.add_space(4.0);
                let _ = this.icons.row_button(
                    ui,
                    IconKey::TaskUsageDetail,
                    "タスク配分状況",
                    "Task usage",
                    124.0,
                );
            });
        });
    }

    fn draw_resource_views_band(&mut self, ui: &mut egui::Ui) {
        self.draw_band(ui, "リソース ビュー", 292.0, |ui, this| {
            ui.horizontal_top(|ui| {
                let _ =
                    this.icons
                        .large_ribbon_button(ui, IconKey::Resources, "リソース", "Resources");
                ui.add_space(4.0);
                let _ = this.icons.row_button(ui, IconKey::Rbs, "RBS", "RBS", 88.0);
                ui.add_space(4.0);
                let _ = this.icons.row_button(
                    ui,
                    IconKey::ResourceUsageDetail,
                    "リソース配分状況",
                    "Resource usage",
                    124.0,
                );
            });
        });
    }

    fn draw_view_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            self.draw_task_views_band(ui);
            self.draw_resource_views_band(ui);
            self.draw_band(ui, "その他 ビュー", 146.0, |ui, this| {
                ui.vertical(|ui| {
                    let _ = this.icons.row_button(
                        ui,
                        IconKey::Projects,
                        "プロジェクト",
                        "Projects",
                        124.0,
                    );
                    let _ = this
                        .icons
                        .row_button(ui, IconKey::Report, "レポート", "Report", 124.0);
                });
            });
            self.draw_band(ui, "サブ ビュー", 614.0, |ui, this| {
                ui.horizontal_top(|ui| {
                    let _ = this.icons.row_button(
                        ui,
                        IconKey::Histogram,
                        "ヒストグラム",
                        "Histogram",
                        90.0,
                    );
                    ui.add_space(4.0);
                    let _ = this
                        .icons
                        .row_button(ui, IconKey::Charts, "チャート", "Charts", 90.0);
                    ui.add_space(4.0);
                    let _ = this.icons.row_button(
                        ui,
                        IconKey::TaskUsage,
                        "タスク配分状況",
                        "Task usage",
                        124.0,
                    );
                    ui.add_space(4.0);
                    let _ = this.icons.row_button(
                        ui,
                        IconKey::ResourceUsage,
                        "リソース配分状況",
                        "Resource usage",
                        124.0,
                    );
                    ui.add_space(4.0);
                    let _ = this.icons.row_button(
                        ui,
                        IconKey::NoSubWindow,
                        "サブ ウィンドウ非表示",
                        "No sub window",
                        160.0,
                    );
                });
            });
            self.draw_band(ui, "フィルター", 214.0, |ui, this| {
                ui.vertical(|ui| {
                    ui.add_space(1.0);
                    this.draw_filter_combo(ui, "フィルターなし", 0);
                    this.draw_filter_combo(ui, "並べ替えなし", 1);
                    this.draw_filter_combo(ui, "グループなし", 2);
                });
            });
        });
    }

    fn draw_filter_combo(&self, ui: &mut egui::Ui, selected_text: &str, index: usize) {
        ui.scope(|ui| {
            ui.spacing_mut().interact_size = vec2(150.0, 28.0);
            egui::ComboBox::from_id_salt(("view_filter_combo", index))
                .width(150.0)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    let _ = ui.selectable_label(true, selected_text);
                    ui.separator();
                    let _ = ui.selectable_label(false, "なし");
                    let _ = ui.selectable_label(false, "標準");
                    let _ = ui.selectable_label(false, "カスタム");
                });
        });
    }

    fn draw_band<F>(&mut self, ui: &mut egui::Ui, title: &str, min_width: f32, mut build: F)
    where
        F: FnMut(&mut egui::Ui, &mut Self),
    {
        egui::Frame::new()
            .fill(Color32::from_rgb(245, 245, 245))
            .stroke(Stroke::new(1.0, Color32::from_rgb(187, 187, 187)))
            .corner_radius(egui::CornerRadius::same(1))
            .show(ui, |ui| {
                ui.set_min_width(min_width);
                ui.vertical(|ui| {
                    ui.add_space(1.0);
                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        ui.horizontal(|ui| {
                            build(ui, self);
                        });
                    });
                    ui.add_space(0.0);
                    ui.horizontal_centered(|ui| {
                        ui.label(
                            egui::RichText::new(title)
                                .size(10.0)
                                .color(Color32::from_rgb(50, 50, 50)),
                        );
                    });
                });
            });
        ui.add_space(2.0);
    }

    fn draw_logo(&self, ui: &mut egui::Ui) {
        if let Some(texture) = self.icons.logo() {
            let size = vec2(164.0, 34.0);
            let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 2.0, Color32::WHITE);
                painter.rect_stroke(
                    rect,
                    2.0,
                    Stroke::new(1.0, Color32::from_rgb(192, 192, 192)),
                    egui::StrokeKind::Outside,
                );
                painter.image(
                    texture.id(),
                    rect.shrink2(vec2(2.0, 1.0)),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            let response = response.on_hover_text("ProjectLibre");
            if response.clicked() {
                let _ = std::process::Command::new("cmd")
                    .args([
                        "/C",
                        "start",
                        "",
                        "https://github.com/tetsuji16/RustyProject",
                    ])
                    .spawn();
            }
        } else {
            ui.label(
                egui::RichText::new("ProjectLibre")
                    .size(24.0)
                    .strong()
                    .color(Color32::from_rgb(220, 20, 20)),
            );
        }
    }

    fn adjust_selected_indent(&mut self, delta: isize) {
        let Some(index) = self.current_task_index() else {
            return;
        };
        self.push_history_checkpoint();
        let next_indent = (self.snapshot.tasks[index].indent as isize + delta).max(0) as usize;
        if let Some(task) = self.snapshot.tasks.get_mut(index) {
            task.indent = next_indent;
        }
        self.snapshot.normalize();
    }

    fn load_project_from_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Project", &["json", "mpp", "xml", "pod"])
            .pick_file()
        {
            self.project_path_input = path.display().to_string();
            let path_string = self.project_path_input.clone();
            self.load_project_from_path(&path_string);
        }
    }

    fn load_project_from_path(&mut self, path: &str) {
        let ext = Path::new(path)
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase());
        match ext.as_deref() {
            Some("mpp") => match load_mpp(path) {
                Ok(snapshot) => {
                    self.push_history_checkpoint();
                    self.restore_snapshot(snapshot);
                    self.project_path_input = path.to_string();
                    self.status_message = format!("Loaded {path}");
                }
                Err(err) => {
                    self.status_message = format!("Load failed: {err}");
                }
            },
            Some("xml") => match load_xml(path) {
                Ok(snapshot) => {
                    self.push_history_checkpoint();
                    self.restore_snapshot(snapshot);
                    self.project_path_input = path.to_string();
                    self.status_message = format!("Loaded {path}");
                }
                Err(err) => {
                    self.status_message = format!("Load failed: {err}");
                }
            },
            Some("pod") => match load_pod(path) {
                Ok(snapshot) => {
                    self.push_history_checkpoint();
                    self.restore_snapshot(snapshot);
                    self.project_path_input = path.to_string();
                    self.status_message = format!("Loaded {path}");
                }
                Err(err) => {
                    self.status_message = format!("Load failed: {err}");
                }
            },
            _ => match load_project(path) {
                Ok(document) => {
                    self.push_history_checkpoint();
                    self.restore_document(document);
                    self.status_message = format!("Loaded {path}");
                }
                Err(err) => {
                    self.status_message = format!("Load failed: {err}");
                }
            },
        }
    }

    fn save_project_to_entry_or_dialog(&mut self) {
        let path = save_target_path(self.project_path_input.trim());
        if path.is_empty() {
            self.save_project_to_dialog();
            return;
        }
        self.save_project_to_path(&path);
    }

    fn save_project_to_dialog(&mut self) {
        let suggested = suggested_save_name(self.project_path_input.trim());
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Project", &["json", "xml", "pod"])
            .set_file_name(&suggested)
            .save_file()
        {
            self.project_path_input = path.display().to_string();
            let path_string = self.project_path_input.clone();
            self.save_project_to_path(&path_string);
        }
    }

    fn save_project_to_path(&mut self, path: &str) {
        let ext = Path::new(path)
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase());
        match ext.as_deref() {
            Some("xml") | Some("mpp") => match save_xml(path, &self.snapshot) {
                Ok(()) => {
                    self.status_message = format!("Saved {path}");
                }
                Err(err) => {
                    self.status_message = format!("Save failed: {err}");
                }
            },
            Some("pod") => match save_pod(path, &self.snapshot) {
                Ok(()) => {
                    self.status_message = format!("Saved {path}");
                }
                Err(err) => {
                    self.status_message = format!("Save failed: {err}");
                }
            },
            _ => {
                let document = self.capture_document();
                match save_project(path, &document) {
                    Ok(()) => {
                        self.status_message = format!("Saved {path}");
                    }
                    Err(err) => {
                        self.status_message = format!("Save failed: {err}");
                    }
                }
            }
        }
    }

    fn capture_document(&self) -> ProjectDocument {
        ProjectDocument::from_app_state(
            self.snapshot.clone(),
            self.selected_task_id,
            self.collapsed_summaries.iter().copied().collect(),
            self.day_width,
            self.left_table_width,
        )
    }

    fn push_history_checkpoint(&mut self) {
        self.history.push(self.capture_document());
    }

    fn undo(&mut self) {
        self.drag = None;
        let current = self.capture_document();
        if let Some(previous) = self.history.undo(current) {
            self.restore_document(previous);
            self.status_message = "Undo".to_string();
        } else {
            self.status_message = "Nothing to undo".to_string();
        }
    }

    fn redo(&mut self) {
        self.drag = None;
        let current = self.capture_document();
        if let Some(next) = self.history.redo(current) {
            self.restore_document(next);
            self.status_message = "Redo".to_string();
        } else {
            self.status_message = "Nothing to redo".to_string();
        }
    }

    fn restore_document(&mut self, document: ProjectDocument) {
        self.snapshot = document.snapshot;
        self.selected_task_id = document.selected_task_id;
        self.collapsed_summaries = document.collapsed_summaries.into_iter().collect();
        self.day_width = document.day_width;
        self.left_table_width = document
            .left_table_width
            .clamp(LEFT_TABLE_MIN_W, LEFT_TABLE_MAX_W);
        self.drag = None;
        self.selected_task_id = self
            .snapshot
            .task(self.selected_task_id)
            .map(|task| task.number)
            .or_else(|| self.snapshot.tasks.first().map(|task| task.number))
            .unwrap_or(0);
    }

    fn restore_snapshot(&mut self, snapshot: ProjectSnapshot) {
        self.snapshot = snapshot;
        self.selected_task_id = self
            .snapshot
            .tasks
            .first()
            .map(|task| task.number)
            .unwrap_or(0);
        self.collapsed_summaries.clear();
        self.day_width = self.day_width.clamp(14.0, 48.0);
        self.left_table_width = self
            .left_table_width
            .clamp(LEFT_TABLE_MIN_W, LEFT_TABLE_MAX_W);
        self.drag = None;
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let undo = ctx.input(|input| {
            input.key_pressed(egui::Key::Z) && input.modifiers.command && !input.modifiers.shift
        });
        let redo = ctx.input(|input| {
            (input.key_pressed(egui::Key::Y) && input.modifiers.command)
                || (input.key_pressed(egui::Key::Z)
                    && input.modifiers.command
                    && input.modifiers.shift)
        });

        if undo {
            self.undo();
        } else if redo {
            self.redo();
        }
    }

    fn toggle_summary(&mut self, task_id: usize) {
        self.push_history_checkpoint();
        if !self.collapsed_summaries.insert(task_id) {
            self.collapsed_summaries.remove(&task_id);
        }
    }

    fn current_task_index(&self) -> Option<usize> {
        self.snapshot.task_index(self.selected_task_id)
    }

    fn add_task_relative(&mut self, child: bool) {
        let Some(index) = self.current_task_index() else {
            return;
        };
        self.push_history_checkpoint();
        let reference = self.snapshot.tasks[index].clone();
        let insert_at = subtree_end_index(&self.snapshot.tasks, index) + 1;
        let new_id = self.snapshot.next_task_id();
        let indent = if child {
            reference.indent + 1
        } else {
            reference.indent
        };
        let start = reference.finish + Duration::days(1);
        let task = TaskSnapshot {
            number: new_id,
            name: if child {
                "New child task".to_string()
            } else {
                "New task".to_string()
            },
            start,
            finish: start,
            progress: 0.0,
            indent,
            summary: false,
            milestone: false,
            predecessors: Vec::new(),
            resource_names: Vec::new(),
            start_text: None,
            finish_text: None,
            duration_text: None,
            notes: None,
            deadline: None,
        };
        self.snapshot.insert_task_after(insert_at - 1, task);
        self.selected_task_id = new_id;
    }

    fn delete_selected_task(&mut self) {
        let Some(index) = self.current_task_index() else {
            return;
        };
        self.push_history_checkpoint();
        let deleted_ids = subtree_task_ids(&self.snapshot.tasks, index);
        self.snapshot.clear_display_texts();
        self.snapshot.remove_subtree_at(index);
        for task in &mut self.snapshot.tasks {
            task.predecessors
                .retain(|pred| !deleted_ids.contains(&pred.predecessor));
        }
        self.selected_task_id = self
            .snapshot
            .tasks
            .get(index.saturating_sub(1))
            .map(|task| task.number)
            .or_else(|| self.snapshot.tasks.first().map(|task| task.number))
            .unwrap_or(0);
        self.snapshot.normalize();
    }

    fn send_edit(&mut self, edit: EditCommand) {
        self.snapshot.apply_edit(edit);
    }

    fn handle_pointer(
        &mut self,
        ctx: &egui::Context,
        content_rect: Rect,
        left_table_width: f32,
        chart: &crate::ui::gantt_view::TimelineGeometry,
        visible_rows: &[crate::ui::gantt_view::VisibleTaskRow],
    ) {
        let pointer = ctx.input(|input| input.pointer.clone());
        let hover = pointer.hover_pos().and_then(|pos| {
            crate::ui::gantt_chart::hit_test_task_bar(
                chart,
                &self.snapshot.tasks,
                visible_rows,
                pos,
            )
        });

        if self.drag.is_none() {
            if let Some(hit) = hover {
                let cursor = match hit.action {
                    crate::ui::gantt_chart::DragAction::ResizeStart
                    | crate::ui::gantt_chart::DragAction::ResizeEnd => {
                        egui::CursorIcon::ResizeHorizontal
                    }
                    crate::ui::gantt_chart::DragAction::Progress => egui::CursorIcon::PointingHand,
                    crate::ui::gantt_chart::DragAction::Move => egui::CursorIcon::Grab,
                };
                ctx.set_cursor_icon(cursor);
            }
        } else {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        }

        if pointer.primary_pressed() {
            if let Some(pointer_pos) = pointer.interact_pos() {
                let local_pos = pos2(
                    pointer_pos.x - content_rect.left(),
                    pointer_pos.y - content_rect.top(),
                );
                let table_rect = Rect::from_min_size(
                    pos2(0.0, 0.0),
                    vec2(left_table_width, content_rect.height()),
                );
                if let Some((row, is_toggle)) = crate::ui::task_table::hit_test_row_toggle(
                    table_rect,
                    &self.snapshot.tasks,
                    visible_rows,
                    local_pos,
                ) {
                    let task = &self.snapshot.tasks[row.task_index];
                    self.selected_task_id = task.number;
                    if is_toggle && task.summary {
                        self.drag = None;
                        self.toggle_summary(task.number);
                        ctx.request_repaint();
                        return;
                    }
                }
            }

            if let Some(hit) = hover {
                let task = self.snapshot.tasks[hit.task_index].clone();
                self.selected_task_id = task.number;
                let history_snapshot = self.capture_document();
                self.drag = Some(DragState {
                    task_index: hit.task_index,
                    task_id: task.number,
                    action: hit.action,
                    origin_pointer: hit.pointer,
                    current_pointer: hit.pointer,
                    original_start: task.start,
                    original_finish: task.finish,
                    history_snapshot,
                    changed: false,
                });
            } else if let Some(pointer_pos) = pointer.interact_pos() {
                if let Some(row_index) = chart.row_at(pointer_pos, visible_rows.len()) {
                    let task_index = visible_rows[row_index].task_index;
                    self.selected_task_id = self.snapshot.tasks[task_index].number;
                }
            }
        }

        if let Some(drag) = self.drag.clone() {
            if pointer.primary_down() {
                if let Some(pointer_pos) = pointer.interact_pos() {
                    if let Some(active_drag) = self.drag.as_mut() {
                        active_drag.current_pointer = pointer_pos;
                        if active_drag.current_pointer != active_drag.origin_pointer {
                            active_drag.changed = true;
                        }
                    }
                    self.selected_task_id = drag.task_id;
                    ctx.request_repaint();
                }
            }

            if pointer.primary_released() {
                if let Some(active_drag) = self.drag.take() {
                    if active_drag.changed {
                        if let Some(edit) = active_drag.to_edit(chart) {
                            self.send_edit(edit);
                            self.history.push(active_drag.history_snapshot);
                        }
                    }
                }
            }
        }
    }
}

fn save_target_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let path = Path::new(trimmed);
    match path.extension().and_then(|value| value.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => trimmed.to_string(),
        Some(ext)
            if ext.eq_ignore_ascii_case("xml")
                || ext.eq_ignore_ascii_case("mpp")
                || ext.eq_ignore_ascii_case("pod") =>
        {
            path.with_extension("xml").display().to_string()
        }
        Some(_) => path.with_extension("xml").display().to_string(),
        None => path.with_extension("xml").display().to_string(),
    }
}

fn suggested_save_name(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return "project.xml".to_string();
    }

    let path = Path::new(trimmed);
    match path.extension().and_then(|value| value.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => path
            .with_extension("json")
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value.to_string())
            .unwrap_or_else(|| "project.json".to_string()),
        Some(ext)
            if ext.eq_ignore_ascii_case("xml")
                || ext.eq_ignore_ascii_case("mpp")
                || ext.eq_ignore_ascii_case("pod") =>
        {
            path.with_extension("xml")
                .file_name()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string())
                .unwrap_or_else(|| "project.xml".to_string())
        }
        _ => "project.xml".to_string(),
    }
}

fn subtree_end_index(tasks: &[TaskSnapshot], index: usize) -> usize {
    let indent = tasks[index].indent;
    let mut end = index + 1;
    while end < tasks.len() && tasks[end].indent > indent {
        end += 1;
    }
    end - 1
}

fn subtree_task_ids(tasks: &[TaskSnapshot], index: usize) -> HashSet<usize> {
    let mut ids = HashSet::new();
    let end = subtree_end_index(tasks, index);
    for task in &tasks[index..=end] {
        ids.insert(task.number);
    }
    ids
}

#[derive(Clone)]
struct DragState {
    task_index: usize,
    task_id: usize,
    action: crate::ui::gantt_chart::DragAction,
    origin_pointer: Pos2,
    current_pointer: Pos2,
    original_start: NaiveDate,
    original_finish: NaiveDate,
    history_snapshot: ProjectDocument,
    changed: bool,
}

impl DragState {
    fn preview(&self) -> crate::ui::gantt_view::DragPreview {
        crate::ui::gantt_view::DragPreview {
            task_index: self.task_index,
            task_id: self.task_id,
            action: self.action,
            origin_pointer: self.origin_pointer,
            current_pointer: self.current_pointer,
            original_start: self.original_start,
            original_finish: self.original_finish,
        }
    }

    fn to_edit(&self, chart: &crate::ui::gantt_view::TimelineGeometry) -> Option<EditCommand> {
        let delta_days = chart.pixel_delta_to_days(self.current_pointer.x - self.origin_pointer.x);
        match self.action {
            crate::ui::gantt_chart::DragAction::Move => Some(EditCommand::Move {
                id: self.task_id,
                start: self.original_start + Duration::days(delta_days),
                finish: self.original_finish + Duration::days(delta_days),
            }),
            crate::ui::gantt_chart::DragAction::ResizeStart => Some(EditCommand::ResizeStart {
                id: self.task_id,
                start: self.original_start + Duration::days(delta_days),
            }),
            crate::ui::gantt_chart::DragAction::ResizeEnd => Some(EditCommand::ResizeEnd {
                id: self.task_id,
                finish: self.original_finish + Duration::days(delta_days),
            }),
            crate::ui::gantt_chart::DragAction::Progress => {
                let bar = crate::ui::gantt_chart::task_bar_rect_for_dates_at_y(
                    chart,
                    self.original_start,
                    self.original_finish,
                    chart.row_top(self.task_index) + ROW_H * 0.5,
                );
                let progress =
                    ((self.current_pointer.x - bar.left()) / bar.width().max(1.0)).clamp(0.0, 1.0);
                Some(EditCommand::SetProgress {
                    id: self.task_id,
                    progress,
                })
            }
        }
    }
}

fn bundled_sample_path() -> Option<PathBuf> {
    let xml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_XML_PATH);
    if xml_path.exists() {
        return Some(xml_path);
    }

    let pod_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_POD_PATH);
    if pod_path.exists() {
        return Some(pod_path);
    }

    let mpp_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_MPP_PATH);
    mpp_path.exists().then_some(mpp_path)
}
