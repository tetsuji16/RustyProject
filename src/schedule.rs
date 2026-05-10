use chrono::{Datelike, Duration};

use crate::model::{EditCommand, ProjectSnapshot, TaskSnapshot};

pub fn normalize_calendar(tasks: &mut [TaskSnapshot]) {
    for task in tasks.iter_mut() {
        if task.start > task.finish {
            task.finish = task.start;
        }
        if task.milestone {
            task.finish = task.start;
        }
    }
}

pub fn apply_edit(snapshot: &mut ProjectSnapshot, edit: EditCommand) {
    match edit {
        EditCommand::Move { id, start, finish } => {
            if let Some(task) = snapshot.task_mut(id) {
                task.start = start;
                task.finish = finish;
                normalize_task(task);
            }
        }
        EditCommand::ResizeStart { id, start } => {
            if let Some(task) = snapshot.task_mut(id) {
                task.start = start.min(task.finish);
                normalize_task(task);
            }
        }
        EditCommand::ResizeEnd { id, finish } => {
            if let Some(task) = snapshot.task_mut(id) {
                task.finish = finish.max(task.start);
                normalize_task(task);
            }
        }
        EditCommand::SetProgress { id, progress } => {
            if let Some(task) = snapshot.task_mut(id) {
                task.progress = progress.clamp(0.0, 1.0);
            }
        }
    }

    normalize_calendar(&mut snapshot.tasks);
    enforce_dependency_constraints(snapshot);
    recompute_summary_ranges(&mut snapshot.tasks);
    snapshot.refresh_bounds();
}

pub fn recompute_summary_ranges(tasks: &mut [TaskSnapshot]) {
    for index in (0..tasks.len()).rev() {
        if !tasks[index].summary {
            continue;
        }

        let indent = tasks[index].indent;
        let mut child_indexes = Vec::new();
        for child_index in index + 1..tasks.len() {
            if tasks[child_index].indent <= indent {
                break;
            }
            if tasks[child_index].indent == indent + 1 {
                child_indexes.push(child_index);
            }
        }

        if child_indexes.is_empty() {
            continue;
        }

        let mut start = tasks[child_indexes[0]].start;
        let mut finish = tasks[child_indexes[0]].finish;
        let mut weighted_progress = 0.0;
        let mut total_days = 0.0;

        for child_index in child_indexes {
            let child = &tasks[child_index];
            if child.start < start {
                start = child.start;
            }
            if child.finish > finish {
                finish = child.finish;
            }
            let duration = child.duration_days().max(1) as f32;
            weighted_progress += child.progress * duration;
            total_days += duration;
        }

        tasks[index].start = start;
        tasks[index].finish = finish;
        tasks[index].progress = if total_days > 0.0 {
            weighted_progress / total_days
        } else {
            0.0
        };
    }
}

fn normalize_task(task: &mut TaskSnapshot) {
    if task.milestone {
        task.finish = task.start;
        task.progress = task.progress.clamp(0.0, 1.0);
    }
}

fn enforce_dependency_constraints(snapshot: &mut ProjectSnapshot) {
    let len = snapshot.tasks.len();
    for index in 0..len {
        if snapshot.tasks[index].summary {
            continue;
        }

        let mut min_start = snapshot.tasks[index].start;
        for predecessor in snapshot.tasks[index].predecessors.clone() {
            if let Some(pred_index) = snapshot.task_index(predecessor) {
                let pred_finish =
                    next_working_day(snapshot.tasks[pred_index].finish + Duration::days(1));
                if pred_finish > min_start {
                    min_start = pred_finish;
                }
            }
        }

        if snapshot.tasks[index].start < min_start {
            let duration = snapshot.tasks[index].duration_days().max(1) - 1;
            snapshot.tasks[index].start = min_start;
            snapshot.tasks[index].finish = add_working_days(min_start, duration);
            if snapshot.tasks[index].milestone {
                snapshot.tasks[index].finish = snapshot.tasks[index].start;
            }
        }
    }
}

fn is_working_day(date: chrono::NaiveDate) -> bool {
    matches!(date.weekday().number_from_monday(), 1..=5)
}

fn next_working_day(mut date: chrono::NaiveDate) -> chrono::NaiveDate {
    while !is_working_day(date) {
        date += Duration::days(1);
    }
    date
}

