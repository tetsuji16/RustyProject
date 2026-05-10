use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::NaiveDate;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::model::{DependencyLink, DependencyRelation};

pub struct JavaBridge {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

impl JavaBridge {
    pub fn start() -> Result<Self, String> {
        let classpath = env!("JAVA_BRIDGE_CLASSES").to_string();

        let mut child = Command::new("java")
            .arg("-cp")
            .arg(classpath)
            .arg("com.projectlibre.bridge.BridgeServer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|err| format!("Failed to start Java bridge: {err}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Bridge stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Bridge stdout unavailable".to_string())?;

        Ok(Self {
            child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
        })
    }

    pub fn snapshot(&mut self) -> Result<ProjectSnapshot, String> {
        self.request("SNAPSHOT")
    }

    pub fn apply_edit(&mut self, edit: EditCommand) -> Result<ProjectSnapshot, String> {
        match edit {
            EditCommand::Move { id, start, finish } => {
                self.request(&format!("MOVE_ABS\t{id}\t{start}\t{finish}"))
            }
            EditCommand::ResizeStart { id, start } => {
                self.request(&format!("RESIZE_START_ABS\t{id}\t{start}"))
            }
            EditCommand::ResizeEnd { id, finish } => {
                self.request(&format!("RESIZE_END_ABS\t{id}\t{finish}"))
            }
            EditCommand::SetProgress { id, progress } => {
                self.request(&format!("SET_PROGRESS\t{id}\t{progress:.4}"))
            }
        }
    }

    fn request(&mut self, command: &str) -> Result<ProjectSnapshot, String> {
        writeln!(self.stdin, "{command}").map_err(|err| format!("Bridge write failed: {err}"))?;
        self.stdin
            .flush()
            .map_err(|err| format!("Bridge flush failed: {err}"))?;

        let mut status = None;
        let mut range = None;
        let mut tasks = Vec::new();

        loop {
            let mut line = String::new();
            let bytes = self
                .stdout
                .read_line(&mut line)
                .map_err(|err| format!("Bridge read failed: {err}"))?;
            if bytes == 0 {
                return Err("Bridge exited unexpectedly".to_string());
            }

            let line = line.trim_end_matches(&['\r', '\n'][..]);
            if line == "END" {
                break;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            match parts.as_slice() {
                ["OK"] => status = Some(true),
                ["ERR", msg] => return Err(decode(msg)),
                ["RANGE", start, end] => {
                    range = Some((
                        NaiveDate::parse_from_str(start, "%Y-%m-%d")
                            .map_err(|err| format!("Invalid start date: {err}"))?,
                        NaiveDate::parse_from_str(end, "%Y-%m-%d")
                            .map_err(|err| format!("Invalid end date: {err}"))?,
                    ));
                }
                ["TASKS", _count] => {}
                ["TASK", id, name_b64, start, finish, progress, indent, summary, milestone, preds] =>
                {
                    let name_bytes = URL_SAFE_NO_PAD
                        .decode(name_b64)
                        .map_err(|err| format!("Invalid task name encoding: {err}"))?;
                    let name = String::from_utf8(name_bytes)
                        .map_err(|err| format!("Invalid task name UTF-8: {err}"))?;
                    let preds = parse_dependency_links(preds)?;
                    tasks.push(TaskSnapshot {
                        number: id
                            .parse::<usize>()
                            .map_err(|err| format!("Invalid id: {err}"))?,
                        name,
                        start: NaiveDate::parse_from_str(start, "%Y-%m-%d")
                            .map_err(|err| format!("Invalid task start: {err}"))?,
                        finish: NaiveDate::parse_from_str(finish, "%Y-%m-%d")
                            .map_err(|err| format!("Invalid task finish: {err}"))?,
                        progress: progress
                            .parse::<f32>()
                            .map_err(|err| format!("Invalid progress: {err}"))?,
                        indent: indent
                            .parse::<usize>()
                            .map_err(|err| format!("Invalid indent: {err}"))?,
                        summary: summary
                            .parse::<bool>()
                            .map_err(|err| format!("Invalid summary flag: {err}"))?,
                        milestone: milestone
                            .parse::<bool>()
                            .map_err(|err| format!("Invalid milestone flag: {err}"))?,
                        predecessors: preds,
                        resource_names: Vec::new(),
                        start_text: None,
                        finish_text: None,
                        duration_text: None,
                        notes: None,
                        deadline: None,
                    });
                }
                _ => return Err(format!("Malformed bridge response: {line}")),
            }
        }

        if status != Some(true) {
            return Err("Bridge returned no OK status".to_string());
        }

        let (start_date, end_date) =
            range.ok_or_else(|| "Bridge snapshot missing range".to_string())?;
        Ok(ProjectSnapshot {
            start_date,
            end_date,
            status_date: None,
            tasks,
        })
    }
}

impl Drop for JavaBridge {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

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

#[derive(Clone)]
pub struct ProjectSnapshot {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status_date: Option<NaiveDate>,
    pub tasks: Vec<TaskSnapshot>,
}

#[derive(Clone)]
pub struct TaskSnapshot {
    pub number: usize,
    pub name: String,
    pub start: NaiveDate,
    pub finish: NaiveDate,
    pub progress: f32,
    pub indent: usize,
    pub summary: bool,
    pub milestone: bool,
    pub predecessors: Vec<DependencyLink>,
    pub resource_names: Vec<String>,
    pub start_text: Option<String>,
    pub finish_text: Option<String>,
    pub duration_text: Option<String>,
    pub notes: Option<String>,
    pub deadline: Option<NaiveDate>,
}

fn parse_dependency_links(text: &str) -> Result<Vec<DependencyLink>, String> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for raw_link in text.split([';', ',']) {
        let link = raw_link.trim();
        if link.is_empty() {
            continue;
        }
        out.push(parse_dependency_link(link)?);
    }
    Ok(out)
}

fn parse_dependency_link(text: &str) -> Result<DependencyLink, String> {
    let mut digits_end = 0;
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            digits_end += ch.len_utf8();
        } else {
            break;
        }
    }

