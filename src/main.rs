mod history;
mod model;
mod mpp_import;
mod project_file;
mod schedule;

use chrono::{Datelike, Duration, NaiveDate};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke,
};
use history::UndoRedo;
use model::{EditCommand, ProjectSnapshot, TaskSnapshot};
use mpp_import::load_mpp;
use project_file::{load as load_project, save as save_project, ProjectDocument};
use std::collections::HashSet;
use std::path::Path;

const APP_NAME: &str = "ProjectLibre Gantt - Rust";
const VIEW_WIDTH: f32 = 2048.0;
const VIEW_HEIGHT: f32 = 1222.0;

const HEADER_H: f32 = 54.0;
const MONTH_H: f32 = 28.0;
const DAY_H: f32 = 26.0;
const ROW_H: f32 = 31.0;
const LEFT_ROW_NO_W: f32 = 58.0;
const LEFT_WBS_W: f32 = 88.0;
const LEFT_ICON_W: f32 = 44.0;
const LEFT_NAME_W: f32 = 360.0;
const LEFT_DURATION_W: f32 = 96.0;
const LEFT_TABLE_W: f32 = LEFT_ROW_NO_W + LEFT_WBS_W + LEFT_ICON_W + LEFT_NAME_W + LEFT_DURATION_W;
const SPLITTER_W: f32 = 6.0;
const CHART_MARGIN_X: f32 = 10.0;
const DAY_W: f32 = 24.0;

const BAR_H: f32 = 14.0;
const SUMMARY_H: f32 = 10.0;
const MILESTONE_SIZE: f32 = 16.0;
const BAR_HANDLE_W: f32 = 7.0;
const BAR_HIT_PAD: f32 = 4.0;

type ProjectTask = TaskSnapshot;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([VIEW_WIDTH, VIEW_HEIGHT])
            .with_title(APP_NAME),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(GanttApp::new(cc)))),
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
    inspector_task_id: Option<usize>,
    editor_start: String,
    editor_finish: String,
    editor_name: String,
    editor_indent: String,
    editor_predecessors: String,
}

impl GanttApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(240, 240, 240);
        visuals.widgets.noninteractive.bg_stroke =
            Stroke::new(1.0, Color32::from_rgb(185, 185, 185));
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(248, 248, 248);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(205, 205, 205));
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(252, 252, 252);
        visuals.widgets.active.bg_fill = Color32::from_rgb(225, 236, 248);
        cc.egui_ctx.set_visuals(visuals);

        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = vec2(0.0, 0.0);
        style.spacing.window_margin = egui::Margin::same(0);
        cc.egui_ctx.set_style(style);

        let snapshot = ProjectSnapshot::sample();
        let selected_task_id = snapshot.tasks.first().map(|task| task.number).unwrap_or(0);

        Self {
            selected_task_id,
            history: UndoRedo::default(),
            drag: None,
            collapsed_summaries: HashSet::new(),
            day_width: DAY_W,
            left_table_width: LEFT_TABLE_W,
            project_path_input: "project.json".to_string(),
            status_message: String::from("Ready"),
            inspector_task_id: None,
            editor_start: String::new(),
            editor_finish: String::new(),
            editor_name: String::new(),
            editor_indent: String::new(),
            editor_predecessors: String::new(),
            snapshot,
        }
    }
}

impl eframe::App for GanttApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);
        self.ensure_inspector_sync();
        self.draw_toolbar(ctx);
        self.draw_inspector(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(248, 248, 248)))
            .show(ctx, |ui| {
                let visible_rows =
                    build_visible_rows(&self.snapshot.tasks, &self.collapsed_summaries);
                let rect = ui.max_rect();
                let chart =
                    ChartGeometry::new(rect, &self.snapshot, self.day_width, self.left_table_width);

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let content_size = vec2(
                            self.content_width(&chart),
                            self.content_height(visible_rows.len()),
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
                                .clamp(360.0, 760.0);
                        }
                        if splitter_response.hovered() || splitter_response.dragged() {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }
                        let painter = ui.painter_at(content_rect);
                        self.handle_pointer(ctx, &chart, &visible_rows);

                        draw_workspace(
                            &painter,
                            content_rect,
                            &chart,
                            &self.snapshot.tasks,
                            &visible_rows,
                            self.selected_task_id,
                            &self.collapsed_summaries,
                            self.left_table_width,
                        );
                    });
            });
    }
}

impl GanttApp {
    fn ensure_inspector_sync(&mut self) {
        if self.inspector_task_id == Some(self.selected_task_id) {
            return;
        }

        if let Some(task) = self.current_task().cloned() {
            self.inspector_task_id = Some(task.number);
            self.editor_start = task.start.format("%Y-%m-%d").to_string();
            self.editor_finish = task.finish.format("%Y-%m-%d").to_string();
            self.editor_name = task.name;
            self.editor_indent = task.indent.to_string();
            self.editor_predecessors = task
                .predecessors
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ");
        } else {
            self.inspector_task_id = None;
            self.editor_start.clear();
            self.editor_finish.clear();
            self.editor_name.clear();
            self.editor_indent.clear();
            self.editor_predecessors.clear();
        }
    }

