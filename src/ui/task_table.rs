use std::collections::HashSet;

use eframe::egui::{pos2, vec2, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Stroke};

use crate::model::TaskSnapshot;
use crate::ui::gantt_view::{VisibleTaskRow, HEADER_H, ROW_H};
use crate::ui::icons::{IconKey, ProjectLibreIcons};

pub const ROWNUM_W: f32 = 40.0;
pub const DEFAULT_TABLE_W: f32 = 614.0;

const NAME_ICON_W: f32 = 16.0;
const NAME_TEXT_PAD: f32 = 2.0;
const NAME_LEADING_PAD: f32 = 16.0;
const INDICATOR_SIZE: f32 = 12.0;
const INDICATOR_GAP: f32 = 2.0;
const INDICATORS_W: f32 = 50.0;

const COLUMN_SPECS: [Column; 7] = [
    Column {
        field_name: "indicators",
        label: "",
        width: 50.0,
    },
    Column {
        field_name: "name",
        label: "名前",
        width: 150.0,
    },
    Column {
        field_name: "duration",
        label: "期間",
        width: 60.0,
    },
    Column {
        field_name: "start",
        label: "開始",
        width: 80.0,
    },
    Column {
        field_name: "finish",
        label: "終了",
        width: 80.0,
    },
    Column {
        field_name: "predecessors",
        label: "先行",
        width: 72.0,
    },
    Column {
        field_name: "resourceNames",
        label: "リソース名",
        width: 82.0,
    },
];

#[derive(Clone, Copy)]
pub struct Column {
    pub field_name: &'static str,
    pub label: &'static str,
    pub width: f32,
}

pub struct ColumnModel {
    pub columns: &'static [Column],
}

impl ColumnModel {
    pub fn new() -> Self {
        Self {
            columns: &COLUMN_SPECS,
        }
    }
}

pub fn draw_headers(painter: &Painter, rect: Rect) {
    let border = Color32::from_rgb(175, 175, 175);
    let header_rect = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER_H));

    painter.rect_filled(header_rect, 0.0, Color32::from_rgb(238, 238, 238));
    painter.rect_stroke(
        header_rect,
        0.0,
        Stroke::new(1.0, border),
        eframe::egui::StrokeKind::Outside,
    );

    let rownum_rect = Rect::from_min_size(rect.min, vec2(ROWNUM_W, HEADER_H));
    painter.line_segment(
        [
            pos2(rownum_rect.right(), rownum_rect.top()),
            pos2(rownum_rect.right(), rownum_rect.bottom()),
        ],
        Stroke::new(1.0, border),
    );

    let column_model = ColumnModel::new();
    let mut x = rect.left() + ROWNUM_W;
    for column in column_model.columns {
        let column_rect = Rect::from_min_size(pos2(x, rect.top()), vec2(column.width, HEADER_H));
        painter.line_segment(
            [
                pos2(column_rect.right(), column_rect.top()),
                pos2(column_rect.right(), column_rect.bottom()),
            ],
            Stroke::new(1.0, border),
        );
        if column.label.is_empty() {
            draw_indicator_header(painter, column_rect.center());
        } else {
            painter.text(
                column_rect.center(),
                Align2::CENTER_CENTER,
                column.label,
                FontId::new(14.0, FontFamily::Proportional),
                Color32::from_rgb(38, 38, 38),
            );
        }
        x += column.width;
    }
}

