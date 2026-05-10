#[path = "../src/model.rs"]
mod model;
#[path = "../src/mpp_import.rs"]
mod mpp_import;
#[path = "../src/pod_import.rs"]
mod pod_import;
#[path = "../src/schedule.rs"]
mod schedule;

use std::path::PathBuf;

#[test]
fn imports_sample_pod_from_repo_path() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("sample data/Commercial construction project plan.pod");

    let snapshot = pod_import::load_pod(&path).unwrap_or_else(|err| {
        panic!("POD import failed for {}: {err}", path.display());
    });

    assert!(
        !snapshot.tasks.is_empty(),
        "POD import should produce at least one task"
    );
    assert!(
        snapshot.start_date <= snapshot.end_date,
        "project date range should be valid"
    );
}
