# ProjectLibre Gantt - Rust

ProjectLibre Gantt - Rust is a desktop rewrite of the ProjectLibre Gantt experience built with Rust and `egui`. The goal is not just to render a schedule, but to preserve the feel of the original Java application: the ribbon chrome, the task spreadsheet, the Gantt chart, and the data relationships that make the view useful.

## Highlights

- Side-by-side task table and Gantt chart
- Java bridge-based import pipeline for MPP files
- Preserved task hierarchy, dependencies, resource names, and editable timeline behavior
- Ribbon chrome and tab layout tuned for visual parity with ProjectLibre
- Bundled sample project for quick launch and comparison

## Quick Start

Run the app with the bundled sample project:

```bash
cargo run -- "sample data/Commercial construction project plan.mpp"
```

If you omit the argument, the bundled sample project opens automatically.

## Why This Exists

This repo is a parity-first Rust port. That means the implementation favors matching ProjectLibre's behavior and presentation over inventing new UI rules. It is especially focused on:

- task table formatting
- chart layout and annotations
- ribbon structure and icon placement
- bridge-backed MPP import fidelity

## Screenshots

Main workspace:

![ProjectLibre Gantt - Rust](artifacts/readme_screenshot_clean.png)

Resource ribbon parity:

![ProjectLibre Resource Ribbon](artifacts/ribbon-parity/resource-window-final-clean.png)

View ribbon parity:

![ProjectLibre View Ribbon](artifacts/ribbon-parity/view-window-final-clean.png)

## Notes

- The sample MPP file lives at `sample data/Commercial construction project plan.mpp`.
- The import path uses the Java bridge so the Rust view can stay aligned with ProjectLibre's original data model.
- This repository is still evolving, but the current focus is stable visual and behavioral parity.

