use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};
use eframe::egui::{
    pos2, vec2, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke,
};

use crate::model::{DependencyRelation, TaskSnapshot};
use crate::ui::gantt_view::{DragPreview, TimelineGeometry, VisibleTaskRow, HEADER_H, ROW_H};

const BAR_H: f32 = 11.0;
const MILESTONE_SIZE: f32 = 11.0;
const BAR_Y_OFFSET: f32 = 4.0;
const BAR_HANDLE_W: f32 = 7.0;
const BAR_HIT_PAD: f32 = 4.0;
const BAR_LABEL_PAD: f32 = 7.0;
const BAR_MIN_W: f32 = 0.0;

const PROGRESS_BAR_H: f32 = 4.0;
const ANNOTATION_X_OFFSET: f32 = 12.0;
const ANNOTATION_Y_OFFSET: f32 = 1.0;

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
    let top_y = rect.top();
    let mid_y = rect.top() + HEADER_H * 0.5;
    let bottom_y = rect.bottom();
    let text_color = Color32::from_rgb(34, 34, 34);

    painter.line_segment(
        [pos2(rect.left(), top_y), pos2(rect.right(), top_y)],
        Stroke::new(1.0, border),
    );
    painter.line_segment(
        [pos2(rect.left(), mid_y), pos2(rect.right(), mid_y)],
        Stroke::new(1.0, border),
    );
    painter.line_segment(
        [pos2(rect.left(), bottom_y), pos2(rect.right(), bottom_y)],
        Stroke::new(1.0, border),
    );

    let mut day = chart.start_date;
    let mut last_month_key = None;
    while day <= chart.end_date {
        let day_start_x = chart.date_to_x(day);
        let next_day = day + Duration::days(1);
        let day_end_x = chart.date_to_x(next_day);
        let is_weekend = matches!(day.weekday().number_from_monday(), 6 | 7);
        let day_stroke = if is_weekend {
            Stroke::new(1.0, Color32::from_rgb(210, 210, 210))
        } else {
            Stroke::new(1.0, Color32::from_rgb(226, 226, 226))
        };

        painter.line_segment(
            [pos2(day_start_x, mid_y), pos2(day_start_x, bottom_y)],
            day_stroke,
        );
        painter.line_segment(
            [pos2(day_end_x, mid_y), pos2(day_end_x, bottom_y)],
            day_stroke,
        );
        painter.text(
            pos2(day_start_x + 2.0, mid_y + 1.0),
            Align2::LEFT_TOP,
            format!("{:02}", day.day()),
            FontId::new(13.0, FontFamily::Proportional),
            text_color,
        );

        let month_key = (day.year(), day.month());
        if last_month_key != Some(month_key) {
            painter.line_segment(
                [pos2(day_start_x, top_y), pos2(day_start_x, mid_y)],
                Stroke::new(1.0, border),
            );
            let mut month_end = day;
            while month_end < chart.end_date
                && (month_end + Duration::days(1)).month() == day.month()
            {
                month_end += Duration::days(1);
            }
            let month_end = month_end + Duration::days(1);
            let month_end_x = chart.date_to_x(month_end);
            painter.line_segment(
                [pos2(month_end_x, top_y), pos2(month_end_x, mid_y)],
                Stroke::new(1.0, border),
            );
            painter.text(
                pos2(day_start_x + 2.0, top_y + 1.0),
                Align2::LEFT_TOP,
                format!("{} {}", month_label(day.month()), day.year()),
                FontId::new(13.0, FontFamily::Proportional),
                text_color,
            );
            last_month_key = Some(month_key);
        }

        day = next_day;
    }
}

