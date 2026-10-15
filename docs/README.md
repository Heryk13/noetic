# Noetic — Documentation

All project documents, in reading order. Each one builds on the previous one.

| # | Document | Format | What it answers |
| --- | --- | --- | --- |
| 0 | [Vision](00_vision.md) | Markdown | What we are building and why: living wildlife, the four species, how animals decide |
| 1 | [Requirements Definition](01_requirements_definition.md) | Markdown | What the system must do: functional (FR-xxx) and non-functional (NFR-xx) requirements, data, screens, milestones, open issues |
| 2 | [Basic Design](02_basic_design.xlsx) | Excel | How the whole system is built: architecture, crates, function list (F-xxx), processing order (PRC-xx), data, file formats, external interfaces, UI policy |
| 3 | [External Design](03_external_design.xlsx) | Excel | What the user sees: every screen with annotated screenshots and numbered items (SCR-nn-mm), transitions, messages, shortcuts |
| 4 | [Internal Design](04_internal_design.xlsx) | Excel | How the Rust program is organized: workspace, modules (MOD-xxx), types, Bevy ECS, state machines, sequences, errors, tests |
| 5 | [Detailed Design](05_detailed_design.xlsx) | Excel | Every function (API-xxx): signature, processing, errors; item → API map; algorithms (ALG-xx); data schema; unit tests |

The Excel files are made for LibreOffice Calc (they also open in Excel). Each workbook starts with a Cover, a Revision History and a Contents sheet with links.

## Other material

- [prototype/noetic_prototype.html](prototype/noetic_prototype.html) — clickable HTML prototype (v21, English UI, Bevy Feathers default theme). Open it in a browser. The screenshots in the External Design come from this file.
- [diagrams/](diagrams/) — Mermaid sources (`.mmd`) and rendered images of the module, class, state and sequence diagrams used in the Internal Design, plus the decision diagram of the Vision.
- [images/system_overview.png](images/system_overview.png) — system overview (source: `system_overview.svg`, rendered at 1344 × 516) used in the Requirements Definition and the Vision.

## Traceability

The IDs connect the documents from requirement to code:

```
FR-105 (Requirements)
  → F-202 (Basic Design · Function List)
    → SCR-03-03 / SCR-03-04 (External Design · item on the screenshot)
      → API-BRN-007 set_branch_weight (Detailed Design) → ALG-07 (algorithm)
        → noetic_editor::brain::edit (Internal Design · MOD-xxx)
          → UT-004… (Detailed Design · unit tests)
```

| Prefix | Meaning | Defined in |
| --- | --- | --- |
| FR-xxx / NFR-xx | Functional / non-functional requirement | Requirements Definition |
| F-xxx · PRC-xx · ENT-xx · FMT-xx · IF-xx | Function · processing step · data entity · file format · external interface | Basic Design |
| SCR-nn / DLG-nn / POP-nn, item SCR-nn-mm | Screen / dialog / popup and its numbered items | External Design |
| MSG-xxx · KEY-xx · UI-xx | Message · shortcut · common UI rule | External Design |
| MOD-xxx · TYP-xx · CMP/RES/EVT/STA/SYS-xx · ERR-xx | Module · type · Bevy ECS parts · error type | Internal Design |
| API-<AREA>-nnn · ALG-xx · UT-xxx | Function spec · algorithm · unit test | Detailed Design |

In code, the doc comment of every public item starts with its API ID, and every editor UI entity carries `UiItem("SCR-nn-mm")`, so tests and reviews can go from the screen to the code and back.

## Decisions recorded so far

- Everything is written in **Rust**; the engine is **Bevy**.
- Editor UI: **bevy_ui + bevy_feathers with the default dark theme**, unchanged; UI language **English**.
- Editor view is **2D**; 3D is seen inside the game through **remote debug** (BRP, `127.0.0.1:15702`).
- One **`.noetic.ron`** file per species, loaded by the game through **`noetic`**.
- Two crates on crates.io: **`noetic`** (the plugin games add, re-exports the core) and **`noetic_core`** (no Bevy). The editor ships as a binary per OS on GitHub Releases, never from crates.io; a game downloads the one matching its `noetic` version into `tools/`.
- License **MIT OR Apache-2.0**.

Open questions are listed in the Requirements Definition §10.1.

## Updating the documents

- Markdown files: edit directly.
- Excel files: edit in LibreOffice Calc and add a line to the Revision History sheet.
- When a screen changes, retake its screenshot, keep the item numbers stable (add new numbers at the end) and update the item table and the Item → API map.