    if digits_end == 0 {
        return Err(format!("Invalid predecessor link: {text}"));
    }

    let predecessor = text[..digits_end]
        .parse::<usize>()
        .map_err(|err| format!("Invalid predecessor id in link '{text}': {err}"))?;
    let rest = text[digits_end..].trim();
    if rest.is_empty() {
        return Ok(DependencyLink::fs(predecessor));
    }

    let relation = if rest.starts_with("FS") {
        DependencyRelation::Fs
    } else if rest.starts_with("FF") {
        DependencyRelation::Ff
    } else if rest.starts_with("SS") {
        DependencyRelation::Ss
    } else if rest.starts_with("SF") {
        DependencyRelation::Sf
    } else {
        return Err(format!("Invalid dependency relation in link: {text}"));
    };

    let lag_text = rest[2..].trim();
    let lag = if lag_text.is_empty() {
        0
    } else {
        parse_dependency_lag(lag_text)?
    };

    Ok(DependencyLink {
        predecessor,
        relation,
        lag,
    })
}

fn parse_dependency_lag(text: &str) -> Result<i64, String> {
    let text = text.trim();
    let sign = if text.starts_with('-') {
        -1
    } else {
        1
    };
    let numeric = text.trim_start_matches(['+', '-']);
    let (number_text, unit) = numeric
        .trim_end_matches(|ch: char| ch.is_whitespace())
        .split_at(
            numeric
                .find(|ch: char| !ch.is_ascii_digit() && ch != '.')
                .unwrap_or(numeric.len()),
        );
    let value = number_text
        .parse::<f64>()
        .map_err(|err| format!("Invalid lag value '{text}': {err}"))?;
    let unit = unit.trim().to_ascii_lowercase();
    const MILLIS_PER_SECOND: f64 = 1_000.0;
    const MILLIS_PER_MINUTE: f64 = 60.0 * MILLIS_PER_SECOND;
    const MILLIS_PER_HOUR: f64 = 60.0 * MILLIS_PER_MINUTE;
    const MILLIS_PER_DAY: f64 = 24.0 * MILLIS_PER_HOUR;

    let scale = if unit.starts_with("ms") {
        1.0
    } else if unit.starts_with('s') {
        MILLIS_PER_SECOND
    } else if unit.starts_with('m') {
        MILLIS_PER_MINUTE
    } else if unit.starts_with('h') {
        MILLIS_PER_HOUR
    } else if unit.starts_with('d') {
        MILLIS_PER_DAY
    } else {
        return Err(format!("Invalid lag unit in link: {text}"));
    };

    Ok((sign as f64 * value * scale).round() as i64)
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

fn decode(value: &str) -> String {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .unwrap_or_else(|_| value.as_bytes().to_vec());
    String::from_utf8(bytes).unwrap_or_else(|_| value.to_string())
}
