# Contributing

How every change in this project is made. Follow the loop for each feature, screen or fix.
These are `bevy-starter`'s rules, adapted to a Cargo workspace whose design is written down
in `docs/`.

Every rule below exists for a reason. When the rule and its reason disagree, the reason
wins: do what makes the code better, and write the exception down in a comment where it
lives, saying why the usual rule does not fit there. An exception with its reason beside
it is a decision. An exception without one is drift.

## The loop

1. **Plan**: find where the change belongs in the design: its module (`MOD-xxx`) in the
   Internal Design and its functions (`API-xxx`) in the Detailed Design. Code is organized by
   feature, not by who wrote it: each editor area or feature is a plugin in its own file.
2. **Write** following the [code conventions](#code-conventions).
3. **Test** the logic (see [Tests](#tests)).
4. **Fix and check**: `just fix`, then `just check`.
5. **Run**: the editor, or the example that shows the change, and try it.
6. **Docs**: update `README.md` if features, commands or requirements changed, and the design
   document the change departs from, with a line in its Revision History
   (see [Updating the documents](docs/README.md#updating-the-documents)).
7. **Commit** (see [Commits](#commits)).

## Code conventions

### Workspace

- `noetic_core` never depends on Bevy, and `noetic` depends only on `noetic_core` and Bevy and
  re-exports `noetic_core`. Games and the editor depend on `noetic` only
  (Internal Design › 4 Dependencies, DEP-01 to DEP-07).
- Inside the editor, feature modules (`brain`, `world`, `species`, `individuals`, `io`,
  `remote`) never import each other: they talk through Commands, Events and shared Resources.
- `#![forbid(unsafe_code)]` in every crate.

### File layout

Every `.rs` file follows the same order, skipping the sections it doesn't have:

1. imports: external crates first, a blank line, then crate-local ones (`use super::...`)
2. child modules (`mod ...;`)
3. `Config`: tunable constants
4. `Plugin`: the `plugin` function, a summary of what the file does
5. in a shared file, what it hands to other files, in a section named after it
   (`Widgets` in `theme.rs`), not under `Helpers`
6. types: `State`, `Events`, `Components`, `Resources`
7. `Systems`: in the order they run
8. `Helpers`: pure functions used by the systems, next to the tests that cover them
9. `Tests`

### Splitting files

- Split by feature, never by kind of code: `perception.rs` and `reaction.rs`,
  not `components.rs` and `systems.rs`.
- Code that several features share is expected, and it gets its own file at the top of
  `src/`, named after its domain: `theme.rs`, `i18n.rs`, `rng.rs`. Being at the top of
  `src/` is already what marks it as shared, so it needs no `common/` or `shared/`
  folder around it. Names like `common`, `utils` or `global` describe status instead of
  content: everything fits in them and nothing ever leaves.
- Move something there once a third feature needs it. With one user it stays inside that
  feature; with two, it stays with the first and waits.
- Keep a feature in one file while it reads as one thing.
  Split when a part makes sense on its own, usually around 300 to 400 lines
  or when you find yourself scrolling to look for "the part about X".
  That part becomes its own file.
- A split module is `brain.rs` next to a `brain/` folder with the parts (`brain/edit.rs`),
  never `mod.rs`; clippy's `mod_module_files` fails `just check` on one.
- Unit tests stay in the file they test, `tests/` is only for integration and golden tests.

### Separators

Every `.rs` file splits its sections with separator lines ending at column 80.
The character tells the kind of boundary:

```rust
// ==== Systems ================================================================
// ---- think: needs, perceive, memory, fear, threat, choose -------------------
```

- `====`: a section of the file. Title Case.
- `----`: a group inside a section, lowercase, may carry a short note.
  Inside a function body it is indented with the code and still ends at column 80.

### Comments

- Comments explain *why*, not *what*. Code that already says what it does gets no comment.
- Every system and type has a `///` doc comment written as a sentence:
  capitalized, ending with a period (`/// Picks the next reaction by weight.`).
- The doc comment of every public item that implements a design ID starts with that ID
  (`/// API-COR-007 Picks one entry by weight.`), so a review can go from the design to
  the code and back.
- Plain `//` comments are lowercase, without a final period.

### Bevy patterns

- One plugin per editor area or feature: `pub(super) fn plugin(app: &mut App)`.
- Area views get `DespawnOnExit(Area::X)`, area systems get `.run_if(in_state(Area::X))`.
  Whatever a state spawns, its exit despawns.
- An observer gets the same `run_if` as the state its buttons live in, so it never runs, nor
  asks for that state's resources, anywhere else.
- A control carries a component naming what it does, and one observer per file reads it off
  `activate.entity`. Every editor control also carries `UiItem("SCR-nn-mm")` and a `Tooltip`,
  matching its item in the External Design.
- Every undoable edit is a `Command` sent through `EditorCommand`, never a direct write to
  `Project`, so undo (FR-707) covers it for free.
- Every Feathers widget is created through the helpers in `theme.rs`, with the default dark
  theme unchanged: the Feathers API moves between Bevy versions, and one file absorbs it.
- Anything that must keep moving while the simulation is paused (the UI, tooltips, toasts)
  counts `Time<Real>`. The simulation counts its own clock, which pause and speed control.
- State that belongs to an entity is a component on it, not a global resource.
- Exactly one entity: `Single<...>`. Zero or more: `Query<...>`.
- Ask for the minimum access: `&` to read, `&mut` only to write.
- When a system reads what another writes, make the order explicit with the system sets of
  the Internal Design (`NoeticSet`, `EditorSet`), `.after(...)` or `.chain()`.
- Tunable values are named constants in the `Config` section, never magic numbers.
  A value that follows from other constants is derived from them in its own group, never
  restated: two spellings of the same number drift apart the day one of them changes.
- Dev-only code goes behind `#[cfg(feature = "dev")]`, remote debug behind `debug`.

### Determinism and errors

- Randomness only through the seeded `Rng` of `noetic_core::rng`, never `thread_rng`, and no
  global mutable state: same seed and same inputs give the same result (NFR-01).
- `BTreeMap` (or `IndexMap`) wherever iteration order changes behavior; `HashMap` only for
  lookups.
- Prefer the call that can say no over the one that panics: `get()` over `[0]`, `?` over
  `unwrap()`, `saturating_sub` where a `usize` could go below zero. No `unwrap` or `expect`
  outside tests and `main`; `noetic_core` denies `clippy::unwrap_used`.
- A bad file or a broken graph ends in a message, never a crash: the runtime logs and leaves
  that species idle, the editor shows a `MSG-xxx` (NFR-12).
- Errors are typed `thiserror` enums per area; `anyhow` only at the top of the editor binary.
- File types use `#[serde(deny_unknown_fields)]`, and every `.noetic.ron` carries
  `schema_version`: an old file is migrated on load, never silently broken.

### Text

- Every word the user reads goes through the localization table (`i18n.rs`). Code names the
  key, never the words. The UI language is English; other languages only add tables.
- A key belongs to one place in the UI, even when its English repeats another's: another
  language may need two different words.

### Naming

- Systems and observers: verb + noun (`spawn_world_view`, `apply_editor_command`).
- Constants: `SCREAMING_SNAKE_CASE`, with the unit in the name (`THINK_HZ`, `GRID_CELL_M`).
- Editor UI entities get `Name` from the External Design object name (`btnPlayPause`).

## Tests

- Logic lives in pure functions: plain values in, plain values out.
  Systems stay thin: read from the ECS, call the function, write the result.
- Tests sit at the bottom of the same file, in `#[cfg(test)] mod tests` under `// ==== Tests ====`.
- The cases come from the Detailed Design › Unit Tests, and each test names its `UT-xxx`.
- Compare floats with a small tolerance instead of `assert_eq!`, except in the golden
  determinism test, which is bit-exact on the same platform.
- Every test asserts something. A test with an empty body passes and proves nothing, which is
  worse than no test: it reports green while the code it names is unprotected.
- The name says what the test guarantees, as a sentence about the subject
  (`branch_weights_always_sum_to_100`), not what the function under test is called.
- Write the times and sizes a test uses as expressions over the constants
  (`STEP_SECS / 2.0`), so the test follows the code when a constant changes, and pick them
  away from the boundary between two phases, where `f32` rounding decides the answer.
- A test that touches the disk works in a folder of its own under `std::env::temp_dir()`,
  emptied first, so tests running in parallel never share files.

## Commits

- [Conventional Commits](https://www.conventionalcommits.org/), in English, imperative, short title:
  `feat: add needs tick`, `fix: ...`, `refactor: ...`, `test: ...`, `style: ...`, `docs: ...`, `build: ...`.
- One change per commit.
- `git commit --amend` only before pushing.

## Dependencies

- Check first if Bevy or the standard library already does it.
- Then prefer a well-known crate to code written by hand: it is code someone else tests and
  maintains, and what is left here reads as names instead of arithmetic. A crate qualifies when
  it is widely used (millions of downloads, depended on by other well-known crates), maintained
  (a release in the last year) and permissively licensed (MIT or Apache-2.0). A small or
  little-known crate does not: write the code instead, since a crate nobody watches is a risk
  nobody sees.
- Every crate has to move to the next Bevy release before Noetic can, so a crate that depends
  on Bevy costs more than one that does not.
- The allowed crates per workspace member are listed in Internal Design › 2 Workspace.
  `cargo-deny` checks the layering and every dependency license; review a new license before
  allowing it.