    fn draw_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(241, 241, 241))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(181, 181, 181))),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(6.0);
                    ui.label("File");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.project_path_input)
                            .desired_width(220.0),
                    );
                    if ui.button("Load").clicked() {
                        self.load_project_from_entry_or_dialog();
                    }
                    if ui.button("Import").clicked() {
                        self.load_project_from_dialog();
                    }
                    if ui.button("Save").clicked() {
                        self.save_project_to_entry_or_dialog();
                    }
                    if ui.button("Export").clicked() {
                        self.save_project_to_dialog();
                    }
                    if ui.button("New").clicked() {
                        self.push_history_checkpoint();
                        self.snapshot = ProjectSnapshot::sample();
                        self.status_message = "Loaded sample project".to_string();
                        self.selected_task_id = self
                            .snapshot
                            .tasks
                            .first()
                            .map(|task| task.number)
                            .unwrap_or(0);
                        self.collapsed_summaries.clear();
                        self.drag = None;
                        self.inspector_task_id = None;
                    }

                    ui.separator();

                    let undo_enabled = self.history.can_undo();
                    if ui
                        .add_enabled(undo_enabled, egui::Button::new("Undo"))
                        .clicked()
                    {
                        self.undo();
                    }
                    let redo_enabled = self.history.can_redo();
                    if ui
                        .add_enabled(redo_enabled, egui::Button::new("Redo"))
                        .clicked()
                    {
                        self.redo();
                    }

                    ui.separator();

                    if ui.button("A-").clicked() {
                        self.day_width = (self.day_width - 2.0).max(14.0);
                    }
                    if ui.button("A+").clicked() {
                        self.day_width = (self.day_width + 2.0).min(48.0);
                    }
                    ui.add(
                        egui::Slider::new(&mut self.day_width, 14.0..=48.0)
                            .text("Day width")
                            .clamping(egui::SliderClamping::Always),
                    );

                    ui.separator();
                    if ui.button("Collapse all").clicked() {
                        self.push_history_checkpoint();
                        self.collapsed_summaries.clear();
                        for task in &self.snapshot.tasks {
                            if task.summary {
                                self.collapsed_summaries.insert(task.number);
                            }
                        }
                    }
                    if ui.button("Expand all").clicked() {
                        self.push_history_checkpoint();
                        self.collapsed_summaries.clear();
                    }

                    ui.separator();
                    ui.label(format!("Tasks: {}", self.snapshot.tasks.len()));
                    ui.label(format!(
                        "Visible: {}",
                        build_visible_rows(&self.snapshot.tasks, &self.collapsed_summaries).len()
                    ));
                    ui.separator();
                    ui.label(self.status_message.as_str());
                });
            });
    }

    fn draw_inspector(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector")
            .default_width(300.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(248, 248, 248))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(186, 186, 186))),
            )
            .show(ctx, |ui| {
                ui.heading("Task");
                ui.add_space(8.0);

                let Some(task) = self.current_task().cloned() else {
                    ui.label("No task selected");
                    return;
                };

                ui.label(format!("ID {}", task.number));
                if let Some(wbs) = self
                    .snapshot
                    .task_index(task.number)
                    .and_then(|index| build_wbs_codes(&self.snapshot.tasks).get(index).cloned())
                {
                    ui.label(format!("WBS {}", wbs));
                }
                let name_response = ui.add(
                    egui::TextEdit::singleline(&mut self.editor_name).desired_width(f32::INFINITY),
                );
                if name_response.lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                {
                    self.commit_name_editor(task.number);
                }
                ui.label(if task.summary {
                    "Summary task"
                } else if task.milestone {
                    "Milestone"
                } else {
                    "Task"
                });
                ui.label(format!("Duration: {}", task.duration_label()));
                ui.separator();

                ui.label("Start");
                let start_response =
                    ui.add(egui::TextEdit::singleline(&mut self.editor_start).desired_width(120.0));
                if start_response.lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                {
                    self.commit_date_editor(task.number, true);
                }

                ui.label("Finish");
                let finish_response = ui
                    .add(egui::TextEdit::singleline(&mut self.editor_finish).desired_width(120.0));
                if finish_response.lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                {
                    self.commit_date_editor(task.number, false);
                }

                ui.label("Indent");
                let indent_response = ui
                    .add(egui::TextEdit::singleline(&mut self.editor_indent).desired_width(120.0));
                if indent_response.lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                {
                    self.commit_indent_editor(task.number);
                }

                ui.label("Predecessors");
                let preds_response = ui.add(
                    egui::TextEdit::singleline(&mut self.editor_predecessors)
                        .desired_width(f32::INFINITY),
                );
                if preds_response.lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                {
                    self.commit_predecessors_editor(task.number);
                }

                ui.add_space(8.0);
                let mut progress = task.progress;
                if ui
                    .add(egui::Slider::new(&mut progress, 0.0..=1.0).text("Progress"))
                    .changed()
                {
                    self.send_edit(EditCommand::SetProgress {
                        id: task.number,
                        progress,
                    });
                }

                ui.add_space(8.0);
                if task.summary {
                    let collapsed = self.collapsed_summaries.contains(&task.number);
                    if ui
                        .button(if collapsed { "Expand" } else { "Collapse" })
                        .clicked()
                    {
                        self.toggle_summary(task.number);
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Add Child").clicked() {
                        self.add_task_relative(true);
                    }
                    if ui.button("Add Sibling").clicked() {
                        self.add_task_relative(false);
                    }
                    if ui.button("Delete").clicked() {
                        self.delete_selected_task();
                    }
                });
            });
    }

    fn commit_date_editor(&mut self, task_id: usize, start: bool) {
        let text = if start {
            self.editor_start.clone()
        } else {
            self.editor_finish.clone()
        };
        let Ok(date) = NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d") else {
            return;
        };

        self.push_history_checkpoint();
        if start {
            self.send_edit(EditCommand::ResizeStart {
                id: task_id,
                start: date,
            });
        } else {
            self.send_edit(EditCommand::ResizeEnd {
                id: task_id,
                finish: date,
            });
        }
    }

    fn commit_name_editor(&mut self, task_id: usize) {
        self.push_history_checkpoint();
        if let Some(task) = self.snapshot.task_mut(task_id) {
            task.name = self.editor_name.trim().to_string();
        }
        self.snapshot.normalize();
        self.inspector_task_id = None;
    }

    fn commit_indent_editor(&mut self, task_id: usize) {
        let Ok(indent) = self.editor_indent.trim().parse::<usize>() else {
            return;
        };

        self.push_history_checkpoint();
        if let Some(task) = self.snapshot.task_mut(task_id) {
            task.indent = indent;
        }
        self.snapshot.normalize();
        self.inspector_task_id = None;
    }

    fn commit_predecessors_editor(&mut self, task_id: usize) {
        let predecessors = self
            .editor_predecessors
            .split(',')
            .filter_map(|value| value.trim().parse::<usize>().ok())
            .collect::<Vec<_>>();

        self.push_history_checkpoint();
        if let Some(task) = self.snapshot.task_mut(task_id) {
            task.predecessors = predecessors;
        }
        self.snapshot.normalize();
        self.inspector_task_id = None;
    }

    fn send_edit(&mut self, edit: EditCommand) {
        self.snapshot.apply_edit(edit);
        self.inspector_task_id = None;
    }

    fn load_project_from_entry_or_dialog(&mut self) {
        let path = self.project_path_input.trim().to_string();
        if !path.is_empty() && Path::new(&path).exists() {
            return self.load_project_from_path(&path);
        }
        self.load_project_from_dialog();
    }

    fn load_project_from_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Project", &["json", "mpp"])
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
                    self.project_path_input = json_save_path(path);
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
        let path = json_save_path(self.project_path_input.trim());
        if path.is_empty() {
            self.save_project_to_dialog();
            return;
        }
        self.save_project_to_path(&path);
    }

    fn save_project_to_dialog(&mut self) {
        let suggested = if self.project_path_input.trim().is_empty() {
            "project.json".to_string()
        } else {
            json_save_path(self.project_path_input.trim())
        };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Project", &["json"])
            .set_file_name(&suggested)
            .save_file()
        {
            self.project_path_input = path.display().to_string();
            let path_string = self.project_path_input.clone();
            self.save_project_to_path(&path_string);
        }
    }

    fn save_project_to_path(&mut self, path: &str) {
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
        self.left_table_width = document.left_table_width;
        self.drag = None;
        self.selected_task_id = self
            .snapshot
            .task(self.selected_task_id)
            .map(|task| task.number)
            .or_else(|| self.snapshot.tasks.first().map(|task| task.number))
            .unwrap_or(0);
        self.inspector_task_id = None;
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
        self.left_table_width = self.left_table_width.clamp(360.0, 760.0);
        self.inspector_task_id = None;
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

    fn current_task(&self) -> Option<&ProjectTask> {
        self.snapshot
            .tasks
            .iter()
            .find(|task| task.number == self.selected_task_id)
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
        };
        self.snapshot.insert_task_after(insert_at - 1, task);
        self.selected_task_id = new_id;
        self.inspector_task_id = None;
    }

    fn delete_selected_task(&mut self) {
        let Some(index) = self.current_task_index() else {
            return;
        };
        self.push_history_checkpoint();
        let deleted_ids = subtree_task_ids(&self.snapshot.tasks, index);
        self.snapshot.remove_subtree_at(index);
        for task in &mut self.snapshot.tasks {
            task.predecessors.retain(|pred| !deleted_ids.contains(pred));
        }
        self.selected_task_id = self
            .snapshot
            .tasks
            .get(index.saturating_sub(1))
            .map(|task| task.number)
            .or_else(|| self.snapshot.tasks.first().map(|task| task.number))
            .unwrap_or(0);
        self.snapshot.normalize();
        self.inspector_task_id = None;
    }

    fn handle_pointer(
        &mut self,
        ctx: &egui::Context,
        chart: &ChartGeometry,
        visible_rows: &[VisibleTaskRow],
    ) {
        let pointer = ctx.input(|input| input.pointer.clone());
        let hover = pointer
            .hover_pos()
            .and_then(|pos| hit_test_task_bar(chart, &self.snapshot.tasks, visible_rows, pos));

        if self.drag.is_none() {
            if let Some(hit) = hover {
                let cursor = match hit.action {
                    DragAction::ResizeStart | DragAction::ResizeEnd => {
                        egui::CursorIcon::ResizeHorizontal
                    }
                    DragAction::Progress => egui::CursorIcon::PointingHand,
                    DragAction::Move => egui::CursorIcon::Grab,
                };
                ctx.set_cursor_icon(cursor);
            }
        } else {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        }

        if pointer.primary_pressed() {
            if let Some(pointer_pos) = pointer.interact_pos() {
                if let Some((row, is_toggle)) =
                    hit_test_left_row(chart, &self.snapshot.tasks, visible_rows, pointer_pos)
                {
                    let task = &self.snapshot.tasks[row.task_index];
                    self.selected_task_id = task.number;
                    self.inspector_task_id = None;
                    if is_toggle && task.summary {
                        self.toggle_summary(task.number);
                        ctx.request_repaint();
                        return;
                    }
                }
            }

            if let Some(hit) = hover {
                let task = self.snapshot.tasks[hit.task_index].clone();
                self.selected_task_id = task.number;
                self.inspector_task_id = None;
                let history_snapshot = self.capture_document();
                self.drag = Some(DragState {
                    task_index: hit.task_index,
                    task_id: task.number,
                    action: hit.action,
                    origin_pointer: hit.pointer,
                    original_start: task.start,
                    original_finish: task.finish,
                    history_snapshot,
                    changed: false,
                });
            } else if let Some(pointer_pos) = pointer.interact_pos() {
                if let Some(row_index) = chart.row_at(pointer_pos, visible_rows.len()) {
                    let task_index = visible_rows[row_index].task_index;
                    self.selected_task_id = self.snapshot.tasks[task_index].number;
                    self.inspector_task_id = None;
                }
            }
        }

        if let Some(drag) = self.drag.clone() {
            if pointer.primary_down() {
                if let Some(pointer_pos) = pointer.interact_pos() {
                    if let Some(edit) = drag.to_edit(chart, pointer_pos) {
                        self.send_edit(edit);
                        if let Some(active_drag) = self.drag.as_mut() {
                            active_drag.changed = true;
                        }
                        self.selected_task_id = drag.task_id;
                    }
                    ctx.request_repaint();
                }
            }

            if pointer.any_released() {
                if let Some(active_drag) = self.drag.take() {
                    if active_drag.changed {
                        self.history.push(active_drag.history_snapshot);
                    }
                }
                self.inspector_task_id = None;
            }
        }
    }
}

