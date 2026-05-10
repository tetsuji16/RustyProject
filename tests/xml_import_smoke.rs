#[path = "../src/model.rs"]
mod model;
#[path = "../src/mpp_import.rs"]
mod mpp_import;
#[path = "../src/pod_export.rs"]
mod pod_export;
#[path = "../src/pod_import.rs"]
mod pod_import;
#[path = "../src/schedule.rs"]
mod schedule;

use std::path::PathBuf;

#[test]
fn imports_sample_xml_from_repo_path() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("sample data/Commercial construction project plan.xml");

    let snapshot = pod_import::load_xml(&path).expect("sample XML should import");

    assert!(
        !snapshot.tasks.is_empty(),
        "XML import should produce at least one task"
    );
    assert!(
        snapshot.start_date <= snapshot.end_date,
        "project date range should be valid"
    );
}

#[test]
fn exports_sample_xml_round_trip() {
    let snapshot = model::ProjectSnapshot::sample();
    let path = std::env::temp_dir().join("rustyproject_xml_roundtrip_test.xml");

    pod_export::save_xml(&path, &snapshot).expect("save xml");
    let imported = pod_import::load_xml(&path).expect("load round-tripped xml");
    let _ = std::fs::remove_file(&path);

    assert_eq!(imported.tasks.len(), snapshot.tasks.len());
}
