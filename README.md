# ProjectLibre Gantt - Rust

ProjectLibre Gantt - Rust is a parity-first desktop rewrite of the ProjectLibre Gantt experience built with Rust and `egui`. The goal is not simply to draw a schedule, but to preserve the character of the original Java application: the ribbon chrome, the task spreadsheet, the Gantt chart, and the data relationships that make the view feel alive.

This repository exists for one reason: to make the ProjectLibre experience feel native in Rust without losing the layout, behavior, or data fidelity that users expect from the original application.

## Highlights

- Side-by-side task table and Gantt chart in a single desktop window
- Java bridge-based MPP / XML / POD import path backed by MPXJ
- Preserved task hierarchy, dependencies, resource names, and timeline editing
- Ribbon chrome, tab layout, and icon placement tuned for visual parity with ProjectLibre
- Persistent project documents with undo / redo support
- Bundled sample project for quick launch and comparison

## What Makes It Different

Most rewrites stop at "close enough". This one is stricter.

- The UI is designed around parity, not reinterpretation.
- The task grid and Gantt canvas are meant to match the original workflow, not just the same general concept.
- Import and save behavior are structured so the Rust app can keep up with the data the Java app understands.
- Visual verification matters, so the repo keeps dedicated ribbon screenshots for resource and view tabs.

## Quick Start

Run the app with the bundled sample project:

```bash
cargo run -- "sample data/Commercial construction project plan.xml"
```

If you omit the argument, the bundled sample project opens automatically.

## Data Flow

The app follows a simple, traceable path:

1. A project is loaded from the bundled sample file, a saved JSON project document, or an MPP / XML / POD file path you provide.
2. When an MPP or XML file is requested, the Rust app launches a Java bridge that reads the file through MPXJ and returns a structured snapshot. POD files are unpacked in Rust first, then handed off as embedded XML.
3. The snapshot drives the ribbon, spreadsheet, and chart rendering.
4. User edits such as selection, collapsing summaries, resizing the chart/table split, and undo/redo are stored in the project document model.
5. Saved documents can be reopened later without re-importing the MPP source.

## Why This Exists

This repo is a parity-first Rust port. That means the implementation favors matching ProjectLibre's behavior and presentation over inventing new UI rules. It is especially focused on:

- task table formatting
- chart layout and annotations
- ribbon structure and icon placement
- bridge-backed MPP import fidelity
- stable window chrome and tab behavior

## Implementation Notes

- The main desktop shell is built with `eframe` / `egui`.
- Japanese fonts are installed when available so the UI can render localized labels cleanly on Windows.
- The default window opens at a fixed desktop-friendly size and starts from the bundled sample project unless you pass a different path.
- The import pipeline downloads its Java-side dependencies on demand into a temp cache, so a fresh checkout can still load MPP files without committing third-party jars into the repo.
- Project state is serialized as JSON, which makes it easier to inspect, diff, and restore during parity work.

## Screenshots

Main workspace:

![ProjectLibre Gantt - Rust](artifacts/readme_screenshot_clean.png)

Resource ribbon parity:

![ProjectLibre Resource Ribbon](artifacts/ribbon-parity/resource-window-final-clean.png)

View ribbon parity:

![ProjectLibre View Ribbon](artifacts/ribbon-parity/view-window-final-clean.png)

## Notes

- The sample MPP file lives at `sample data/Commercial construction project plan.mpp`.
- The sample XML file lives at `sample data/Commercial construction project plan.xml`.
- The sample POD file lives at `sample data/Commercial construction project plan.pod`.
- The import path uses the Java bridge so the Rust view can stay aligned with ProjectLibre's original data model.
- The project document format stores the selected task, collapsed summaries, and current layout widths alongside the snapshot itself.
- This repository is still evolving, but the current focus is stable visual and behavioral parity.

