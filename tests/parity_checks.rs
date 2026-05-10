#[path = "../src/model.rs"]
mod model;
#[path = "../src/schedule.rs"]
mod schedule;

use chrono::NaiveDate;
use model::TaskSnapshot;

#[test]
fn public_task_display_fallbacks_stay_stable() {
    let task = TaskSnapshot {
        number: 1,
        name: "Test".to_string(),
        start: NaiveDate::from_ymd_opt(2024, 6, 1).expect("date"),
        finish: NaiveDate::from_ymd_opt(2024, 6, 10).expect("date"),
        progress: 1.0,
        indent: 0,
        summary: true,
        milestone: false,
        predecessors: Vec::new(),
        resource_names: vec!["R1".to_string()],
        start_text: None,
        finish_text: None,
        duration_text: None,
        notes: Some("notes".to_string()),
        deadline: Some(NaiveDate::from_ymd_opt(2024, 6, 5).expect("date")),
    };

    assert!(task.has_notes());
    assert!(task.missed_deadline());
    assert_eq!(task.resource_names_label(), "R1");
    assert_eq!(task.duration_label(), "6 days");
}
