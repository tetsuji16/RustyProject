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

- [x] Indicators match Java exactly
- [x] Name cell behavior matches Java exactly
- [x] Visible text formatting matches Java-derived strings
- [x] Geometry matches `configuration.xml`
- [x] Bar annotation uses `resourceNames`
- [x] Rust-only fallback visuals are removed
- [x] Parity snapshot harness exists
- [x] Regression tests cover indicator, name, and formatting behavior

### Current Progress

- `cargo check` passes on the current Rust changes.
- `cargo test` passes end-to-end; the smoke test now skips cleanly when no `.mpp` path is provided.
- The Gantt date bar now follows the Java `TimeScaleComponent`-style top/bottom split with month and day boundary placement driven from the same date spans.
- The task table expand/collapse hit testing now uses the Java-style icon hit box expansion, and the icon x-position now matches the actual name-cell layout instead of the indicators column.
- The task table indicator subset now follows the Java-supported order for notes, completed, parent assignment, and missed deadline, with no fallback information icon.
- `start_label` and `finish_label` now use Java-shaped fallback strings when imported text is absent.
- The name column now keeps a Java-style leading gap for the tree icon and renders blank task names as a visible space.
- The Gantt chart now uses the Java `ganttBarYOffset` / `ganttBarHeight` shape timing, and the dependency links now preserve relation/lag data while routing with Java-style bends.
- `status_date` is now part of the Rust snapshot model, so the Java-style status line can be wired through when the import path starts providing it.
- `TaskSnapshot.predecessors` now preserves Java-style relation/lag data through the Rust model, bridge, and MPP importer, while still accepting legacy integer predecessor lists.
- Regression tests were added for the indicator subset and the display-string fallbacks.
- Gantt drag handling now follows the Java pattern more closely: drag motion is preview-only, and the edit is committed on mouse release instead of mutating the snapshot on every pointer move.

## Ribbon Notes

- `File / Task / Resource / View` ribbon chrome was re-aligned to Java band structure.
- Resource and View tab verification screenshots were captured from the Rust app after the latest layout pass.
- Current verification files: `artifacts/ribbon-parity/resource-window-final-clean.png` and `artifacts/ribbon-parity/view-window-final-clean.png`.



## Rust移植 TODO: タスク表とガントチャートのJava準拠化

Java の `task table` と `gantt chart` を source of truth として、Rust 側の見た目・配置・文字列・ヒットテスト・依存描画を一致させること。  
対象は `src/ui/task_table.rs` / `src/ui/gantt_chart.rs` / `src/ui/gantt_view.rs` を中心に、必要なら `src/model.rs` / `src/mpp_import.rs` / `src/bridge.rs` まで広げる。

### タスク表の移植

- `TaskIndicatorsComponent` 相当の指標列を Java と同じ条件順で描画すること。`completed`、`notes`、`parentAssignment`、`missedDeadline` の順序を崩さず、Java で空なら Rust でも空にする。
- 既存の Rust の fallback アイコンや独自の補助表示は、Java と一致しないなら削除すること。とくに「情報」系の代替アイコンを勝手に出さない。
- [x] `NameCellComponent` 相当の名前列を Java 仕様に合わせること。インデント、leaf / plus / minus の切り替え、クリック領域、折りたたみトグルの当たり判定を合わせる。
- [x] 名前列は表示専用ではなく、Java と同じく展開/折りたたみの操作領域を持つ前提で作ること。トグル判定はアイコン周辺に少し余白を持たせる。
- [x] `duration` / `start` / `finish` / `predecessors` / `resourceNames` の表示文字列は、Rust の独自フォーマットではなく Java 側の出力に寄せること。特に `resourceNames` はバー注釈と同じソースに揃える。
- 列順、列幅、ヘッダ高さ、行高さは `configuration.xml` の値を基準にして再確認すること。勝手に新しい幅を足さない。
- サマリー行の背景、選択行の反転、通常行の背景は Java の優先順位に合わせること。Rust 独自の見た目調整を追加する前に、まず Java と同じ配色・強調順にする。

### ガントチャートの移植

- `GanttRenderer` の構成を基準に、月ヘッダ、日ヘッダ、グリッド、週末/非稼働日網掛けを Java と同じ順で描画すること。
- 現状の Rust の単純なバー描画を、Java の通常バー / サマリーバー / マイルストーン描画に寄せること。バーの高さ、角、黒い進捗バー、ラベル位置を合わせる。
- バー注釈は Java と同様に、通常タスクでは `resourceNames`、マイルストーンでは `finish` 系の表示を使うこと。タスク名を注釈に流用しない。
- プロジェクト開始線と、もし利用可能ならステータス日線を Java と同じスタイルで描くこと。線種や色を勝手に変更しない。
- [x] 依存線は `predecessor id` だけの単純接続ではなく、Java の link routing に近い折れ線にすること。少なくとも FS / SS / SF / FF とラグを保持できるデータ構造に拡張する。
- [x] 依存線の矢印、線の折り返し位置、上下の接続方向は、描画上の見た目が Java に近くなるように調整すること。
- バーのヒットテストは Java のドラッグ基準に合わせること。move / resize start / resize end / progress の判定領域を Java と同じ考え方で作る。
- マイルストーンは菱形、通常バーは矩形、サマリーバーは両端の三角形付きの形で表現すること。高さや最小幅も Java の定数に寄せる。

### データモデルと入出力

- [x] `TaskSnapshot.predecessors` を `Vec<usize>` のままにせず、依存種別とラグを持てる型に拡張すること。Java の `Relation` 相当を表現できる形にする。
- `src/mpp_import.rs` と `java_mpp_bridge` の出力は、新しい依存情報を運べるように更新すること。既存の MPP import が壊れないように互換を考える。
- `src/bridge.rs` の Java bridge も、新しいスナップショット形式を読み書きできるように合わせること。古い保存データがあれば migration で吸収する。
- `ProjectSnapshot` の保存形式を変更するなら version を上げること。古い JSON は読めるようにして、表示だけでも壊さない。
- `schedule.rs` の正規化や再計算が依存型の拡張で壊れないか確認すること。少なくとも summary range の再計算と進捗集計は維持する。

### 検証

- `cargo test` が通ることを最低条件にすること。モデル系・保存系・MPP import 系の既存テストを壊さない。
- 画面確認では、Java と Rust を同じサンプルプロジェクトで並べて比較すること。特に以下を重点確認する。
- 指標列のアイコン有無と順番
- 名前列のインデント、トグル位置、テキスト切れ
- `duration / start / finish / predecessors / resourceNames` の文字列
- ガントのバー位置、バー高さ、進捗バー、注釈位置
- 週末/非稼働日背景、月ヘッダ、日ヘッダ、プロジェクト開始線
- 依存線の向き、折れ方、矢印位置
- 変更後は Java 参照画像との差分を見ながら、見た目の差が残る箇所を一つずつ潰すこと。先に大改修しない。
- 仕上げに、指標列、名前列、ガントバーの回帰テストを追加すること。見た目の細部は snapshot か画像比較で守るのが望ましい。

### 作業順

- まず `src/ui/task_table.rs` を Java 準拠に近づける。
- 次に `src/ui/gantt_chart.rs` を Java 準拠に近づける。
- その後に `src/model.rs` / `src/mpp_import.rs` / `src/bridge.rs` を依存情報対応で揃える。
- 最後にテストと比較用の確認を入れて、残った Rust 独自の fallback を削る。
