use std::collections::HashSet;

use chrono::NaiveDate;
use eframe::egui::{pos2, vec2, Color32, Painter, Rect, Stroke};

use crate::model::{ProjectSnapshot, TaskSnapshot};
use crate::ui::{gantt_chart, icons::ProjectLibreIcons, task_table};

pub const HEADER_H: f32 = 30.0;
pub const MONTH_H: f32 = 28.0;
pub const DAY_H: f32 = 26.0;
pub const ROW_H: f32 = 19.0;
pub const SPLITTER_W: f32 = 6.0;
pub const CHART_MARGIN_X: f32 = 10.0;
pub const DAY_W: f32 = 24.0;
pub const LEFT_TABLE_W: f32 = task_table::DEFAULT_TABLE_W;

#[derive(Clone, Copy)]
pub struct VisibleTaskRow {
    pub task_index: usize,
}

pub struct TimelineGeometry {
    pub gantt_left: f32,
    pub rows_top: f32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub day_width: f32,
    pub origin_x: f32,
}

impl TimelineGeometry {
    pub fn new(
        rect: Rect,
        snapshot: &ProjectSnapshot,
        day_width: f32,
        left_table_width: f32,
    ) -> Self {
        Self {
            gantt_left: rect.left() + left_table_width + SPLITTER_W + CHART_MARGIN_X,
            rows_top: rect.top() + HEADER_H,
            start_date: snapshot.start_date,
            end_date: snapshot.end_date,
            day_width,
            origin_x: rect.left(),
        }
    }

    pub fn date_to_x(&self, date: NaiveDate) -> f32 {
        self.gantt_left + (date - self.start_date).num_days() as f32 * self.day_width
    }

    pub fn row_top(&self, index: usize) -> f32 {
        self.rows_top + index as f32 * ROW_H
    }

    pub fn row_at(&self, point: eframe::egui::Pos2, row_count: usize) -> Option<usize> {
        if point.y < self.rows_top {
            return None;
        }

        let row = ((point.y - self.rows_top) / ROW_H).floor() as usize;
        (row < row_count).then_some(row)
    }

    pub fn pixel_delta_to_days(&self, delta_x: f32) -> i64 {
        (delta_x / self.day_width.max(1.0)).round() as i64
    }
}

pub fn content_width(chart: &TimelineGeometry, left_table_width: f32) -> f32 {
    let duration_days = (chart.end_date - chart.start_date).num_days().max(0) as f32 + 1.0;
    left_table_width.max(task_table::DEFAULT_TABLE_W)
        + SPLITTER_W
        + CHART_MARGIN_X * 2.0
        + duration_days * chart.day_width
        + 240.0
}

pub fn content_height(visible_rows: usize) -> f32 {
    HEADER_H + visible_rows as f32 * ROW_H + 160.0
}

pub fn build_visible_rows(
    tasks: &[TaskSnapshot],
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

pub fn draw_workspace(
    painter: &Painter,
    rect: Rect,
    chart: &TimelineGeometry,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
    selected_task_id: usize,
    collapsed_summaries: &HashSet<usize>,
    left_table_width: f32,
    icons: &ProjectLibreIcons,
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
    painter.rect_filled(
        Rect::from_min_size(
            pos2(left_rect.right(), rect.top()),
            vec2(SPLITTER_W, rect.height()),
        ),
        0.0,
        Color32::from_rgb(226, 226, 226),
    );

    task_table::draw_headers(painter, left_rect);
    gantt_chart::draw_timeline_headers(painter, gantt_rect, chart);
    task_table::draw_rows(
        painter,
        left_rect,
        tasks,
        visible_rows,
        selected_task_id,
        collapsed_summaries,
        icons,
    );
    gantt_chart::draw_rows_and_grid(
        painter,
        gantt_rect,
        chart,
        tasks,
        visible_rows,
        selected_task_id,
    );
    gantt_chart::draw_dependency_links(painter, chart, tasks, visible_rows);
    gantt_chart::draw_task_bars(painter, chart, tasks, visible_rows, selected_task_id);
}
