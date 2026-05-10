use std::fs;
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::model::{DependencyRelation, ProjectSnapshot, TaskSnapshot};

const POD_SEPARATOR: &[u8] = b"@@@@@@@@@@ProjectLibreSeparator_MSXML@@@@@@@@@@";
const POD_PREFIX: &[u8] = &[
    0xAC, 0xED, 0x00, 0x05, 0x74, 0x00, 0x05, 0x31, 0x2E, 0x30, 0x2E, 0x30,
];

pub fn save_pod(path: impl AsRef<Path>, snapshot: &ProjectSnapshot) -> Result<(), String> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(POD_PREFIX);
    bytes.extend_from_slice(POD_SEPARATOR);
    bytes.extend_from_slice(build_mspdi_xml(snapshot).as_bytes());
    fs::write(path, bytes).map_err(|err| format!("write file: {err}"))
}

pub fn save_xml(path: impl AsRef<Path>, snapshot: &ProjectSnapshot) -> Result<(), String> {
    fs::write(path, build_mspdi_xml(snapshot)).map_err(|err| format!("write file: {err}"))
}

fn build_mspdi_xml(snapshot: &ProjectSnapshot) -> String {
    let tasks = build_tasks_xml(snapshot);
    let start = datetime_text(snapshot.start_date, 8, 0, 0);
    let finish = datetime_text(snapshot.end_date, 17, 0, 0);
    let project_name = "ProjectLibre";

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Project xmlns="http://schemas.microsoft.com/project">
    <SaveVersion>12</SaveVersion>
    <Name>{project_name}</Name>
    <Title>{project_name}</Title>
    <ScheduleFromStart>1</ScheduleFromStart>
    <StartDate>{start}</StartDate>
    <FinishDate>{finish}</FinishDate>
    <CalendarUID>1</CalendarUID>
    <Calendars>
        <Calendar>
            <UID>1</UID>
            <Name>Standard</Name>
            <IsBaseCalendar>1</IsBaseCalendar>
        </Calendar>
    </Calendars>
    <Tasks>
{tasks}
    </Tasks>
</Project>
"#
    )
}

fn build_tasks_xml(snapshot: &ProjectSnapshot) -> String {
    let outline_numbers = outline_numbers(&snapshot.tasks);
    let mut out = String::new();

    for (index, task) in snapshot.tasks.iter().enumerate() {
        let uid = task.number;
        let outline = outline_numbers
            .get(index)
            .cloned()
            .unwrap_or_else(|| "1".to_string());
        let outline_level = task.indent + 1;
        let task_type = if task.summary { 1 } else { 0 };
        let duration_hours = task.duration_days().max(0) * 8;
        let duration = if task.milestone {
            "PT0H0M0S".to_string()
        } else {
            format!("PT{duration_hours}H0M0S")
        };
        let start = datetime_text(task.start, 8, 0, 0);
        let finish = datetime_text(task.finish, 17, 0, 0);
        let percent_complete = percentage(task.progress);

        out.push_str("        <Task>\n");
        out.push_str(&format!("            <UID>{uid}</UID>\n"));
        out.push_str(&format!("            <ID>{uid}</ID>\n"));
        out.push_str(&format!(
            "            <Name>{}</Name>\n",
            escape_xml(&task.name)
        ));
        out.push_str(&format!("            <Type>{task_type}</Type>\n"));
        out.push_str("            <IsNull>0</IsNull>\n");
        out.push_str(&format!("            <CreateDate>{start}</CreateDate>\n"));
        out.push_str(&format!("            <WBS>{outline}</WBS>\n"));
        out.push_str(&format!(
            "            <OutlineNumber>{outline}</OutlineNumber>\n"
        ));
        out.push_str(&format!(
            "            <OutlineLevel>{outline_level}</OutlineLevel>\n"
        ));
        out.push_str("            <Priority>500</Priority>\n");
        out.push_str(&format!("            <Start>{start}</Start>\n"));
        out.push_str(&format!("            <Finish>{finish}</Finish>\n"));
        out.push_str(&format!("            <Duration>{duration}</Duration>\n"));
        out.push_str("            <DurationFormat>7</DurationFormat>\n");
        out.push_str(&format!("            <Resume>{start}</Resume>\n"));
        out.push_str("            <ResumeValid>0</ResumeValid>\n");
        out.push_str("            <EffortDriven>1</EffortDriven>\n");
        out.push_str("            <Recurring>0</Recurring>\n");
        out.push_str("            <OverAllocated>0</OverAllocated>\n");
        out.push_str("            <Estimated>0</Estimated>\n");
        out.push_str(&format!(
            "            <Milestone>{}</Milestone>\n",
            if task.milestone { 1 } else { 0 }
        ));
        out.push_str(&format!(
            "            <Summary>{}</Summary>\n",
            if task.summary { 1 } else { 0 }
        ));
        out.push_str("            <Critical>0</Critical>\n");
        out.push_str("            <IsSubproject>0</IsSubproject>\n");
        out.push_str("            <IsSubprojectReadOnly>0</IsSubprojectReadOnly>\n");
        out.push_str("            <ExternalTask>0</ExternalTask>\n");
        out.push_str("            <FixedCostAccrual>3</FixedCostAccrual>\n");
        out.push_str(&format!(
            "            <PercentComplete>{percent_complete}</PercentComplete>\n"
        ));
        out.push_str(&format!(
            "            <PercentWorkComplete>{percent_complete}</PercentWorkComplete>\n"
        ));
        out.push_str(&format!(
            "            <RemainingDuration>{}</RemainingDuration>\n",
            if task.milestone {
                "PT0H0M0S".to_string()
            } else {
                duration.clone()
            }
        ));
        out.push_str("            <Rollup>0</Rollup>\n");
        out.push_str("            <EarnedValueMethod>0</EarnedValueMethod>\n");
        for predecessor in &task.predecessors {
            out.push_str("            <PredecessorLink>\n");
            out.push_str(&format!(
                "                <PredecessorUID>{}</PredecessorUID>\n",
                predecessor.predecessor
            ));
            out.push_str("                <CrossProject>0</CrossProject>\n");
            out.push_str(&format!(
                "                <Type>{}</Type>\n",
                predecessor_type_code(predecessor.relation)
            ));
            out.push_str("            </PredecessorLink>\n");
        }
        out.push_str("            <Active>1</Active>\n");
        out.push_str("            <Manual>0</Manual>\n");
        out.push_str("        </Task>\n");
    }

    out
}

