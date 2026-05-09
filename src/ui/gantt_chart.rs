use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};
use eframe::egui::{
    pos2, vec2, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke,
};

use crate::model::TaskSnapshot;
use crate::ui::gantt_view::{TimelineGeometry, VisibleTaskRow, DAY_H, HEADER_H, MONTH_H, ROW_H};

const BAR_H: f32 = 14.0;
const SUMMARY_H: f32 = 10.0;
const MILESTONE_SIZE: f32 = 16.0;
const BAR_HANDLE_W: f32 = 7.0;
const BAR_HIT_PAD: f32 = 4.0;
const BAR_LABEL_PAD: f32 = 7.0;

#[derive(Clone, Copy)]
pub enum DragAction {
    Move,
    ResizeStart,
    ResizeEnd,
    Progress,
}

#[derive(Clone, Copy)]
pub struct BarHit {
    pub task_index: usize,
    pub action: DragAction,
    pub pointer: Pos2,
}

pub fn draw_timeline_headers(painter: &Painter, rect: Rect, chart: &TimelineGeometry) {
    let border = Color32::from_rgb(175, 175, 175);
    let header_rect = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER_H));
    painter.rect_filled(header_rect, 0.0, Color32::from_rgb(238, 238, 238));
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
            eframe::egui::StrokeKind::Outside,
        );
        painter.text(
            month_rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::new(15.0, FontFamily::Proportional),
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
        eframe::egui::StrokeKind::Outside,
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
            FontId::new(14.0, FontFamily::Proportional),
            Color32::from_rgb(34, 34, 34),
        );
        day += Duration::days(1);
    }
}

pub fn draw_rows_and_grid(
    painter: &Painter,
    rect: Rect,
    chart: &TimelineGeometry,
    tasks: &[TaskSnapshot],
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

    for (row_index, row) in visible_rows.iter().enumerate() {
        let task = &tasks[row.task_index];
        let y = chart.row_top(row_index);
        if task.number == selected_task_id {
            painter.rect_filled(
                Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), ROW_H)),
                0.0,
                Color32::from_rgba_premultiplied(90, 120, 150, 28),
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

pub fn draw_task_bars(
    painter: &Painter,
    chart: &TimelineGeometry,
    tasks: &[TaskSnapshot],
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
            draw_bar_label(
                painter,
                chart,
                task,
                pos2(chart.date_to_x(task.start) + 12.0, y_center),
                task.number == selected_task_id,
            );
        } else if task.summary {
            let rect = draw_summary_bar(painter, chart, task, y_center);
            draw_bar_label(
                painter,
                chart,
                task,
                pos2(rect.right() + BAR_LABEL_PAD, y_center),
                false,
            );
        } else {
            let rect = draw_normal_bar(
                painter,
                chart,
                task,
                y_center,
                task.number == selected_task_id,
            );
            draw_bar_label(
                painter,
                chart,
                task,
                pos2(rect.right() + BAR_LABEL_PAD, y_center),
                task.number == selected_task_id,
            );
        }
    }
}

pub fn hit_test_task_bar(
    chart: &TimelineGeometry,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
    pointer: Pos2,
) -> Option<BarHit> {
    let visible_positions: HashMap<usize, usize> = visible_rows
        .iter()
        .enumerate()
        .map(|(row_index, row)| (row.task_index, row_index))
        .collect();

    for row in visible_rows.iter().rev() {
        let index = row.task_index;
        let row_index = *visible_positions.get(&index)?;
        let task = &tasks[index];
        if task.summary {
            continue;
        }

        if task.milestone {
            let center = pos2(
                chart.date_to_x(task.start),
                chart.row_top(row_index) + ROW_H * 0.5,
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

        let rect = task_bar_rect_at_row(chart, row_index, task).expand(BAR_HIT_PAD);
        if !rect.contains(pointer) {
            continue;
        }

        let raw_rect = task_bar_rect_at_row(chart, row_index, task);
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

pub fn task_bar_rect_for_dates_at_y(
    chart: &TimelineGeometry,
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

fn task_bar_rect_at_row(chart: &TimelineGeometry, row_index: usize, task: &TaskSnapshot) -> Rect {
    task_bar_rect_for_dates_at_y(
        chart,
        task.start,
        task.finish,
        chart.row_top(row_index) + ROW_H * 0.5,
    )
}

fn draw_normal_bar(
    painter: &Painter,
    chart: &TimelineGeometry,
    task: &TaskSnapshot,
    y_center: f32,
    selected: bool,
) -> Rect {
    let rect = task_bar_rect_for_dates_at_y(chart, task.start, task.finish, y_center);
    let fill = if selected {
        Color32::from_rgb(45, 102, 170)
    } else {
        Color32::from_rgb(76, 137, 204)
    };

    painter.rect_filled(rect, 1.0, fill);
    painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0, Color32::from_rgb(31, 75, 125)),
        eframe::egui::StrokeKind::Outside,
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

    rect
}

fn draw_summary_bar(
    painter: &Painter,
    chart: &TimelineGeometry,
    task: &TaskSnapshot,
    y_center: f32,
) -> Rect {
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

    rect
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

fn draw_bar_label(
    painter: &Painter,
    chart: &TimelineGeometry,
    task: &TaskSnapshot,
    anchor: Pos2,
    selected: bool,
) {
    let label_rect = Rect::from_min_max(
        pos2(chart.gantt_left, anchor.y - ROW_H * 0.5),
        pos2(chart.gantt_left + 1000.0, anchor.y + ROW_H * 0.5),
    );
    let clipped = painter.with_clip_rect(label_rect);
    clipped.text(
        anchor,
        Align2::LEFT_CENTER,
        task.name.as_str(),
        FontId::new(13.0, FontFamily::Proportional),
        if selected {
            Color32::from_rgb(20, 20, 20)
        } else {
            Color32::from_rgb(35, 35, 35)
        },
    );
}

pub fn draw_dependency_links(
    painter: &Painter,
    chart: &TimelineGeometry,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
) {
    let visible_positions: HashMap<usize, usize> = visible_rows
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
