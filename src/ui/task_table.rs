use std::collections::HashSet;

use eframe::egui::{
    pos2, vec2, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke,
};

use crate::model::TaskSnapshot;
use crate::ui::gantt_view::{TimelineGeometry, VisibleTaskRow, HEADER_H, ROW_H};
use crate::ui::icons::{IconKey, ProjectLibreIcons};

const ROWNUM_W: f32 = 32.0;
const INDICATORS_W: f32 = 30.0;
const NAME_W: f32 = 140.0;
const DURATION_W: f32 = 60.0;
const START_W: f32 = 80.0;
const FINISH_W: f32 = 80.0;
const PREDECESSORS_W: f32 = 72.0;
const RESOURCE_W: f32 = 82.0;

pub const DEFAULT_TABLE_W: f32 =
    INDICATORS_W + NAME_W + DURATION_W + START_W + FINISH_W + PREDECESSORS_W + RESOURCE_W;

#[derive(Clone, Copy)]
pub enum TableColumn {
    Indicators,
    Name,
    Duration,
    Start,
    Finish,
    Predecessors,
    ResourceNames,
}

impl TableColumn {
    fn label(self) -> &'static str {
        match self {
            Self::Indicators => "",
            Self::Name => "名前",
            Self::Duration => "期間",
            Self::Start => "開始",
            Self::Finish => "終了",
            Self::Predecessors => "先行",
            Self::ResourceNames => "リソース名",
        }
    }

    fn width(self, table_width: f32) -> f32 {
        match self {
            Self::Indicators => INDICATORS_W,
            Self::Name => name_width(table_width),
            Self::Duration => DURATION_W,
            Self::Start => START_W,
            Self::Finish => FINISH_W,
            Self::Predecessors => PREDECESSORS_W,
            Self::ResourceNames => RESOURCE_W,
        }
    }
}

const COLUMNS: [TableColumn; 7] = [
    TableColumn::Indicators,
    TableColumn::Name,
    TableColumn::Duration,
    TableColumn::Start,
    TableColumn::Finish,
    TableColumn::Predecessors,
    TableColumn::ResourceNames,
];

pub fn draw_headers(painter: &Painter, rect: Rect, table_width: f32) {
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
    painter.text(
        rownum_rect.center(),
        Align2::CENTER_CENTER,
        "",
        FontId::new(14.0, FontFamily::Proportional),
        Color32::from_rgb(38, 38, 38),
    );

    for (column, column_rect) in
        column_rects(rect.left() + ROWNUM_W, rect.top(), HEADER_H, table_width)
    {
        painter.line_segment(
            [
                pos2(column_rect.right(), column_rect.top()),
                pos2(column_rect.right(), column_rect.bottom()),
            ],
            Stroke::new(1.0, border),
        );
        let label = column.label();
        if !label.is_empty() {
            painter.text(
                column_rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::new(14.0, FontFamily::Proportional),
                Color32::from_rgb(38, 38, 38),
            );
        } else {
            draw_indicator_header(painter, column_rect.center());
        }
    }
}

pub fn draw_rows(
    painter: &Painter,
    rect: Rect,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
    selected_task_id: usize,
    collapsed_summaries: &HashSet<usize>,
    table_width: f32,
    icons: &ProjectLibreIcons,
) {
    let line = Color32::from_rgb(214, 214, 214);
    for (row_index, row) in visible_rows.iter().enumerate() {
        let task = &tasks[row.task_index];
        let y = rect.top() + HEADER_H + row_index as f32 * ROW_H;
        let row_rect = Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), ROW_H));
        let rownum_rect = Rect::from_min_size(pos2(rect.left(), y), vec2(ROWNUM_W, ROW_H));
        let selected_row = task.number == selected_task_id;
        let bg = if selected_row {
            Color32::from_rgb(84, 94, 108)
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

        for (column, column_rect) in column_rects(rect.left() + ROWNUM_W, y, ROW_H, table_width) {
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
        }
    }
}

pub fn hit_test_row_toggle(
    chart: &TimelineGeometry,
    tasks: &[TaskSnapshot],
    visible_rows: &[VisibleTaskRow],
    pointer: Pos2,
) -> Option<(VisibleTaskRow, bool)> {
    if pointer.x < chart.origin_x || pointer.x > chart.gantt_left {
        return None;
    }

    let row_index = chart.row_at(pointer, visible_rows.len())?;
    let row = visible_rows[row_index];
    let task = &tasks[row.task_index];
    let name_left = column_left(
        chart.origin_x + ROWNUM_W,
        TableColumn::Name,
        DEFAULT_TABLE_W,
    );
    let toggle_x = name_left + 8.0 + task.indent as f32 * 18.0;
    let toggle_rect = Rect::from_center_size(
        pos2(toggle_x, chart.row_top(row_index) + ROW_H * 0.5),
        vec2(20.0, 18.0),
    );
    Some((row, task.summary && toggle_rect.contains(pointer)))
}