pub fn draw_rows_and_grid(
    painter: &Painter,
    rect: Rect,
    chart: &TimelineGeometry,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
    selected_task_id: usize,
    status_date: Option<NaiveDate>,
) {
    let data_rect = Rect::from_min_max(pos2(rect.left(), rect.top() + HEADER_H), rect.max);
    let painter = painter.with_clip_rect(data_rect);
    let line = Color32::from_rgb(224, 224, 224);

    painter.rect_filled(data_rect, 0.0, Color32::from_rgb(250, 250, 250));

    let mut day = chart.start_date;
    while day <= chart.end_date {
        let x = chart.date_to_x(day);
        let weekday = day.weekday().number_from_monday();
        if weekday >= 6 {
            let weekend_rect = Rect::from_min_size(
                pos2(x, data_rect.top()),
                vec2(chart.day_width, data_rect.height()),
            );
            painter.rect_filled(weekend_rect, 0.0, Color32::from_rgb(244, 244, 244));
        }

        let stroke = if day.day() == 1 || day.day() == 15 {
            Stroke::new(1.0, Color32::from_rgb(200, 200, 200))
        } else {
            Stroke::new(1.0, Color32::from_rgb(232, 232, 232))
        };
        painter.line_segment(
            [pos2(x, data_rect.top()), pos2(x, data_rect.bottom())],
            stroke,
        );
        day += Duration::days(1);
    }

    for (row_index, row) in visible_rows.iter().enumerate() {
        let task = &tasks[row.task_index];
        let y = chart.row_top(row_index);
        if task.number == selected_task_id {
            painter.rect_filled(
                Rect::from_min_size(pos2(data_rect.left(), y), vec2(data_rect.width(), ROW_H)),
                0.0,
                Color32::from_rgba_premultiplied(90, 120, 150, 28),
            );
        }
        painter.line_segment(
            [pos2(data_rect.left(), y), pos2(data_rect.right(), y)],
            Stroke::new(1.0, line),
        );
        painter.line_segment(
            [
                pos2(data_rect.left(), y + ROW_H),
                pos2(data_rect.right(), y + ROW_H),
            ],
            Stroke::new(1.0, line),
        );
    }

    let project_start_x = chart.date_to_x(chart.start_date);
    painter.line_segment(
        [
            pos2(project_start_x, data_rect.top()),
            pos2(project_start_x, data_rect.bottom()),
        ],
        Stroke::new(1.0, Color32::from_rgb(155, 196, 155)),
    );

    if let Some(status_date) = status_date {
        let status_x = chart.date_to_x(status_date);
        painter.line_segment(
            [
                pos2(status_x, data_rect.top()),
                pos2(status_x, data_rect.bottom()),
            ],
            Stroke::new(1.0, Color32::from_rgb(74, 192, 74)),
        );
    }
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
        let y_center = bar_center_for_row(chart, row_index);
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
                pos2(
                    chart.date_to_x(task.start) + ANNOTATION_X_OFFSET,
                    y_center + ANNOTATION_Y_OFFSET,
                ),
                task.number == selected_task_id,
                BarAnnotation::Milestone,
            );
        } else if task.summary {
            let _rect = draw_summary_bar(painter, chart, task, y_center);
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
                BarAnnotation::Normal,
            );
        }
    }
}

pub fn draw_drag_preview(
    painter: &Painter,
    chart: &TimelineGeometry,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
    drag: &DragPreview,
) {
    let Some(row_index) = visible_rows
        .iter()
        .enumerate()
        .find_map(|(row_index, row)| (row.task_index == drag.task_index).then_some(row_index))
    else {
        return;
    };

    let task = &tasks[drag.task_index];
    let delta_days = chart.pixel_delta_to_days(drag.current_pointer.x - drag.origin_pointer.x);
    let dx = delta_days as f32 * chart.day_width;
    let y_center = bar_center_for_row(chart, row_index);
    let shadow = Color32::from_rgba_premultiplied(180, 40, 40, 72);
    let stroke = Stroke::new(2.0, Color32::from_rgb(170, 40, 40));

    if task.milestone {
        let center = pos2(chart.date_to_x(task.start) + dx, y_center);
        let half = MILESTONE_SIZE * 0.5;
        painter.add(Shape::convex_polygon(
            vec![
                pos2(center.x, center.y - half),
                pos2(center.x + half, center.y),
                pos2(center.x, center.y + half),
                pos2(center.x - half, center.y),
            ],
            shadow,
            stroke,
        ));
        return;
    }

    let original = task_bar_rect_for_dates_at_y(chart, task.start, task.finish, y_center);
    let rect = match drag.action {
        DragAction::Move => original.translate(vec2(dx, 0.0)),
        DragAction::ResizeStart => {
            let left = (original.left() + dx).min(original.right() - 1.0);
            Rect::from_min_max(pos2(left, original.top()), original.right_bottom())
        }
        DragAction::ResizeEnd => {
            let right = (original.right() + dx).max(original.left() + 1.0);
            Rect::from_min_max(original.left_top(), pos2(right, original.bottom()))
        }
        DragAction::Progress => {
            let progress = ((drag.current_pointer.x - original.left()) / original.width().max(1.0))
                .clamp(0.0, 1.0);
            let completed_w = original.width() * progress;
            Rect::from_min_size(
                original.left_top(),
                vec2(completed_w.max(0.0), original.height()),
            )
        }
    };

    painter.rect_filled(rect, 1.0, shadow);
    painter.rect_stroke(rect, 1.0, stroke, eframe::egui::StrokeKind::Outside);
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
                bar_center_for_row(chart, row_index),
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
        pos2(x1.max(x0 + BAR_MIN_W), y_center + BAR_H * 0.5),
    )
}