fn draw_workspace(
    painter: &Painter,
    rect: Rect,
    chart: &ChartGeometry,
    tasks: &[ProjectTask],
    visible_rows: &[VisibleTaskRow],
    selected_task_id: usize,
    collapsed_summaries: &HashSet<usize>,
    left_table_width: f32,
) {
    let left_rect = Rect::from_min_max(
        rect.min,
        pos2(rect.left() + left_table_width, rect.bottom()),
    );
    let gantt_rect = Rect::from_min_max(pos2(chart.gantt_left, rect.top()), rect.max);
    let splitter_x = left_rect.right() + SPLITTER_W * 0.5;

    painter.rect_filled(rect, 0.0, Color32::from_rgb(248, 248, 248));
    painter.rect_filled(left_rect, 0.0, Color32::from_rgb(252, 252, 252));
    painter.rect_filled(gantt_rect, 0.0, Color32::from_rgb(250, 250, 250));

    painter.line_segment(
        [
            pos2(splitter_x, rect.top()),
            pos2(splitter_x, rect.bottom()),
        ],
        Stroke::new(1.0, Color32::from_rgb(160, 160, 160)),
    );

    draw_left_headers(painter, left_rect, left_table_width);
    draw_timeline_headers(painter, gantt_rect, chart);
    draw_left_rows(
        painter,
        left_rect,
        tasks,
        visible_rows,
        selected_task_id,
        collapsed_summaries,
        left_table_width,
    );
    draw_gantt_rows_and_grid(
        painter,
        gantt_rect,
        chart,
        tasks,
        visible_rows,
        selected_task_id,
    );
    draw_dependency_links(painter, chart, tasks, visible_rows);
    draw_task_bars(painter, chart, tasks, visible_rows, selected_task_id);
}

