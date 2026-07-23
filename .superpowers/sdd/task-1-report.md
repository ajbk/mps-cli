# Task 1 Report: Shared Flashcard Domain Contract

## Status

Task 1 is implemented on branch `feat/flashcards`. The existing Task 1
commit is:

`45bffeb feat: add flashcard domain contracts`

## Files

The commit contains only:

- `Cargo.toml` — registers `crates/mps-flashcards` in the workspace.
- `crates/mps-flashcards/Cargo.toml` — declares the shared serde and error
  dependencies.
- `crates/mps-flashcards/src/lib.rs` — defines the serializable flashcard
  domain types, locked style profile, draft validation, transition guard, and
  unit tests.
- `crates/mps-flashcards/src/catalog.rs` — defines `CatalogExercise` and
  `CanonicalCatalog::load_json` / `find` for the derived export at
  `data/reference/mps_database_v1_core.json`.

Unrelated visual, workbook, documentation, and source-of-truth changes remain
unstaged and uncommitted.

## Implementation review

- `FlashcardStatus` contains the six requested statuses and derives serde
  serialization/deserialization.
- `FlashcardCard`, `VisualBrief`, and `AssetReview` contain the requested
  fields and derive serde serialization/deserialization.
- `validate_new_draft` rejects empty source IDs, missing canonical exercises,
  non-draft status, and any style profile other than
  `mono-gesture-ink-pilates-v1`.
- `can_transition` rejects `published -> draft` and permits only the defined
  forward/review workflow transitions.
- `CanonicalCatalog` reads `tables.exerciseMaster`, preserving workbook
  source-row IDs and catalog metadata rather than introducing another catalog
  source.

## Tests and checks

- `cargo test -p mps-flashcards`: attempted; unavailable because `cargo` is
  not installed or present on `PATH`.
- `cargo fmt --all --check`: attempted; unavailable for the same reason.
- `git diff --check`: passed.
- Independent JSON parse and lookup check: passed for the 429-row canonical
  export and `source_chair_achilles_stretch_row_4`.

## TDD evidence

The unit tests were written first in `src/lib.rs` for the valid draft, empty
exercise ID, invalid published-to-draft transition, visual brief JSON
round-trip with the locked style and cheek accent, and canonical catalog
lookup. The focused red test command was attempted before implementation, but
could not reach compilation because the Rust toolchain is unavailable. The
implementation was then added to make those tests pass when run in a Rust
environment.

## Concerns

The focused Rust test suite and formatter still need to be executed in an
environment with Cargo/rustfmt installed. No unrelated working-tree changes
were staged or modified by the Task 1 commit.

## Review Fix Report

The review findings were addressed in the Task 1 domain files:

- Added `validate_visual_brief`, rejecting any style profile other than
  `mono-gesture-ink-pilates-v1`, with focused valid and invalid tests.
- Added explicit serde names for all `FlashcardStatus` variants so they match
  the planned persistence values: `draft`, `generating`, `needs-review`,
  `revision-requested`, `approved`, and `published`.
- Added `asset_version` to `AssetReview` and a focused serialization test so a
  review identifies both the asset and the reviewed revision.
- Removed the misleading `Serialize` and `Deserialize` derives from
  `CanonicalCatalog`; its public JSON loader remains unchanged and its
  in-memory shape is no longer advertised as the source export shape.

### Commands and results

- `cargo test -p mps-flashcards` — unavailable: `cargo` is not installed or
  present on `PATH` (exit 127).
- `cargo fmt --all --check` — unavailable: `cargo` is not installed or
  present on `PATH` (exit 127).
- `git diff --check` — passed.
- Independent canonical JSON parse and lookup check — passed for the 429-row
  export and `source_chair_achilles_stretch_row_4`.
