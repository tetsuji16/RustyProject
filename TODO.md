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
- [x] WBS code column in the task table
- [ ] Dependency and calendar rule expansion
  - [x] Weekend-aware calendar normalization
  - [ ] Dependency lag and richer link types

## P1
- [ ] Scroll/pan/layout improvements
  - [x] Scrollable gantt surface
  - [x] Zoom control
  - [x] Splitter drag for table width
  - [ ] Full pan gestures
- [x] UI state persistence
- [x] Undo/redo
- [ ] Dependency link editing

## P2
- [ ] Regression tests and snapshot parity tests
  - [x] Schedule and calendar unit tests
  - [x] JSON project round-trip test
  - [x] MPP importer smoke test
  - [ ] Java/Rust snapshot parity harness
- [ ] UI smoke tests