fn draw_left_headers(painter: &Painter, rect: Rect, table_width: f32) {
    let border = Color32::from_rgb(175, 175, 175);
    let header_rect = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER_H));

    painter.rect_filled(header_rect, 0.0, Color32::from_rgb(241, 241, 241));
    painter.rect_stroke(
        header_rect,
        0.0,
        Stroke::new(1.0, border),
        egui::StrokeKind::Outside,
    );
    draw_vertical_table_lines(
        painter,
        rect.left(),
        rect.top(),
        rect.top() + HEADER_H,
        border,
        table_width,
    );

    draw_header_text(
        painter,
        pos2(
            rect.left()
                + LEFT_ROW_NO_W
                + LEFT_WBS_W
                + LEFT_ICON_W
                + name_column_width(table_width) * 0.5,
            rect.top() + HEADER_H * 0.5,
        ),
        "Name",
    );
    draw_header_text(
        painter,
        pos2(
            rect.left() + LEFT_ROW_NO_W + LEFT_WBS_W * 0.5,
            rect.top() + HEADER_H * 0.5,
        ),
        "WBS",
    );
    draw_header_text(
        painter,
        pos2(
            rect.left() + LEFT_ROW_NO_W * 0.5,
            rect.top() + HEADER_H * 0.5,
        ),
        "#",
    );
    draw_header_text(
        painter,
        pos2(
            rect.right() - LEFT_DURATION_W * 0.5,
            rect.top() + HEADER_H * 0.5,
        ),
        "Duration",
    );
}

fn draw_header_text(painter: &Painter, pos: Pos2, text: &str) {
    painter.text(
        pos,
        Align2::CENTER_CENTER,
        text,
        FontId::new(18.0, FontFamily::Proportional),
        Color32::from_rgb(28, 28, 28),
    );
}

fn draw_timeline_headers(painter: &Painter, rect: Rect, chart: &ChartGeometry) {
    let border = Color32::from_rgb(175, 175, 175);
    let header_rect = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER_H));
    painter.rect_filled(header_rect, 0.0, Color32::from_rgb(241, 241, 241));
    painter.line_segment(
        [
            pos2(rect.left(), header_rect.bottom()),
            pos2(rect.right(), header_rect.bottom()),
        ],
        Stroke::new(1.0, border),
    );

    for (label, start, end) in month_spans(chart.start_date, chart.end_date) {
        let x0 = chart.date_to_x(start).max(rect.left());
        let x1 = chart.date_to_x(end + Duration::days(1)).min(rect.right());
        if x1 <= rect.left() || x0 >= rect.right() {
            continue;
        }

        let month_rect = Rect::from_min_max(pos2(x0, rect.top()), pos2(x1, rect.top() + MONTH_H));
        painter.rect_stroke(
            month_rect,
            0.0,
            Stroke::new(1.0, border),
            egui::StrokeKind::Outside,
        );
        painter.text(
            month_rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::new(18.0, FontFamily::Proportional),
            Color32::from_rgb(45, 45, 45),
        );
    }

    let days_top = rect.top() + MONTH_H;
    let days_rect = Rect::from_min_max(
        pos2(rect.left(), days_top),
        pos2(
            chart.date_to_x(chart.end_date + Duration::days(1)),
            rect.top() + HEADER_H,
        ),
    );
    painter.rect_stroke(
        days_rect,
        0.0,
        Stroke::new(1.0, border),
        egui::StrokeKind::Outside,
    );

    let mut day = chart.start_date;
    while day <= chart.end_date {
        let x = chart.date_to_x(day);
        let is_weekend = matches!(day.weekday().number_from_monday(), 6 | 7);
        let stroke = if is_weekend {
            Stroke::new(1.0, Color32::from_rgb(210, 210, 210))
        } else {
            Stroke::new(1.0, Color32::from_rgb(226, 226, 226))
        };
        painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
        painter.text(
            pos2(x + chart.day_width * 0.5, days_top + DAY_H * 0.5),
            Align2::CENTER_CENTER,
            format!("{:02}", day.day()),
            FontId::new(17.0, FontFamily::Proportional),
            Color32::from_rgb(34, 34, 34),
        );
        day += Duration::days(1);
    }
}

