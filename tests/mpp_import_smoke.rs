#[path = "../src/model.rs"]
mod model;
#[path = "../src/mpp_import.rs"]
mod mpp_import;
#[path = "../src/schedule.rs"]
mod schedule;

use std::path::PathBuf;

fn main() {
    let Some(path) = std::env::args_os()
        .nth(1)
        .or_else(|| std::env::var_os("RUSTYPROJECT_SAMPLE_MPP"))
    else {
        eprintln!("Skipping mpp_import_smoke: pass a path or set RUSTYPROJECT_SAMPLE_MPP");
        return;
    };

    let path = PathBuf::from(path);
    let snapshot = mpp_import::load_mpp(&path).unwrap_or_else(|err| {
        eprintln!("MPP import failed for {}: {err}", path.display());
        std::process::exit(1);
    });

    assert!(
        !snapshot.tasks.is_empty(),
        "MPP import should produce at least one task"
    );
    assert!(
        snapshot.start_date <= snapshot.end_date,
        "project date range should be valid"
    );

    println!(
        "Imported {} tasks from {}",
        snapshot.tasks.len(),
        path.display()
    );
}
