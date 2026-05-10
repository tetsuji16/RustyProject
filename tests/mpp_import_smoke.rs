#[path = "../src/model.rs"]
mod model;
#[path = "../src/schedule.rs"]
mod schedule;
#[path = "../src/mpp_import.rs"]
mod mpp_import;

use std::path::PathBuf;

fn main() {
    let path = std::env::args_os().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo test --test mpp_import_smoke -- <path-to-mpp>");
        std::process::exit(1);
    });

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