fn name_column_width(table_width: f32) -> f32 {
    (table_width - LEFT_ROW_NO_W - LEFT_WBS_W - LEFT_ICON_W - LEFT_DURATION_W).max(120.0)
}

fn draw_left_rows(
    painter: &Painter,
    rect: Rect,
    tasks: &[ProjectTask],
    visible_rows: &[VisibleTaskRow],
    selected_task_id: usize,
    collapsed_summaries: &HashSet<usize>,
    table_width: f32,
) {
    let line = Color32::from_rgb(214, 214, 214);
    let text = Color32::from_rgb(28, 28, 28);
    let selected_text = Color32::from_rgb(255, 255, 255);
    let wbs_codes = build_wbs_codes(tasks);

    for (row_index, row) in visible_rows.iter().enumerate() {
        let task = &tasks[row.task_index];
        let y = rect.top() + HEADER_H + row_index as f32 * ROW_H;
        let row_rect = Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), ROW_H));
        let selected_row = task.number == selected_task_id;
        let bg = if selected_row {
            Color32::from_rgb(92, 92, 92)
        } else if task.summary {
            Color32::from_rgb(250, 250, 250)
        } else {
            Color32::from_rgb(255, 255, 255)
        };

        painter.rect_filled(row_rect, 0.0, bg);
        painter.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, line),
        );
        painter.line_segment(
            [pos2(rect.left(), y + ROW_H), pos2(rect.right(), y + ROW_H)],
            Stroke::new(1.0, line),
        );
        draw_vertical_table_lines(painter, rect.left(), y, y + ROW_H, line, table_width);

        let color = if selected_row { selected_text } else { text };
        painter.text(
            pos2(rect.left() + 18.0, y + ROW_H * 0.5),
            Align2::LEFT_CENTER,
            task.number.to_string(),
            FontId::new(17.0, FontFamily::Proportional),
            color,
        );

        painter.text(
            pos2(rect.left() + LEFT_ROW_NO_W + 12.0, y + ROW_H * 0.5),
            Align2::LEFT_CENTER,
            wbs_codes[row.task_index].as_str(),
            FontId::new(15.0, FontFamily::Monospace),
            color,
        );

        let name_x = rect.left()
            + LEFT_ROW_NO_W
            + LEFT_WBS_W
            + LEFT_ICON_W
            + 8.0
            + task.indent as f32 * 18.0;
        if task.summary {
            let expanded = !collapsed_summaries.contains(&task.number);
            draw_expand_box(
                painter,
                pos2(name_x, y + ROW_H * 0.5),
                selected_row,
                expanded,
            );
        } else if task.milestone {
            draw_milestone_icon(painter, pos2(name_x + 1.0, y + ROW_H * 0.5), selected_row);
        }

        let name_offset = if task.summary || task.milestone {
            18.0
        } else {
            0.0
        };
        let font = if task.summary { 17.0 } else { 16.0 };
        painter.text(
            pos2(name_x + name_offset, y + ROW_H * 0.5),
            Align2::LEFT_CENTER,
            task.name.as_str(),
            FontId::new(font, FontFamily::Proportional),
            color,
        );
        painter.text(
            pos2(rect.right() - LEFT_DURATION_W + 10.0, y + ROW_H * 0.5),
            Align2::LEFT_CENTER,
            task.duration_label(),
            FontId::new(16.0, FontFamily::Proportional),
            color,
        );
    }
}

fn draw_gantt_rows_and_grid(
    painter: &Painter,
    rect: Rect,
    chart: &ChartGeometry,
    tasks: &[ProjectTask],
    visible_rows: &[VisibleTaskRow],
    selected_task_id: usize,
) {
    let painter = painter.with_clip_rect(rect);
    let line = Color32::from_rgb(224, 224, 224);

    painter.rect_filled(rect, 0.0, Color32::from_rgb(250, 250, 250));

    let mut day = chart.start_date;
    while day <= chart.end_date {
        let x = chart.date_to_x(day);
        let weekday = day.weekday().number_from_monday();
        if weekday >= 6 {
            let weekend_rect = Rect::from_min_size(
                pos2(x, rect.top() + HEADER_H),
                vec2(chart.day_width, rect.height() - HEADER_H),
            );
            painter.rect_filled(weekend_rect, 0.0, Color32::from_rgb(244, 244, 244));
        }

        let stroke = if day.day() == 1 || day.day() == 15 {
            Stroke::new(1.0, Color32::from_rgb(200, 200, 200))
        } else {
            Stroke::new(1.0, Color32::from_rgb(232, 232, 232))
        };
        painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
        day += Duration::days(1);
    }

    for row_index in 0..visible_rows.len() {
        let task = &tasks[visible_rows[row_index].task_index];
        let y = chart.row_top(row_index);
        if task.number == selected_task_id {
            painter.rect_filled(
                Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), ROW_H)),
                0.0,
                Color32::from_rgba_premultiplied(90, 120, 150, 26),
            );
        }
        painter.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, line),
        );
        painter.line_segment(
            [pos2(rect.left(), y + ROW_H), pos2(rect.right(), y + ROW_H)],
            Stroke::new(1.0, line),
        );
    }

    let project_start_x = chart.date_to_x(chart.start_date);
    painter.line_segment(
        [
            pos2(project_start_x, rect.top() + HEADER_H),
            pos2(project_start_x, rect.bottom()),
        ],
        Stroke::new(1.0, Color32::from_rgb(155, 196, 155)),
    );
}