pub fn draw_rows(
    painter: &Painter,
    rect: Rect,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
    selected_task_id: usize,
    collapsed_summaries: &HashSet<usize>,
    icons: &ProjectLibreIcons,
) {
    let line = Color32::from_rgb(214, 214, 214);
    let column_model = ColumnModel::new();
    for (row_index, row) in visible_rows.iter().enumerate() {
        let task = &tasks[row.task_index];
        let y = rect.top() + HEADER_H + row_index as f32 * ROW_H;
        let row_rect = Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), ROW_H));
        let rownum_rect = Rect::from_min_size(pos2(rect.left(), y), vec2(ROWNUM_W, ROW_H));
        let selected_row = task.number == selected_task_id;
        let bg = if selected_row {
            Color32::from_rgb(64, 64, 64)
        } else if task.summary {
            Color32::from_rgb(246, 246, 246)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        let text = if selected_row {
            Color32::from_rgb(255, 255, 255)
        } else {
            Color32::from_rgb(30, 30, 30)
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
        painter.line_segment(
            [
                pos2(rownum_rect.right(), y),
                pos2(rownum_rect.right(), y + ROW_H),
            ],
            Stroke::new(1.0, line),
        );

        painter.text(
            rownum_rect.center(),
            Align2::CENTER_CENTER,
            (row_index + 1).to_string(),
            FontId::new(13.0, FontFamily::Proportional),
            text,
        );

        let mut x = rect.left() + ROWNUM_W;
        for column in column_model.columns {
            let column_rect = Rect::from_min_size(pos2(x, y), vec2(column.width, ROW_H));
            painter.line_segment(
                [
                    pos2(column_rect.right(), y),
                    pos2(column_rect.right(), y + ROW_H),
                ],
                Stroke::new(1.0, line),
            );
            draw_cell(
                painter,
                column,
                column_rect,
                task,
                selected_row,
                text,
                collapsed_summaries,
                icons,
            );
            x += column.width;
        }
    }
}

pub fn hit_test_row_toggle(
    table_rect: Rect,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
    pointer: Pos2,
) -> Option<(VisibleTaskRow, bool)> {
    if pointer.x < table_rect.left() || pointer.x > table_rect.right() {
        return None;
    }

    if pointer.y < table_rect.top() + HEADER_H {
        return None;
    }

    let row_index = ((pointer.y - table_rect.top() - HEADER_H) / ROW_H).floor() as usize;
    if row_index >= visible_rows.len() {
        return None;
    }
    let row = visible_rows[row_index];
    let task = &tasks[row.task_index];
    if !task.summary {
        return None;
    }

    let icon_rect = summary_toggle_icon_rect(table_rect, row_index, task);
    Some((row, icon_rect.expand(4.0).contains(pointer)))
}

fn draw_cell(
    painter: &Painter,
    column: &Column,
    rect: Rect,
    task: &TaskSnapshot,
    selected: bool,
    color: Color32,
    collapsed_summaries: &HashSet<usize>,
    icons: &ProjectLibreIcons,
) {
    match column.field_name {
        "indicators" => draw_indicators(painter, rect, task, icons),
        "name" => draw_name(
            painter,
            rect,
            task,
            selected,
            color,
            collapsed_summaries,
            icons,
        ),
        "duration" => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.duration_label(),
            Align2::LEFT_CENTER,
            color,
            14.0,
        ),
        "start" => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.start_label(),
            Align2::LEFT_CENTER,
            color,
            13.0,
        ),
        "finish" => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.finish_label(),
            Align2::LEFT_CENTER,
            color,
            13.0,
        ),
        "predecessors" => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.predecessors_label(),
            Align2::LEFT_CENTER,
            color,
            14.0,
        ),
        "resourceNames" => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.resource_names_label(),
            Align2::LEFT_CENTER,
            color,
            14.0,
        ),
        _ => {}
    }
}