fn add_working_days(mut date: chrono::NaiveDate, days: i64) -> chrono::NaiveDate {
    let mut remaining = days;
    while remaining > 0 {
        date += Duration::days(1);
        if is_working_day(date) {
            remaining -= 1;
        }
    }
    next_working_day(date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ProjectSnapshot, TaskSnapshot};

    fn date(value: &str) -> chrono::NaiveDate {
        chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn summary_range_rolls_up_children() {
        let mut snapshot = ProjectSnapshot::from_tasks(vec![
            TaskSnapshot {
                number: 1,
                name: "Summary".into(),
                start: date("2025-01-01"),
                finish: date("2025-01-01"),
                progress: 0.0,
                indent: 0,
                summary: true,
                milestone: false,
                predecessors: vec![],
                resource_names: vec![],
                start_text: None,
                finish_text: None,
                duration_text: None,
                notes: None,
                deadline: None,
            },
            TaskSnapshot {
                number: 2,
                name: "Child A".into(),
                start: date("2025-01-02"),
                finish: date("2025-01-03"),
                progress: 0.5,
                indent: 1,
                summary: false,
                milestone: false,
                predecessors: vec![],
                resource_names: vec![],
                start_text: None,
                finish_text: None,
                duration_text: None,
                notes: None,
                deadline: None,
            },
            TaskSnapshot {
                number: 3,
                name: "Child B".into(),
                start: date("2025-01-05"),
                finish: date("2025-01-06"),
                progress: 1.0,
                indent: 1,
                summary: false,
                milestone: false,
                predecessors: vec![],
                resource_names: vec![],
                start_text: None,
                finish_text: None,
                duration_text: None,
                notes: None,
                deadline: None,
            },
        ]);

        recompute_summary_ranges(&mut snapshot.tasks);
        assert_eq!(snapshot.tasks[0].start, date("2025-01-02"));
        assert_eq!(snapshot.tasks[0].finish, date("2025-01-06"));
    }

    #[test]
    fn dependency_constraints_push_followers_forward() {
        let mut snapshot = ProjectSnapshot::from_tasks(vec![
            TaskSnapshot {
                number: 1,
                name: "A".into(),
                start: date("2025-01-01"),
                finish: date("2025-01-02"),
                progress: 0.0,
                indent: 0,
                summary: false,
                milestone: false,
                predecessors: vec![],
                resource_names: vec![],
                start_text: None,
                finish_text: None,
                duration_text: None,
                notes: None,
                deadline: None,
            },
            TaskSnapshot {
                number: 2,
                name: "B".into(),
                start: date("2025-01-01"),
                finish: date("2025-01-01"),
                progress: 0.0,
                indent: 0,
                summary: false,
                milestone: false,
                predecessors: vec![1],
                resource_names: vec![],
                start_text: None,
                finish_text: None,
                duration_text: None,
                notes: None,
                deadline: None,
            },
        ]);

        apply_edit(
            &mut snapshot,
            EditCommand::Move {
                id: 1,
                start: date("2025-01-10"),
                finish: date("2025-01-11"),
            },
        );
        assert!(snapshot.tasks[1].start > snapshot.tasks[0].finish);
    }

    #[test]
    fn calendar_normalization_snaps_weekend_tasks_forward() {
        let mut snapshot = ProjectSnapshot::from_tasks(vec![TaskSnapshot {
            number: 1,
            name: "Weekend".into(),
            start: date("2025-01-11"),
            finish: date("2025-01-13"),
            progress: 0.0,
            indent: 0,
            summary: false,
            milestone: false,
            predecessors: vec![],
            resource_names: vec![],
            start_text: None,
            finish_text: None,
            duration_text: None,
            notes: None,
            deadline: None,
        }]);

        snapshot.normalize();

        assert_eq!(snapshot.tasks[0].start, date("2025-01-11"));
        assert_eq!(snapshot.tasks[0].finish, date("2025-01-13"));
    }

    #[test]
    fn dependency_constraints_use_working_days() {
        let mut snapshot = ProjectSnapshot::from_tasks(vec![
            TaskSnapshot {
                number: 1,
                name: "Pred".into(),
                start: date("2025-01-09"),
                finish: date("2025-01-10"),
                progress: 0.0,
                indent: 0,
                summary: false,
                milestone: false,
                predecessors: vec![],
                resource_names: vec![],
                start_text: None,
                finish_text: None,
                duration_text: None,
                notes: None,
                deadline: None,
            },
            TaskSnapshot {
                number: 2,
                name: "Follower".into(),
                start: date("2025-01-11"),
                finish: date("2025-01-13"),
                progress: 0.0,
                indent: 0,
                summary: false,
                milestone: false,
                predecessors: vec![1],
                resource_names: vec![],
                start_text: None,
                finish_text: None,
                duration_text: None,
                notes: None,
                deadline: None,
            },
        ]);

        apply_edit(
            &mut snapshot,
            EditCommand::Move {
                id: 1,
                start: date("2025-01-16"),
                finish: date("2025-01-17"),
            },
        );

        assert_eq!(snapshot.tasks[1].start, date("2025-01-20"));
        assert_eq!(snapshot.tasks[1].finish, date("2025-01-20"));
    }
}