fn draw_task_bars(
    painter: &Painter,
    chart: &ChartGeometry,
    tasks: &[ProjectTask],
    visible_rows: &[VisibleTaskRow],
    selected_task_id: usize,
) {
    for (row_index, row) in visible_rows.iter().enumerate() {
        let task = &tasks[row.task_index];
        let y_center = chart.row_top(row_index) + ROW_H * 0.5;
        if task.milestone {
            draw_milestone(
                painter,
                pos2(chart.date_to_x(task.start), y_center),
                task.number == selected_task_id,
            );
        } else if task.summary {
            draw_summary_bar(painter, chart, task, y_center);
        } else {
            draw_normal_bar(
                painter,
                chart,
                task,
                y_center,
                task.number == selected_task_id,
            );
        }
    }
}

fn hit_test_task_bar(
    chart: &ChartGeometry,
    tasks: &[ProjectTask],
    visible_rows: &[VisibleTaskRow],
    pointer: Pos2,
) -> Option<BarHit> {
    for (_row_index, row) in visible_rows.iter().enumerate().rev() {
        let index = row.task_index;
        let task = &tasks[index];
        if task.summary {
            continue;
        }

        if task.milestone {
            let center = pos2(
                chart.date_to_x(task.start),
                chart.row_top(index) + ROW_H * 0.5,
            );
            let rect = Rect::from_center_size(
                center,
                vec2(MILESTONE_SIZE + BAR_HIT_PAD, MILESTONE_SIZE + BAR_HIT_PAD),
            );
            if rect.contains(pointer) {
                return Some(BarHit {
                    task_index: index,
                    action: DragAction::Move,
                    pointer,
                });
            }
            continue;
        }

        let rect = task_bar_rect(chart, index, task).expand(BAR_HIT_PAD);
        if !rect.contains(pointer) {
            continue;
        }

        let raw_rect = task_bar_rect(chart, index, task);
        let completed_x = raw_rect.left() + raw_rect.width() * task.progress.clamp(0.0, 1.0);
        let progress_handle = Rect::from_center_size(
            pos2(completed_x, raw_rect.center().y),
            vec2(BAR_HANDLE_W * 2.0, BAR_H + BAR_HIT_PAD),
        );
        let action =
            if task.progress > 0.0 && task.progress < 1.0 && progress_handle.contains(pointer) {
                DragAction::Progress
            } else if pointer.x <= raw_rect.left() + BAR_HANDLE_W {
                DragAction::ResizeStart
            } else if pointer.x >= raw_rect.right() - BAR_HANDLE_W {
                DragAction::ResizeEnd
            } else {
                DragAction::Move
            };

        return Some(BarHit {
            task_index: index,
            action,
            pointer,
        });
    }

    None
}

fn task_bar_rect(chart: &ChartGeometry, index: usize, task: &ProjectTask) -> Rect {
    task_bar_rect_for_dates_at_y(
        chart,
        task.start,
        task.finish,
        chart.row_top(index) + ROW_H * 0.5,
    )
}

fn task_bar_rect_for_dates_at_y(
    chart: &ChartGeometry,
    start: NaiveDate,
    finish: NaiveDate,
    y_center: f32,
) -> Rect {
    let x0 = chart.date_to_x(start);
    let x1 = chart.date_to_x(finish + Duration::days(1));
    Rect::from_min_max(
        pos2(x0, y_center - BAR_H * 0.5),
        pos2(x1.max(x0 + 7.0), y_center + BAR_H * 0.5),
    )
}

fn draw_normal_bar(
    painter: &Painter,
    chart: &ChartGeometry,
    task: &ProjectTask,
    y_center: f32,
    selected: bool,
) {
    let rect = task_bar_rect_for_dates_at_y(chart, task.start, task.finish, y_center);
    let fill = if selected {
        Color32::from_rgb(54, 112, 178)
    } else {
        Color32::from_rgb(76, 137, 204)
    };

    painter.rect_filled(rect, 1.0, fill);
    painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0, Color32::from_rgb(31, 75, 125)),
        egui::StrokeKind::Outside,
    );

    if task.progress > 0.0 {
        let progress_w = rect.width() * task.progress.clamp(0.0, 1.0);
        let progress_rect = Rect::from_min_size(
            pos2(rect.left(), rect.center().y - 2.0),
            vec2(progress_w, 4.0),
        );
        painter.rect_filled(progress_rect, 0.0, Color32::from_rgb(12, 12, 12));
        if selected && task.progress < 1.0 {
            painter.circle_filled(
                pos2(progress_rect.right(), rect.center().y),
                3.5,
                Color32::from_rgb(12, 12, 12),
            );
        }
    }

    if task.progress < 1.0 {
        painter.text(
            pos2(rect.right() + 7.0, rect.center().y),
            Align2::LEFT_CENTER,
            format!("{}%", (task.progress * 100.0).round() as i32),
            FontId::new(13.0, FontFamily::Proportional),
            Color32::from_rgb(76, 76, 76),
        );
    }
}

fn draw_summary_bar(painter: &Painter, chart: &ChartGeometry, task: &ProjectTask, y_center: f32) {
    let x0 = chart.date_to_x(task.start);
    let x1 = chart.date_to_x(task.finish + Duration::days(1));
    let y = y_center - SUMMARY_H * 0.5;
    let rect = Rect::from_min_max(pos2(x0, y), pos2(x1.max(x0 + 12.0), y + SUMMARY_H));

    painter.rect_filled(rect, 0.0, Color32::from_rgb(38, 38, 38));
    painter.add(Shape::convex_polygon(
        vec![
            pos2(rect.left(), rect.bottom()),
            pos2(rect.left() + 8.0, rect.bottom()),
            pos2(rect.left(), rect.bottom() + 8.0),
        ],
        Color32::from_rgb(38, 38, 38),
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![
            pos2(rect.right(), rect.bottom()),
            pos2(rect.right() - 8.0, rect.bottom()),
            pos2(rect.right(), rect.bottom() + 8.0),
        ],
        Color32::from_rgb(38, 38, 38),
        Stroke::NONE,
    ));
}