fn outline_numbers(tasks: &[TaskSnapshot]) -> Vec<String> {
    let mut counters = Vec::<usize>::new();
    let mut result = Vec::with_capacity(tasks.len());

    for task in tasks {
        let level = task.indent + 1;
        if counters.len() < level {
            counters.resize(level, 0);
        }
        counters[level - 1] += 1;
        for counter in counters.iter_mut().skip(level) {
            *counter = 0;
        }

        let parts: Vec<String> = counters[..level]
            .iter()
            .copied()
            .filter(|value| *value > 0)
            .map(|value| value.to_string())
            .collect();
        result.push(parts.join("."));
    }

    result
}

fn datetime_text(date: NaiveDate, hour: u32, minute: u32, second: u32) -> String {
    let time = NaiveTime::from_hms_opt(hour, minute, second).expect("valid time");
    NaiveDateTime::new(date, time)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}

fn percentage(progress: f32) -> i32 {
    (progress.clamp(0.0, 1.0) * 100.0).round() as i32
}

fn predecessor_type_code(relation: DependencyRelation) -> i32 {
    match relation {
        DependencyRelation::Ff => 0,
        DependencyRelation::Fs => 1,
        DependencyRelation::Sf => 2,
        DependencyRelation::Ss => 3,
    }
}

fn escape_xml(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pod_import::load_pod;

    #[test]
    fn pod_bytes_start_with_java_stream_magic_and_separator() {
        let snapshot = ProjectSnapshot::sample();
        let path = std::env::temp_dir().join("rustyproject_pod_export_test.pod");
        save_pod(&path, &snapshot).expect("save pod");
        let bytes = std::fs::read(&path).expect("read pod");
        let imported = load_pod(&path).expect("round trip pod");
        let _ = std::fs::remove_file(&path);

        assert!(bytes.starts_with(POD_PREFIX));
        assert!(bytes
            .windows(POD_SEPARATOR.len())
            .any(|window| window == POD_SEPARATOR));
        assert_eq!(imported.tasks.len(), snapshot.tasks.len());
    }

    #[test]
    fn xml_output_is_plain_mspdi_document() {
        let snapshot = ProjectSnapshot::sample();
        let path = std::env::temp_dir().join("rustyproject_xml_export_test.xml");
        save_xml(&path, &snapshot).expect("save xml");
        let xml = std::fs::read_to_string(&path).expect("read xml");
        let _ = std::fs::remove_file(&path);

        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<Project xmlns=\"http://schemas.microsoft.com/project\">"));
        assert!(xml.contains("<Tasks>"));
    }
}