fn draw_name(
    painter: &Painter,
    rect: Rect,
    task: &TaskSnapshot,
    selected: bool,
    color: Color32,
    collapsed_summaries: &HashSet<usize>,
    icons: &ProjectLibreIcons,
) {
    let icon_rect = name_icon_rect_from_rect(rect, task);
    let text_x = icon_rect.right() + NAME_TEXT_PAD;
    let clip = painter.with_clip_rect(rect);

    if task.summary {
        draw_tree_icon(
            painter,
            icon_rect,
            if collapsed_summaries.contains(&task.number) {
                IconKey::Plus
            } else {
                IconKey::Minus
            },
            selected,
            icons,
        );
    } else {
        draw_tree_icon(painter, icon_rect, IconKey::Leaf, selected, icons);
    }

    let text = name_text(task);

    clip.text(
        pos2(text_x, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        FontId::new(14.0, FontFamily::Proportional),
        color,
    );
}

fn draw_indicators(painter: &Painter, rect: Rect, task: &TaskSnapshot, icons: &ProjectLibreIcons) {
    let mut x = rect.left() + 2.0;
    let y = rect.center().y - INDICATOR_SIZE * 0.5;

    for indicator in task_indicator_icons(task) {
        if let Some(texture) = icons.texture(indicator) {
            let icon_rect = Rect::from_min_size(pos2(x, y), vec2(INDICATOR_SIZE, INDICATOR_SIZE));
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            x += INDICATOR_SIZE + INDICATOR_GAP;
        }
    }
}

fn task_indicator_icons(task: &TaskSnapshot) -> Vec<IconKey> {
    let mut out = Vec::new();

    // Java orders the indicators from left to right with notes first in the subset we support.
    if task.has_notes() {
        out.push(IconKey::Note);
    }
    if task.progress >= 1.0 {
        out.push(IconKey::Completed);
    }
    if task.summary && !task.resource_names.is_empty() {
        out.push(IconKey::ParentAssignment);
    }
    if task.missed_deadline() {
        out.push(IconKey::MissedDeadline);
    }

    out
}

fn draw_text(
    painter: &Painter,
    rect: Rect,
    text: impl Into<String>,
    align: Align2,
    color: Color32,
    size: f32,
) {
    let painter = painter.with_clip_rect(rect);
    let x = match align {
        Align2::CENTER_CENTER => rect.center().x,
        Align2::LEFT_CENTER => rect.left(),
        _ => rect.left(),
    };
    painter.text(
        pos2(x, rect.center().y),
        align,
        text.into(),
        FontId::new(size, FontFamily::Proportional),
        color,
    );
}

fn draw_tree_icon(
    painter: &Painter,
    rect: Rect,
    icon: IconKey,
    selected: bool,
    icons: &ProjectLibreIcons,
) {
    if let Some(texture) = icons.texture(icon) {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        return;
    }

    let fill = if selected {
        Color32::from_rgb(110, 110, 110)
    } else {
        Color32::from_rgb(245, 245, 245)
    };
    let stroke = if selected {
        Color32::WHITE
    } else {
        Color32::from_rgb(50, 50, 50)
    };
    painter.rect_filled(rect, 1.0, fill);
    painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0, stroke),
        eframe::egui::StrokeKind::Outside,
    );
    painter.line_segment(
        [
            pos2(rect.left() + 3.0, rect.center().y),
            pos2(rect.right() - 3.0, rect.center().y),
        ],
        Stroke::new(1.5, stroke),
    );
    if icon == IconKey::Plus {
        painter.line_segment(
            [
                pos2(rect.center().x, rect.top() + 3.0),
                pos2(rect.center().x, rect.bottom() - 3.0),
            ],
            Stroke::new(1.5, stroke),
        );
    }
}

fn name_icon_rect_from_rect(rect: Rect, task: &TaskSnapshot) -> Rect {
    let x = rect.left() + NAME_LEADING_PAD + task.indent as f32 * NAME_ICON_W;
    let y = rect.center().y - NAME_ICON_W * 0.5;
    Rect::from_min_size(pos2(x, y), vec2(NAME_ICON_W, NAME_ICON_W))
}

