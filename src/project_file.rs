use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::model::ProjectSnapshot;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub version: u32,
    pub snapshot: ProjectSnapshot,
    pub selected_task_id: usize,
    pub collapsed_summaries: Vec<usize>,
    pub day_width: f32,
    pub left_table_width: f32,
}

impl ProjectDocument {
    pub fn from_app_state(
        snapshot: ProjectSnapshot,
        selected_task_id: usize,
        collapsed_summaries: Vec<usize>,
        day_width: f32,
        left_table_width: f32,
    ) -> Self {
        Self {
            version: 1,
            snapshot,
            selected_task_id,
            collapsed_summaries,
            day_width,
            left_table_width,
        }
    }
}

pub fn save(path: impl AsRef<Path>, document: &ProjectDocument) -> Result<(), String> {
    let json = serde_json::to_string_pretty(document).map_err(|err| format!("serialize: {err}"))?;
    fs::write(path, json).map_err(|err| format!("write file: {err}"))
}

pub fn load(path: impl AsRef<Path>) -> Result<ProjectDocument, String> {
    let json = fs::read_to_string(path).map_err(|err| format!("read file: {err}"))?;
    let mut document: ProjectDocument =
        serde_json::from_str(&json).map_err(|err| format!("parse file: {err}"))?;
    sanitize_document(&mut document);
    Ok(document)
}

fn sanitize_document(document: &mut ProjectDocument) {
    if document.version == 0 {
        document.version = 1;
    }

    if document.day_width < 14.0 {
        document.day_width = 14.0;
    }
    if document.left_table_width < 280.0 {
        document.left_table_width = 280.0;
    }

    document
        .snapshot
        .tasks
        .retain(|task| task.number > 0 || !task.name.is_empty());
    document.snapshot.normalize();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProjectSnapshot;

    #[test]
    fn round_trips_json_document() {
        let document = ProjectDocument::from_app_state(
            ProjectSnapshot::sample(),
            32,
            vec![39, 45],
            24.0,
            520.0,
        );
        let path = std::env::temp_dir().join("gantt_rust_rewrite_project_test.json");

        save(&path, &document).expect("save document");
        let loaded = load(&path).expect("load document");
        let _ = fs::remove_file(&path);

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.selected_task_id, 32);
        assert_eq!(loaded.collapsed_summaries, vec![39, 45]);
        assert_eq!(loaded.snapshot.tasks.len(), document.snapshot.tasks.len());
    }
}