fn draw_milestone(painter: &Painter, center: Pos2, selected: bool) {
    let half = MILESTONE_SIZE * 0.5;
    let fill = if selected {
        Color32::from_rgb(34, 89, 151)
    } else {
        Color32::from_rgb(72, 72, 72)
    };
    painter.add(Shape::convex_polygon(
        vec![
            pos2(center.x, center.y - half),
            pos2(center.x + half, center.y),
            pos2(center.x, center.y + half),
            pos2(center.x - half, center.y),
        ],
        fill,
        Stroke::new(1.0, Color32::from_rgb(28, 28, 28)),
    ));
}

fn draw_dependency_links(
    painter: &Painter,
    chart: &ChartGeometry,
    tasks: &[ProjectTask],
    visible_rows: &[VisibleTaskRow],
) {
    let visible_positions: std::collections::HashMap<usize, usize> = visible_rows
        .iter()
        .enumerate()
        .map(|(row_index, row)| (row.task_index, row_index))
        .collect();

    for (index, task) in tasks.iter().enumerate() {
        for predecessor_number in &task.predecessors {
            let Some(from_index) = tasks
                .iter()
                .position(|candidate| candidate.number == *predecessor_number)
            else {
                continue;
            };
            let Some(&from_row) = visible_positions.get(&from_index) else {
                continue;
            };
            let Some(&to_row) = visible_positions.get(&index) else {
                continue;
            };
            let from = &tasks[from_index];
            let x0 = chart.date_to_x(from.finish + Duration::days(1));
            let y0 = chart.row_top(from_row) + ROW_H * 0.5;
            let x1 = chart.date_to_x(task.start);
            let y1 = chart.row_top(to_row) + ROW_H * 0.5;
            let mid_x = (x0 + 10.0).max(x1 - 14.0);

            let stroke = Stroke::new(1.0, Color32::from_rgb(92, 92, 92));
            painter.line_segment([pos2(x0, y0), pos2(mid_x, y0)], stroke);
            painter.line_segment([pos2(mid_x, y0), pos2(mid_x, y1)], stroke);
            painter.line_segment([pos2(mid_x, y1), pos2(x1 - 6.0, y1)], stroke);
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(x1 - 6.0, y1 - 4.0),
                    pos2(x1, y1),
                    pos2(x1 - 6.0, y1 + 4.0),
                ],
                Color32::from_rgb(92, 92, 92),
                Stroke::NONE,
            ));
        }
    }
}

fn draw_vertical_table_lines(
    painter: &Painter,
    left: f32,
    top: f32,
    bottom: f32,
    color: Color32,
    table_width: f32,
) {
    for x in [
        left + LEFT_ROW_NO_W,
        left + LEFT_ROW_NO_W + LEFT_WBS_W,
        left + LEFT_ROW_NO_W + LEFT_WBS_W + LEFT_ICON_W,
        left + table_width - LEFT_DURATION_W,
    ] {
        painter.line_segment([pos2(x, top), pos2(x, bottom)], Stroke::new(1.0, color));
    }
}

fn draw_expand_box(painter: &Painter, center: Pos2, inverted: bool, expanded: bool) {
    let rect = Rect::from_center_size(center, vec2(11.0, 11.0));
    let fill = if inverted {
        Color32::from_rgb(110, 110, 110)
    } else {
        Color32::from_rgb(245, 245, 245)
    };
    let stroke = if inverted {
        Color32::from_rgb(255, 255, 255)
    } else {
        Color32::from_rgb(50, 50, 50)
    };

    painter.rect_filled(rect, 1.0, fill);
    painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0, stroke),
        egui::StrokeKind::Outside,
    );
    painter.line_segment(
        [
            pos2(rect.left() + 3.0, rect.center().y),
            pos2(rect.right() - 3.0, rect.center().y),
        ],
        Stroke::new(1.5, stroke),
    );
    if !expanded {
        painter.line_segment(
            [
                pos2(rect.center().x, rect.top() + 3.0),
                pos2(rect.center().x, rect.bottom() - 3.0),
            ],
            Stroke::new(1.5, stroke),
        );
    }
}

fn draw_milestone_icon(painter: &Painter, center: Pos2, inverted: bool) {
    let half = 5.0;
    let fill = if inverted {
        Color32::from_rgb(255, 255, 255)
    } else {
        Color32::from_rgb(85, 85, 85)
    };
    painter.add(Shape::convex_polygon(
        vec![
            pos2(center.x, center.y - half),
            pos2(center.x + half, center.y),
            pos2(center.x, center.y + half),
            pos2(center.x - half, center.y),
        ],
        fill,
        Stroke::NONE,
    ));
}

fn month_spans(start: NaiveDate, end: NaiveDate) -> Vec<(String, NaiveDate, NaiveDate)> {
    let mut spans = Vec::new();
    let mut cursor = start;
    while cursor <= end {
        let month_start = cursor;
        let mut month_end = cursor;
        while month_end + Duration::days(1) <= end
            && (month_end + Duration::days(1)).month() == month_start.month()
        {
            month_end += Duration::days(1);
        }

        spans.push((
            format!(
                "{} {}",
                month_label(month_start.month()),
                month_start.year()
            ),
            month_start,
            month_end,
        ));
        cursor = month_end + Duration::days(1);
    }
    spans
}

fn month_label(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "",
    }
}

fn json_save_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let path = Path::new(trimmed);
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("mpp"))
        .unwrap_or(false)
    {
        path.with_extension("json").display().to_string()
    } else {
        trimmed.to_string()
    }
}