fn task_bar_rect_at_row(chart: &TimelineGeometry, row_index: usize, task: &TaskSnapshot) -> Rect {
    task_bar_rect_for_dates_at_y(
        chart,
        task.start,
        task.finish,
        bar_center_for_row(chart, row_index),
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
            pos2(rect.left(), rect.center().y - PROGRESS_BAR_H * 0.5),
            vec2(progress_w, PROGRESS_BAR_H),
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
            pos2(
                rect.right() + ANNOTATION_X_OFFSET,
                rect.center().y + ANNOTATION_Y_OFFSET,
            ),
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
    let y = y_center - BAR_H * 0.5;
    let rect = Rect::from_min_max(pos2(x0, y), pos2(x1.max(x0 + 12.0), y + BAR_H));
    let body = rect.shrink2(vec2(1.0, 1.0));

    painter.rect_filled(body, 0.0, Color32::from_rgb(38, 38, 38));
    painter.add(Shape::convex_polygon(
        vec![
            pos2(rect.left(), rect.center().y),
            pos2(rect.left() + 8.0, rect.top()),
            pos2(rect.left() + 8.0, rect.bottom()),
        ],
        Color32::from_rgb(38, 38, 38),
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![
            pos2(rect.right(), rect.center().y),
            pos2(rect.right() - 8.0, rect.top()),
            pos2(rect.right() - 8.0, rect.bottom()),
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

fn bar_center_for_row(chart: &TimelineGeometry, row_index: usize) -> f32 {
    chart.row_top(row_index) + BAR_Y_OFFSET + BAR_H * 0.5
}

fn draw_bar_label(
    painter: &Painter,
    chart: &TimelineGeometry,
    task: &TaskSnapshot,
    anchor: Pos2,
    selected: bool,
    annotation: BarAnnotation,
) {
    let text = match annotation {
        BarAnnotation::Normal => task.resource_names_label(),
        BarAnnotation::Milestone => task.finish_label(),
    };
    if text.trim().is_empty() {
        return;
    }
    let label_rect = Rect::from_min_max(
        pos2(chart.gantt_left, anchor.y - ROW_H * 0.5),
        pos2(chart.gantt_left + 1000.0, anchor.y + ROW_H * 0.5),
    );
    let clipped = painter.with_clip_rect(label_rect);
    clipped.text(
        anchor,
        Align2::LEFT_CENTER,
        text,
        FontId::new(13.0, FontFamily::Proportional),
        if selected {
            Color32::from_rgb(20, 20, 20)
        } else {
            Color32::from_rgb(35, 35, 35)
        },
    );
}

enum BarAnnotation {
    Normal,
    Milestone,
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
        for link in &task.predecessors {
            let Some(from_index) = tasks
                .iter()
                .position(|candidate| candidate.number == link.predecessor)
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
            let from_rect = task_bar_rect_for_dates_at_y(
                chart,
                from.start,
                from.finish,
                bar_center_for_row(chart, from_row),
            );
            let to_rect = task_bar_rect_for_dates_at_y(
                chart,
                task.start,
                task.finish,
                bar_center_for_row(chart, to_row),
            );
            let (x0, y0) = dependency_anchor(from_rect, link.relation, true);
            let (x1, y1) = dependency_anchor(to_rect, link.relation, false);
            let stroke = Stroke::new(1.0, Color32::from_rgb(92, 92, 92));
            let from_sign = if matches!(link.relation, DependencyRelation::Sf | DependencyRelation::Ss) {
                -1.0
            } else {
                1.0
            };
            let to_sign = if matches!(link.relation, DependencyRelation::Fs | DependencyRelation::Ss) {
                -1.0
            } else {
                1.0
            };
            let x2 = x0 + from_sign * 5.0;
            let x3 = x1 + to_sign * 15.0;
            let y2 = (y0 + y1) * 0.5;

            match link.relation {
                DependencyRelation::Fs | DependencyRelation::Sf => {
                    if (matches!(link.relation, DependencyRelation::Fs) && x3 >= x2)
                        || (matches!(link.relation, DependencyRelation::Sf) && x3 <= x2)
                    {
                        painter.line_segment([pos2(x0, y0), pos2(x3, y0)], stroke);
                        painter.line_segment([pos2(x3, y0), pos2(x3, y1)], stroke);
                    } else {
                        painter.line_segment([pos2(x0, y0), pos2(x2, y0)], stroke);
                        painter.line_segment([pos2(x2, y0), pos2(x2, y2)], stroke);
                        painter.line_segment([pos2(x2, y2), pos2(x3, y2)], stroke);
                        painter.line_segment([pos2(x3, y2), pos2(x3, y1)], stroke);
                    }
                }
                DependencyRelation::Ss | DependencyRelation::Ff => {
                    let x5 = if matches!(link.relation, DependencyRelation::Ss) {
                        x2.min(x3)
                    } else {
                        x2.max(x3)
                    };
                    painter.line_segment([pos2(x0, y0), pos2(x5, y0)], stroke);
                    painter.line_segment([pos2(x5, y0), pos2(x5, y1)], stroke);
                }
            }
            painter.add(Shape::convex_polygon(
                dependency_arrow_points(x1, y1, link.relation),
                Color32::from_rgb(92, 92, 92),
                Stroke::NONE,
            ));
        }
    }
}

fn dependency_anchor(rect: Rect, relation: DependencyRelation, predecessor: bool) -> (f32, f32) {
    let center_y = rect.center().y;
    let x = match (relation, predecessor) {
        (DependencyRelation::Ss, true) | (DependencyRelation::Sf, true) => rect.left(),
        (DependencyRelation::Ff, true) | (DependencyRelation::Fs, true) => rect.right(),
        (DependencyRelation::Ss, false) | (DependencyRelation::Fs, false) => rect.left(),
        (DependencyRelation::Ff, false) | (DependencyRelation::Sf, false) => rect.right(),
    };
    (x, center_y)
}

fn dependency_arrow_points(x: f32, y: f32, relation: DependencyRelation) -> Vec<Pos2> {
    let left = x - 6.0;
    let right = x;
    let top = y - 4.0;
    let bottom = y + 4.0;
    match relation {
        DependencyRelation::Ss | DependencyRelation::Sf => {
            vec![pos2(right, y), pos2(left, top), pos2(left, bottom)]
        }
        DependencyRelation::Ff | DependencyRelation::Fs => {
            vec![pos2(left, top), pos2(right, y), pos2(left, bottom)]
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProjectSnapshot;

    #[test]
    fn bar_center_matches_java_style_offset() {
        let snapshot = ProjectSnapshot {
            start_date: NaiveDate::from_ymd_opt(2025, 2, 3).expect("date"),
            end_date: NaiveDate::from_ymd_opt(2025, 2, 4).expect("date"),
            status_date: None,
            tasks: vec![],
        };
        let chart = TimelineGeometry::new(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0)),
            &snapshot,
            24.0,
            614.0,
        );

        assert_eq!(bar_center_for_row(&chart, 0), chart.row_top(0) + 9.5);
        assert_eq!(bar_center_for_row(&chart, 4), chart.row_top(4) + 9.5);
    }

    #[test]
    fn task_bar_rect_uses_expected_height() {
        let snapshot = ProjectSnapshot {
            start_date: NaiveDate::from_ymd_opt(2025, 2, 3).expect("date"),
            end_date: NaiveDate::from_ymd_opt(2025, 2, 4).expect("date"),
            status_date: None,
            tasks: vec![],
        };
        let chart = TimelineGeometry::new(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0)),
            &snapshot,
            24.0,
            614.0,
        );
        let rect = task_bar_rect_for_dates_at_y(
            &chart,
            NaiveDate::from_ymd_opt(2025, 2, 3).expect("date"),
            NaiveDate::from_ymd_opt(2025, 2, 3).expect("date"),
            bar_center_for_row(&chart, 0),
        );

        assert_eq!(rect.height(), BAR_H);
        assert_eq!(rect.center().y, bar_center_for_row(&chart, 0));
    }

    #[test]
    fn month_label_matches_java_style_abbreviation() {
        assert_eq!(month_label(2), "Feb");
        assert_eq!(month_label(12), "Dec");
    }
}