fn draw_cell(
    painter: &Painter,
    column: TableColumn,
    rect: Rect,
    task: &TaskSnapshot,
    selected: bool,
    color: Color32,
    collapsed_summaries: &HashSet<usize>,
    icons: &ProjectLibreIcons,
) {
    match column {
        TableColumn::Indicators => draw_indicators(painter, rect, task, selected, icons),
        TableColumn::Name => draw_name(
            painter,
            rect,
            task,
            selected,
            color,
            collapsed_summaries,
            icons,
        ),
        TableColumn::Duration => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.duration_label(),
            Align2::LEFT_CENTER,
            color,
            14.0,
        ),
        TableColumn::Start => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.start_label(),
            Align2::LEFT_CENTER,
            color,
            13.0,
        ),
        TableColumn::Finish => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.finish_label(),
            Align2::LEFT_CENTER,
            color,
            13.0,
        ),
        TableColumn::Predecessors => {
            let value = task
                .predecessors
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            draw_text(
                painter,
                rect.shrink2(vec2(8.0, 0.0)),
                value,
                Align2::LEFT_CENTER,
                color,
                14.0,
            );
        }
        TableColumn::ResourceNames => draw_text(
            painter,
            rect.shrink2(vec2(8.0, 0.0)),
            task.resource_names.join(", "),
            Align2::LEFT_CENTER,
            color,
            14.0,
        ),
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
    let x = rect.left() + 8.0 + task.indent as f32 * 18.0;
    let center_y = rect.center().y;
    if task.summary {
        let expanded = !collapsed_summaries.contains(&task.number);
        draw_expand_box(painter, pos2(x, center_y), selected, expanded, icons);
    } else if task.milestone {
        draw_milestone_icon(painter, pos2(x + 1.0, center_y), selected);
    }

    let name_x = x + if task.summary || task.milestone {
        18.0
    } else {
        0.0
    };
    let clipped = painter.with_clip_rect(rect);
    clipped.text(
        pos2(name_x, center_y),
        Align2::LEFT_CENTER,
        task.name.clone(),
        FontId::new(
            if task.summary { 15.0 } else { 14.0 },
            FontFamily::Proportional,
        ),
        color,
    );
}

fn draw_indicators(
    painter: &Painter,
    rect: Rect,
    task: &TaskSnapshot,
    selected: bool,
    icons: &ProjectLibreIcons,
) {
    let size = vec2(12.0, 12.0);
    let center = rect.center();
    let image_rect = Rect::from_center_size(center, size);
    if task.summary {
        if let Some(texture) = icons.texture(IconKey::Wbs) {
            painter.image(
                texture.id(),
                image_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.rect_filled(image_rect, 1.0, Color32::from_rgb(78, 78, 78));
        }
    } else if task.milestone {
        draw_milestone_icon(painter, rect.center(), selected);
    } else if !task.predecessors.is_empty() {
        if let Some(texture) = icons.texture(IconKey::Constraint) {
            painter.image(
                texture.id(),
                image_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            let color = if selected {
                Color32::WHITE
            } else {
                Color32::from_rgb(78, 78, 78)
            };
            let y = rect.center().y;
            painter.line_segment(
                [
                    pos2(rect.center().x - 7.0, y),
                    pos2(rect.center().x + 5.0, y),
                ],
                Stroke::new(1.5, color),
            );
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(rect.center().x + 5.0, y - 4.0),
                    pos2(rect.center().x + 10.0, y),
                    pos2(rect.center().x + 5.0, y + 4.0),
                ],
                color,
                Stroke::NONE,
            ));
        }
    } else if let Some(texture) = icons.texture(IconKey::Info) {
        painter.image(
            texture.id(),
            image_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
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

fn draw_expand_box(
    painter: &Painter,
    center: Pos2,
    inverted: bool,
    expanded: bool,
    icons: &ProjectLibreIcons,
) {
    let rect = Rect::from_center_size(center, vec2(11.0, 11.0));
    let icon_key = if expanded {
        IconKey::Minus
    } else {
        IconKey::Plus
    };
    if let Some(texture) = icons.texture(icon_key) {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        let fill = if inverted {
            Color32::from_rgb(110, 110, 110)
        } else {
            Color32::from_rgb(245, 245, 245)
        };
        let stroke = if inverted {
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
}

fn draw_milestone_icon(painter: &Painter, center: Pos2, inverted: bool) {
    let half = 5.0;
    let fill = if inverted {
        Color32::WHITE
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

fn column_rects(left: f32, top: f32, height: f32, table_width: f32) -> Vec<(TableColumn, Rect)> {
    let mut x = left;
    COLUMNS
        .iter()
        .map(|column| {
            let width = column.width(table_width);
            let rect = Rect::from_min_size(pos2(x, top), vec2(width, height));
            x += width;
            (*column, rect)
        })
        .collect()
}

fn column_left(left: f32, target: TableColumn, table_width: f32) -> f32 {
    let mut x = left;
    for column in COLUMNS {
        if std::mem::discriminant(&column) == std::mem::discriminant(&target) {
            return x;
        }
        x += column.width(table_width);
    }
    x
}

fn name_width(table_width: f32) -> f32 {
    (table_width - (DEFAULT_TABLE_W - NAME_W)).max(180.0)
}
