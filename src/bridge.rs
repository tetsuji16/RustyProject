use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::NaiveDate;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

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
                    let preds = if preds.is_empty() {
                        Vec::new()
                    } else {
                        preds
                            .split(',')
                            .map(|value| value.parse::<usize>())
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|err| format!("Invalid predecessor id: {err}"))?
                    };
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
    pub predecessors: Vec<usize>,
}

impl TaskSnapshot {
    pub fn duration_days(&self) -> i64 {
        if self.milestone {
            0
        } else {
            (self.finish - self.start).num_days() + 1
        }
    }

    pub fn duration_label(&self) -> String {
        if self.milestone {
            "0 days".to_string()
        } else {
            format!("{} days", self.duration_days())
        }
    }
}

fn decode(value: &str) -> String {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .unwrap_or_else(|_| value.as_bytes().to_vec());
    String::from_utf8(bytes).unwrap_or_else(|_| value.to_string())
}
