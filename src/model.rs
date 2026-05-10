use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    #[serde(default)]
    pub status_date: Option<NaiveDate>,
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
    #[serde(default)]
    pub predecessors: Vec<DependencyLink>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyRelation {
    #[serde(rename = "FF")]
    Ff,
    #[serde(rename = "FS")]
    Fs,
    #[serde(rename = "SF")]
    Sf,
    #[serde(rename = "SS")]
    Ss,
}

impl Default for DependencyRelation {
    fn default() -> Self {
        Self::Fs
    }
}

impl DependencyRelation {
    pub fn as_code(self) -> &'static str {
        match self {
            Self::Ff => "FF",
            Self::Fs => "FS",
            Self::Sf => "SF",
            Self::Ss => "SS",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyLink {
    pub predecessor: usize,
    pub relation: DependencyRelation,
    pub lag: i64,
}

impl DependencyLink {
    pub fn fs(predecessor: usize) -> Self {
        Self {
            predecessor,
            relation: DependencyRelation::Fs,
            lag: 0,
        }
    }

    pub fn is_default(&self) -> bool {
        self.relation == DependencyRelation::Fs && self.lag == 0
    }

    pub fn display_text(&self) -> String {
        let mut out = self.predecessor.to_string();
        if self.relation != DependencyRelation::Fs || self.lag != 0 {
            out.push_str(self.relation.as_code());
            if self.lag != 0 {
                out.push_str(&format_dependency_lag(self.lag));
            }
        }
        out
    }
}

impl Serialize for DependencyLink {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.is_default() {
            serializer.serialize_u64(self.predecessor as u64)
        } else {
            #[derive(Serialize)]
            struct DependencyLinkSerde {
                predecessor: usize,
                relation: DependencyRelation,
                lag: i64,
            }

            DependencyLinkSerde {
                predecessor: self.predecessor,
                relation: self.relation,
                lag: self.lag,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for DependencyLink {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DependencyLinkRepr {
            Id(usize),
            Link {
                predecessor: usize,
                #[serde(default)]
                relation: Option<DependencyRelation>,
                #[serde(default)]
                lag: Option<i64>,
            },
        }

        match DependencyLinkRepr::deserialize(deserializer)? {
            DependencyLinkRepr::Id(predecessor) => Ok(DependencyLink::fs(predecessor)),
            DependencyLinkRepr::Link {
                predecessor,
                relation,
                lag,
            } => Ok(DependencyLink {
                predecessor,
                relation: relation.unwrap_or_default(),
                lag: lag.unwrap_or(0),
            }),
        }
    }
}

impl ProjectSnapshot {
    pub fn sample() -> Self {
        let mut tasks = Vec::new();

        macro_rules! task {
            ($id:expr, $name:expr, $start:expr, $finish:expr, $progress:expr, $indent:expr, $summary:expr, $milestone:expr $(, $pred:expr)*) => {{
                let predecessors = vec![$(DependencyLink::fs($pred)),*];
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
            status_date: None,
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
            .unwrap_or_else(|| {
                let day_abbr = self.start.format("%a").to_string();
                format!(
                    "{} {:02}/{:02}/{:02} 8:00",
                    day_abbr,
                    self.start.month(),
                    self.start.day(),
                    self.start.year() % 100
                )
            })
    }

    pub fn finish_label(&self) -> String {
        self.finish_text
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.to_string())
            .unwrap_or_else(|| {
                let day_abbr = self.finish.format("%a").to_string();
                format!(
                    "{} {:02}/{:02}/{:02} 17:00",
                    day_abbr,
                    self.finish.month(),
                    self.finish.day(),
                    self.finish.year() % 100
                )
            })
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

    pub fn predecessors_label(&self) -> String {
        self.predecessors
            .iter()
            .filter(|link| !link.is_default())
            .map(|link| link.display_text())
            .collect::<Vec<_>>()
            .join(";")
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

fn format_dependency_lag(lag: i64) -> String {
    const MILLIS_PER_SECOND: i64 = 1_000;
    const MILLIS_PER_MINUTE: i64 = 60 * MILLIS_PER_SECOND;
    const MILLIS_PER_HOUR: i64 = 60 * MILLIS_PER_MINUTE;
    const MILLIS_PER_DAY: i64 = 24 * MILLIS_PER_HOUR;

    let sign = if lag > 0 {
        "+"
    } else if lag < 0 {
        "-"
    } else {
        ""
    };
    let abs = lag.abs();
    if abs % MILLIS_PER_DAY == 0 {
        format!("{sign}{}d", abs / MILLIS_PER_DAY)
    } else if abs % MILLIS_PER_HOUR == 0 {
        format!("{sign}{}h", abs / MILLIS_PER_HOUR)
    } else if abs % MILLIS_PER_MINUTE == 0 {
        format!("{sign}{}m", abs / MILLIS_PER_MINUTE)
    } else if abs % MILLIS_PER_SECOND == 0 {
        format!("{sign}{}s", abs / MILLIS_PER_SECOND)
    } else {
        format!("{sign}{}ms", abs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("valid test date")
    }

    #[test]
    fn label_fallbacks_match_expected_projectlibre_shape() {
        let task = TaskSnapshot {
            number: 1,
            name: "Task".to_string(),
            start: date("2025-02-03"),
            finish: date("2025-02-07"),
            progress: 0.5,
            indent: 0,
            summary: false,
            milestone: false,
            predecessors: vec![],
            resource_names: vec!["Alice".to_string(), "Bob".to_string()],
            start_text: None,
            finish_text: None,
            duration_text: None,
            notes: None,
            deadline: None,
        };

        assert_eq!(task.start_label(), "Mon 02/03/25 8:00");
        assert_eq!(task.finish_label(), "Fri 02/07/25 17:00");
        assert_eq!(task.duration_label(), "5 days");
        assert_eq!(task.resource_names_label(), "Alice, Bob");
    }

    #[test]
    fn predecessors_label_uses_java_style_relation_suffixes() {
        let task = TaskSnapshot {
            number: 3,
            name: "Task".to_string(),
            start: date("2025-02-03"),
            finish: date("2025-02-04"),
            progress: 0.0,
            indent: 0,
            summary: false,
            milestone: false,
            predecessors: vec![
                DependencyLink::fs(10),
                DependencyLink {
                    predecessor: 11,
                    relation: DependencyRelation::Ff,
                    lag: 24 * 60 * 60 * 1000,
                },
            ],
            resource_names: vec![],
            start_text: None,
            finish_text: None,
            duration_text: None,
            notes: None,
            deadline: None,
        };

        assert_eq!(task.predecessors_label(), "11FF+1d");
    }

    #[test]
    fn dependency_links_still_accept_legacy_integer_json() {
        let task: TaskSnapshot = serde_json::from_str(
            r#"{
                "number": 1,
                "name": "Task",
                "start": "2025-02-03",
                "finish": "2025-02-04",
                "progress": 0.0,
                "indent": 0,
                "summary": false,
                "milestone": false,
                "predecessors": [34]
            }"#,
        )
        .expect("legacy predecessor json should deserialize");

        assert_eq!(task.predecessors.len(), 1);
        assert_eq!(task.predecessors[0].predecessor, 34);
        assert_eq!(task.predecessors[0].relation, DependencyRelation::Fs);
        assert_eq!(task.predecessors[0].lag, 0);
    }

    #[test]
    fn task_flags_follow_basic_rules() {
        let mut task = TaskSnapshot {
            number: 2,
            name: "Task".to_string(),
            start: date("2025-02-03"),
            finish: date("2025-02-04"),
            progress: 1.0,
            indent: 0,
            summary: false,
            milestone: false,
            predecessors: vec![],
            resource_names: vec![],
            start_text: None,
            finish_text: None,
            duration_text: None,
            notes: Some("note".to_string()),
            deadline: Some(date("2025-02-03")),
        };

        assert!(task.has_notes());
        assert!(task.missed_deadline());

        task.notes = Some("   ".to_string());
        task.deadline = Some(date("2025-02-10"));
        assert!(!task.has_notes());
        assert!(!task.missed_deadline());
    }
}
