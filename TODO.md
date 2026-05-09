# Gantt TODO

## P0
- [x] Rust-only gantt runtime and basic editing
- [x] Project open/save/import/export
  - [x] Open/save JSON project document
  - [x] Import/export aliases on the toolbar
  - [x] Native file dialogs
  - [x] MPP import bridge via MPXJ
- [x] Task name, indent, predecessor, add/delete editing
  - [x] Inspector edits name, indent, and predecessors
  - [x] Add child/sibling and delete task actions
- [ ] Task table parity with ProjectLibre screenshot
  - [x] Remove Rust-only ID/WBS from the task table
  - [x] Match resource names from MPP import
  - [x] Match task date/time formatting and labels
  - [x] Prefer Java-derived task labels from the MPP bridge
  - [ ] Verify indicator and expand/collapse parity on the sample project
- [ ] Dependency and calendar rule expansion
  - [x] Java-like calendar normalization defaults
  - [ ] Dependency lag and richer link types
  - [x] Use ProjectLibre/Java duration formatting for imported tasks

## P1
- [ ] Scroll/pan/layout improvements
  - [x] Scrollable gantt surface
  - [x] Zoom control
  - [x] Splitter drag for table width
  - [ ] Full pan gestures
- [x] UI state persistence
- [x] Undo/redo
- [ ] Remove Rust-only inspector/edit panel state
  - [x] Delete or rewire the unused task inspector methods and fields
- [ ] Dependency link editing
- [x] ProjectLibre-style gantt visualization pass
  - [x] ProjectLibre-like chrome and task table density
  - [x] Default sample `.mpp` autoload on startup
  - [x] ProjectLibre image assets reused from the original Java project
  - [x] Ribbon grouping and spacing parity pass
  - [x] App bar project selector parity
  - [x] Java ribbon top-right selector parity
  - [x] Remove remaining Rust-only chrome widgets

## P2
- [ ] Regression tests and snapshot parity tests
  - [x] Schedule and calendar unit tests
  - [x] JSON project round-trip test
  - [x] MPP importer smoke test
  - [ ] Java/Rust snapshot parity harness
- [ ] UI smoke tests