fn build_visible_rows(
    tasks: &[ProjectTask],
    collapsed_summaries: &HashSet<usize>,
) -> Vec<VisibleTaskRow> {
    let mut rows = Vec::new();
    let mut hidden_until_indent: Option<usize> = None;

    for (task_index, task) in tasks.iter().enumerate() {
        if let Some(indent) = hidden_until_indent {
            if task.indent > indent {
                continue;
            }
            hidden_until_indent = None;
        }

        rows.push(VisibleTaskRow { task_index });

        if task.summary && collapsed_summaries.contains(&task.number) {
            hidden_until_indent = Some(task.indent);
        }
    }

    rows
}

fn build_wbs_codes(tasks: &[ProjectTask]) -> Vec<String> {
    let mut codes = Vec::with_capacity(tasks.len());
    let mut counts: Vec<usize> = Vec::new();

    for task in tasks {
        let level = task.indent;
        if counts.len() <= level {
            counts.resize(level + 1, 0);
        } else {
            counts.truncate(level + 1);
        }
        counts[level] += 1;
        let code = counts[..=level]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(".");
        codes.push(code);
    }

    codes
}

fn subtree_end_index(tasks: &[ProjectTask], index: usize) -> usize {
    let indent = tasks[index].indent;
    let mut end = index + 1;
    while end < tasks.len() && tasks[end].indent > indent {
        end += 1;
    }
    end - 1
}

fn subtree_task_ids(tasks: &[ProjectTask], index: usize) -> HashSet<usize> {
    let mut ids = HashSet::new();
    let end = subtree_end_index(tasks, index);
    for task in &tasks[index..=end] {
        ids.insert(task.number);
    }
    ids
}

fn hit_test_left_row(
    chart: &ChartGeometry,
    tasks: &[ProjectTask],
    visible_rows: &[VisibleTaskRow],
    pointer: Pos2,
) -> Option<(VisibleTaskRow, bool)> {
    let row_index = chart.row_at(pointer, visible_rows.len())?;
    let row = visible_rows[row_index];
    let task = &tasks[row.task_index];
    let y = chart.row_top(row_index);
    let name_x =
        chart.origin_x + LEFT_ROW_NO_W + LEFT_WBS_W + LEFT_ICON_W + 8.0 + task.indent as f32 * 18.0;
    let toggle_rect = Rect::from_center_size(pos2(name_x, y + ROW_H * 0.5), vec2(20.0, 18.0));
    Some((row, task.summary && toggle_rect.contains(pointer)))
}

#[derive(Clone, Copy)]
struct VisibleTaskRow {
    task_index: usize,
}

struct ChartGeometry {
    gantt_left: f32,
    rows_top: f32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    day_width: f32,
    origin_x: f32,
}

impl ChartGeometry {
    fn new(rect: Rect, snapshot: &ProjectSnapshot, day_width: f32, left_table_width: f32) -> Self {
        Self {
            gantt_left: rect.left() + left_table_width + SPLITTER_W + CHART_MARGIN_X,
            rows_top: rect.top() + HEADER_H,
            start_date: snapshot.start_date,
            end_date: snapshot.end_date,
            day_width,
            origin_x: rect.left(),
        }
    }

    fn date_to_x(&self, date: NaiveDate) -> f32 {
        self.gantt_left + (date - self.start_date).num_days() as f32 * self.day_width
    }

    fn row_top(&self, index: usize) -> f32 {
        self.rows_top + index as f32 * ROW_H
    }

    fn row_at(&self, point: Pos2, row_count: usize) -> Option<usize> {
        if point.y < self.rows_top {
            return None;
        }

        let row = ((point.y - self.rows_top) / ROW_H).floor() as usize;
        (row < row_count).then_some(row)
    }

    fn pixel_delta_to_days(&self, delta_x: f32) -> i64 {
        (delta_x / self.day_width.max(1.0)).round() as i64
    }
}

impl GanttApp {
    fn content_width(&self, chart: &ChartGeometry) -> f32 {
        let duration_days = (chart.end_date - chart.start_date).num_days().max(0) as f32 + 1.0;
        self.left_table_width
            + SPLITTER_W
            + CHART_MARGIN_X * 2.0
            + duration_days * self.day_width
            + 240.0
    }

    fn content_height(&self, visible_rows: usize) -> f32 {
        HEADER_H + visible_rows as f32 * ROW_H + 160.0
    }
}

#[derive(Clone, Copy)]
enum DragAction {
    Move,
    ResizeStart,
    ResizeEnd,
    Progress,
}

#[derive(Clone)]
struct DragState {
    task_index: usize,
    task_id: usize,
    action: DragAction,
    origin_pointer: Pos2,
    original_start: NaiveDate,
    original_finish: NaiveDate,
    history_snapshot: ProjectDocument,
    changed: bool,
}

impl DragState {
    fn to_edit(&self, chart: &ChartGeometry, pointer: Pos2) -> Option<EditCommand> {
        let delta_days = chart.pixel_delta_to_days(pointer.x - self.origin_pointer.x);
        match self.action {
            DragAction::Move => Some(EditCommand::Move {
                id: self.task_id,
                start: self.original_start + Duration::days(delta_days),
                finish: self.original_finish + Duration::days(delta_days),
            }),
            DragAction::ResizeStart => Some(EditCommand::ResizeStart {
                id: self.task_id,
                start: self.original_start + Duration::days(delta_days),
            }),
            DragAction::ResizeEnd => Some(EditCommand::ResizeEnd {
                id: self.task_id,
                finish: self.original_finish + Duration::days(delta_days),
            }),
            DragAction::Progress => {
                let bar = task_bar_rect_for_dates_at_y(
                    chart,
                    self.original_start,
                    self.original_finish,
                    chart.row_top(self.task_index) + ROW_H * 0.5,
                );
                let progress = ((pointer.x - bar.left()) / bar.width().max(1.0)).clamp(0.0, 1.0);
                Some(EditCommand::SetProgress {
                    id: self.task_id,
                    progress,
                })
            }
        }
    }
}

#[derive(Clone, Copy)]
struct BarHit {
    task_index: usize,
    action: DragAction,
    pointer: Pos2,
}
