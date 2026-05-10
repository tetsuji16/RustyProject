# ProjectLibre Parity Handoff

This TODO is a handoff guide for the next LLM. Java is the source of truth. If Rust and Java disagree, prefer Java.

## Read First

Read these Java files first. They are the oracle for behavior, layout, ordering, and empty-cell rules:

- `original files of projectlibre/projectlibre_ui/src/com/projectlibre1/pm/graphic/spreadsheet/renderer/TaskIndicatorsComponent.java`
- `original files of projectlibre/projectlibre_ui/src/com/projectlibre1/pm/graphic/spreadsheet/renderer/IndicatorsRenderer.java`
- `original files of projectlibre/projectlibre_ui/src/com/projectlibre1/pm/graphic/spreadsheet/renderer/NameCellComponent.java`
- `original files of projectlibre/projectlibre_ui/src/com/projectlibre1/pm/graphic/spreadsheet/SpreadSheetColumnModel.java`
- `original files of projectlibre/projectlibre_ui/src/com/projectlibre1/pm/graphic/spreadsheet/renderer/SpreadSheetRowHeaderRenderer.java`
- `original files of projectlibre/projectlibre_ui/src/com/projectlibre1/pm/graphic/spreadsheet/renderer/SpreadSheetNameCellRenderer.java`
- `original files of projectlibre/projectlibre_ui/src/com/projectlibre1/pm/graphic/spreadsheet/renderer/SpreadSheetNameCellEditor.java`
- `original files of projectlibre/projectlibre_ui/src/com/projectlibre1/pm/graphic/GraphicConfiguration.java`
- `original files of projectlibre/projectlibre_ui/src/main/resources/configuration.xml`
- `original files of projectlibre/projectlibre_ui/src/main/resources/view.xml`
- `original files of projectlibre/projectlibre_model/src/com/projectlibre/core/field/NormalTask.java`
- `original files of projectlibre/projectlibre_model/src/com/projectlibre/core/field/Task.java`

Read these Rust files next. They are the current implementation to compare against Java:

- `src/ui/task_table.rs`
- `src/ui/gantt_chart.rs`
- `src/ui/gantt_view.rs`
- `src/bridge.rs`
- `src/model.rs`
- `src/mpp_import.rs`

Do not treat Rust approximations as authoritative if they conflict with Java.

## Current Goal

The active target is the Gantt task spreadsheet and its shared display model:

- `Indicators`
- `Name`
- `duration`
- `start`
- `finish`
- `predecessors`
- `resourceNames`
- row/header geometry
- gantt bar annotation

## Exact Parity Tasks

### Indicators

- Match Java exactly.
- Use the same conditions, the same order, and the same empty-cell behavior.
- Show no fallback `information` icon when Java leaves the cell empty.
- Match the Java tooltip text and icon ordering.

### Name Cell

- Match Java exactly.
- Reproduce expand/collapse icon placement.
- Reproduce indent logic, leaf handling, lazy/fetched handling, and hit testing.
- Match editor behavior, including selection and click regions.

### Formatting

- Use Java-derived strings whenever available.
- Keep `duration`, `start`, `finish`, `predecessors`, and `resourceNames` aligned with Java.
- Bar annotation should use `resourceNames`, not task name.

### Geometry

- Follow `configuration.xml` and `GraphicConfiguration`.
- Use the Java values for row height, row header width, column header height, and gantt bar offsets.
- Do not invent new widths or a new column order.

### Shared Model

- Keep the Rust model and bridge aligned with Java output.
- Remove Rust-only fallback visuals and formatting paths once Java data is available.

## Do Not Do

- Do not introduce new Rust-specific display rules.
- Do not change unrelated chrome or ribbon behavior unless it blocks parity.
- Do not touch `sample data/` contents.
- Do not use an `information` fallback icon when Java leaves the cell empty.
- Do not invent a new column order or new widths.
- Do not optimize for elegance before parity.

## Working Order

1. Read the Java files listed above.
2. Compare the current Rust implementation to those Java sources.
3. Fix only the differences that affect visible parity or exact string output.
4. Remove Rust-only fallback logic after Java-equivalent behavior exists.
5. Keep the sample `Commercial construction project plan.mpp` as the comparison file.

## Verification

Use these checks after every meaningful change:

- `cargo check`
- `cargo test`
- Compare the sample `.mpp` rendering against Java for the same locale and file.

Confirm these exact parity points:

- indicator icons visible per row
- name indent and toggle position
- `duration / start / finish / predecessors / resourceNames` strings
- gantt annotation text source
- row height and header height

## Cleanup

After parity is stable:

- delete dead Rust-only code paths
- add regression tests for the indicator matrix
- add regression tests for name cell layout
- add regression tests for formatting strings
- add a snapshot/parity harness only after the display rules are stable

## Status

- [ ] Indicators match Java exactly
- [ ] Name cell behavior matches Java exactly
- [ ] Visible text formatting matches Java-derived strings
- [ ] Geometry matches `configuration.xml`
- [ ] Bar annotation uses `resourceNames`
- [ ] Rust-only fallback visuals are removed
- [ ] Parity snapshot harness exists
- [ ] Regression tests cover indicator, name, and formatting behavior

## Ribbon Notes

- `File / Task / Resource / View` ribbon chrome was re-aligned to Java band structure.
- Resource and View tab verification screenshots were captured from the Rust app after the latest layout pass.
- Current verification files: `resource-window-final.png` and `view-window-final.png`.