pub fn summary_toggle_icon_rect(table_rect: Rect, row_index: usize, task: &TaskSnapshot) -> Rect {
    let x = table_rect.left()
        + ROWNUM_W
        + INDICATORS_W
        + NAME_LEADING_PAD
        + task.indent as f32 * NAME_ICON_W;
    let y =
        table_rect.top() + HEADER_H + row_index as f32 * ROW_H + ROW_H * 0.5 - NAME_ICON_W * 0.5;
    Rect::from_min_size(pos2(x, y), vec2(NAME_ICON_W, NAME_ICON_W))
}

fn draw_indicator_header(painter: &Painter, center: Pos2) {
    painter.circle_stroke(center, 5.0, Stroke::new(1.2, Color32::from_rgb(80, 80, 80)));
    painter.line_segment(
        [
            pos2(center.x, center.y - 2.5),
            pos2(center.x, center.y + 2.0),
        ],
        Stroke::new(1.1, Color32::from_rgb(80, 80, 80)),
    );
}

fn name_text(task: &TaskSnapshot) -> &str {
    if task.name.trim().is_empty() {
        " "
    } else {
        task.name.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn task(
        progress: f32,
        summary: bool,
        resource_names: Vec<&str>,
        notes: Option<&str>,
        deadline: Option<&str>,
    ) -> TaskSnapshot {
        TaskSnapshot {
            number: 1,
            name: "Task".to_string(),
            start: NaiveDate::from_ymd_opt(2025, 2, 3).expect("date"),
            finish: NaiveDate::from_ymd_opt(2025, 2, 4).expect("date"),
            progress,
            indent: 0,
            summary,
            milestone: false,
            predecessors: vec![],
            resource_names: resource_names
                .into_iter()
                .map(|value| value.to_string())
                .collect(),
            start_text: None,
            finish_text: None,
            duration_text: None,
            notes: notes.map(|value| value.to_string()),
            deadline: deadline
                .map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("date")),
        }
    }

    #[test]
    fn indicators_follow_supported_java_order() {
        let icons = task_indicator_icons(&task(
            1.0,
            true,
            vec!["Alice", "Bob"],
            Some("note"),
            Some("2025-02-03"),
        ));

        assert_eq!(
            icons,
            vec![
                IconKey::Note,
                IconKey::Completed,
                IconKey::ParentAssignment,
                IconKey::MissedDeadline,
            ]
        );
    }

    #[test]
    fn indicators_stay_empty_when_java_would_stay_empty() {
        let icons = task_indicator_icons(&task(0.0, false, vec![], None, None));
        assert!(icons.is_empty());
    }

    #[test]
    fn name_icon_rect_keeps_the_java_style_leading_gap() {
        let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(200.0, ROW_H));
        let task = task(0.0, false, vec![], None, None);
        let icon_rect = summary_toggle_icon_rect(rect, 0, &task);

        assert_eq!(icon_rect.left(), 116.0);
        assert_eq!(icon_rect.right(), 132.0);
    }

    #[test]
    fn empty_names_render_as_a_visible_blank() {
        let task = TaskSnapshot {
            name: "   ".to_string(),
            ..task(0.0, false, vec![], None, None)
        };

        assert_eq!(name_text(&task), " ");
    }

    #[test]
    fn row_toggle_uses_the_same_icon_hit_box_expansion_as_java() {
        let snapshot = crate::model::ProjectSnapshot {
            start_date: NaiveDate::from_ymd_opt(2025, 2, 3).expect("date"),
            end_date: NaiveDate::from_ymd_opt(2025, 2, 4).expect("date"),
            status_date: None,
            tasks: vec![task(0.0, true, vec![], None, None)],
        };
        let visible_rows = vec![VisibleTaskRow { task_index: 0 }];
        let hit = hit_test_row_toggle(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0)),
            &snapshot.tasks,
            &visible_rows,
            pos2(ROWNUM_W + INDICATORS_W + NAME_LEADING_PAD + 2.0, HEADER_H + 2.0),
        );

        assert_eq!(
            hit.map(|(row, toggle)| (row.task_index, toggle)),
            Some((0, true))
        );
    }
}
