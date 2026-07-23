# MPS Database v1 Integration

`data/reference/MPS_Database_v1_Core.xlsx` is the canonical exercise source for
MPS flashcards and future database curation. The JSON file at
`data/reference/mps_database_v1_core.json` is a derived, machine-readable export
of that workbook for app and validation workflows. It must not become a second
manually edited source.

## What is included

- 429 exercise-apparatus records
- 204 family relationships
- 165 category relationships
- 301 progression relationships
- 19 exercise families
- controlled vocabularies for apparatus, categories, body regions, and movement taxonomy
- source row and source page references for auditability
- parsed arrays for multi-value fields such as family IDs and source pages

## Why the JSON is a derived export

The workbook has a rich reference taxonomy, but the MPS generator's canonical
`exercises` table also requires fields that are currently blank in
`Exercise_Master`: roles, objectives, movement systems, movement experiences,
level bounds, difficulty, duration, cues, regressions, progressions, and safety
constraints.

The export preserves the workbook data without inventing those values. It is
ready for curation and later transformation into the canonical SQLite seed.
When the workbook changes, regenerate the JSON export rather than editing it by
hand. The existing `data/seed/mps_seed.sql` remains unchanged until that
curation is complete.

## Apparatus mapping

The current MPS model can map these source values immediately:

| Source apparatus | MPS key |
|---|---|
| Mat | `mat` |
| Reformer | `reformer` |
| Chair | `chair` |

These source values need a model decision before generator import:

- Trap Table/Cadillac
- Step Barrel
- Ladder Barrel/High Barrel
- Pedo-Pull
- Props/Other

## Recommended next curation pass

1. Decide which apparatus become first-class MPS equipment and which remain reference-only.
2. Normalize exercise names and aliases across apparatus.
3. Fill the MPS generator fields for the first target subset.
4. Add roles, objectives, cues, safety tags, levels, and duration.
5. Generate SQLite seed rows and browser data from the curated source.
6. Run the existing Rust, CLI, and web quality gates.
