use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::schedule;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum EditCommand {
    Move {
        id: usize,
        start: NaiveDate,
        finish: NaiveDate,
    },
    ResizeStart {
        id: usize,
        start: NaiveDate,
    },
    ResizeEnd {
        id: usize,
        finish: NaiveDate,
    },
    SetProgress {
        id: usize,
        progress: f32,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub tasks: Vec<TaskSnapshot>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskSnapshot {
    pub number: usize,
    pub name: String,
    pub start: NaiveDate,
    pub finish: NaiveDate,
    pub progress: f32,
    pub indent: usize,
    pub summary: bool,
    pub milestone: bool,
    pub predecessors: Vec<usize>,
    #[serde(default)]
    pub resource_names: Vec<String>,
    #[serde(default)]
    pub start_text: Option<String>,
    #[serde(default)]
    pub finish_text: Option<String>,
    #[serde(default)]
    pub duration_text: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub deadline: Option<NaiveDate>,
}

impl ProjectSnapshot {
    pub fn sample() -> Self {
        let mut tasks = Vec::new();

        macro_rules! task {
            ($id:expr, $name:expr, $start:expr, $finish:expr, $progress:expr, $indent:expr, $summary:expr, $milestone:expr $(, $pred:expr)*) => {{
                let predecessors = vec![$($pred),*];
                tasks.push(TaskSnapshot {
                    number: $id,
                    name: $name.to_string(),
                    start: parse_date($start),
                    finish: parse_date($finish),
                    progress: $progress,
                    indent: $indent,
                    summary: $summary,
                    milestone: $milestone,
                    predecessors,
                    resource_names: Vec::new(),
                    start_text: None,
                    finish_text: None,
                    duration_text: None,
                    notes: None,
                    deadline: None,
                });
            }};
        }

        task!(
            32,
            "Construction",
            "2025-01-29",
            "2025-03-21",
            0.43,
            0,
            true,
            false
        );
        task!(
            33,
            "Site preparation",
            "2025-01-29",
            "2025-02-06",
            1.00,
            0,
            true,
            false
        );
        task!(
            34,
            "Obtain permits",
            "2025-01-29",
            "2025-01-31",
            1.00,
            1,
            false,
            false
        );
        task!(
            35,
            "Survey and stake building",
            "2025-02-03",
            "2025-02-03",
            1.00,
            1,
            false,
            false,
            34
        );
        task!(
            36,
            "Clear lot",
            "2025-02-03",
            "2025-02-04",
            1.00,
            1,
            false,
            false,
            35
        );
        task!(
            37,
            "Temporary utilities",
            "2025-02-05",
            "2025-02-06",
            0.80,
            1,
            false,
            false,
            36
        );
        task!(
            38,
            "Site ready milestone",
            "2025-02-06",
            "2025-02-06",
            1.00,
            1,
            false,
            true,
            37
        );
        task!(
            39,
            "Foundation",
            "2025-02-07",
            "2025-02-20",
            0.62,
            0,
            true,
            false,
            38
        );
        task!(
            40,
            "Excavate footings",
            "2025-02-07",
            "2025-02-10",
            1.00,
            1,
            false,
            false,
            38
        );
        task!(
            41,
            "Pour concrete footings",
            "2025-02-11",
            "2025-02-14",
            0.90,
            1,
            false,
            false,
            40
        );
        task!(
            42,
            "Slab",
            "2025-02-17",
            "2025-02-20",
            0.35,
            0,
            true,
            false,
            41
        );
        task!(
            43,
            "Install vapor barrier",
            "2025-02-17",
            "2025-02-17",
            0.70,
            1,
            false,
            false,
            41
        );
        task!(
            44,
            "Pour slab",
            "2025-02-18",
            "2025-02-20",
            0.20,
            1,
            false,
            false,
            43
        );
        task!(
            45,
            "Framing",
            "2025-02-21",
            "2025-03-05",
            0.21,
            0,
            true,
            false,
            44
        );
        task!(
            46,
            "Frame exterior walls",
            "2025-02-21",
            "2025-02-26",
            0.35,
            1,
            false,
            false,
            44
        );
        task!(
            47,
            "Set roof trusses",
            "2025-02-27",
            "2025-03-05",
            0.10,
            1,
            false,
            false,
            46
        );
        task!(
            48,
            "Mechanical rough-in",
            "2025-03-03",
            "2025-03-11",
            0.05,
            0,
            true,
            false,
            46
        );
        task!(
            49,
            "Electrical rough-in",
            "2025-03-03",
            "2025-03-06",
            0.10,
            1,
            false,
            false,
            46
        );
        task!(
            50,
            "Plumbing rough-in",
            "2025-03-05",
            "2025-03-10",
            0.00,
            1,
            false,
            false,
            49
        );
        task!(
            51,
            "HVAC rough-in",
            "2025-03-07",
            "2025-03-11",
            0.00,
            1,
            false,
            false,
            49
        );
        task!(
            52,
            "Exterior close-in",
            "2025-03-06",
            "2025-03-17",
            0.00,
            0,
            true,
            false,
            47
        );
        task!(
            53,
            "Install roofing",
            "2025-03-06",
            "2025-03-10",
            0.00,
            1,
            false,
            false,
            47
        );
        task!(
            54,
            "Install windows",
            "2025-03-11",
            "2025-03-13",
            0.00,
            1,
            false,
            false,
            53
        );
        task!(
            55,
            "Exterior inspection",
            "2025-03-17",
            "2025-03-17",
            0.00,
            1,
            false,
            true,
            54
        );
        task!(
            56,
            "Interior finish",
            "2025-03-12",
            "2025-03-21",
            0.00,
            0,
            true,
            false,
            51
        );
        task!(
            57,
            "Insulation",
            "2025-03-12",
            "2025-03-13",
            0.00,
            1,
            false,
            false,
            51
        );
        task!(
            58,
            "Drywall",
            "2025-03-14",
            "2025-03-19",
            0.00,
            1,
            false,
            false,
            57
        );
        task!(
            59,
            "Final phase",
            "2025-03-20",
            "2025-03-21",
            0.00,
            0,
            true,
            false,
            55,
            58
        );
        task!(
            60,
            "Punch list",
            "2025-03-20",
            "2025-03-21",
            0.00,
            1,
            false,
            false,
            55,
            58
        );
        task!(
            61,
            "Substantial completion",
            "2025-03-21",
            "2025-03-21",
            0.00,
            1,
            false,
            true,
            60
        );

        let mut snapshot = Self::from_tasks(tasks);
        schedule::recompute_summary_ranges(&mut snapshot.tasks);
        snapshot.refresh_bounds();
        snapshot
    }

    pub fn from_tasks(tasks: Vec<TaskSnapshot>) -> Self {
        let mut snapshot = Self {
            start_date: parse_date("2025-01-01"),
            end_date: parse_date("2025-01-01"),
            tasks,
        };
        snapshot.refresh_bounds();
        snapshot
    }

    pub fn apply_edit(&mut self, edit: EditCommand) {
        self.clear_display_texts();
        schedule::apply_edit(self, edit);
    }

    pub fn normalize(&mut self) {
        schedule::normalize_calendar(&mut self.tasks);
        schedule::recompute_summary_ranges(&mut self.tasks);
        self.refresh_bounds();
    }

    pub fn next_task_id(&self) -> usize {
        self.tasks.iter().map(|task| task.number).max().unwrap_or(0) + 1
    }

    pub fn insert_task_after(&mut self, index: usize, task: TaskSnapshot) {
        let insert_at = (index + 1).min(self.tasks.len());
        self.tasks.insert(insert_at, task);
        self.normalize();
    }

    pub fn remove_subtree_at(&mut self, index: usize) {
        if index >= self.tasks.len() {
            return;
        }

        let indent = self.tasks[index].indent;
        let mut end = index + 1;
        while end < self.tasks.len() && self.tasks[end].indent > indent {
            end += 1;
        }
        self.tasks.drain(index..end);
        self.normalize();
    }

    pub fn task_index(&self, id: usize) -> Option<usize> {
        self.tasks.iter().position(|task| task.number == id)
    }

    pub fn task(&self, id: usize) -> Option<&TaskSnapshot> {
        self.task_index(id).map(|index| &self.tasks[index])
    }

    pub(crate) fn task_mut(&mut self, id: usize) -> Option<&mut TaskSnapshot> {
        let index = self.task_index(id)?;
        self.tasks.get_mut(index)
    }

    pub(crate) fn refresh_bounds(&mut self) {
        if self.tasks.is_empty() {
            self.start_date = parse_date("2025-01-01");
            self.end_date = parse_date("2025-01-01");
            return;
        }

        let mut start = self.tasks[0].start;
        let mut end = self.tasks[0].finish;
        for task in &self.tasks {
            if task.start < start {
                start = task.start;
            }
            if task.finish > end {
                end = task.finish;
            }
        }
        self.start_date = start;
        self.end_date = end;
    }

    pub(crate) fn clear_display_texts(&mut self) {
        for task in &mut self.tasks {
            task.start_text = None;
            task.finish_text = None;
            task.duration_text = None;
        }
    }
}

impl TaskSnapshot {
    pub fn start_label(&self) -> String {
        self.start_text
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.to_string())
            .unwrap_or_else(|| format!("{} 8:00", self.start.format("%Y/%m/%d")))
    }

    pub fn finish_label(&self) -> String {
        self.finish_text
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.to_string())
            .unwrap_or_else(|| format!("{} 17:00", self.finish.format("%Y/%m/%d")))
    }

    pub fn duration_days(&self) -> i64 {
        if self.milestone {
            0
        } else {
            working_days_inclusive(self.start, self.finish)
        }
    }

    pub fn duration_label(&self) -> String {
        if let Some(value) = self
            .duration_text
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            return value.to_string();
        }
        if self.milestone {
            "0 days".to_string()
        } else {
            let days = self.duration_days();
            if days == 1 {
                "1 day".to_string()
            } else {
                format!("{days} days")
            }
        }
    }

    pub fn resource_names_label(&self) -> String {
        self.resource_names.join(", ")
    }

    pub fn has_notes(&self) -> bool {
        self.notes
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    }

    pub fn missed_deadline(&self) -> bool {
        self.deadline
            .map(|deadline| self.finish > deadline)
            .unwrap_or(false)
    }
}

fn parse_date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("valid sample date")
}

fn working_days_inclusive(start: NaiveDate, finish: NaiveDate) -> i64 {
    use chrono::Datelike;

    let mut days = 0;
    let mut date = start;
    while date <= finish {
        if matches!(date.weekday().number_from_monday(), 1..=5) {
            days += 1;
        }
        date += chrono::Duration::days(1);
    }
    days
}
