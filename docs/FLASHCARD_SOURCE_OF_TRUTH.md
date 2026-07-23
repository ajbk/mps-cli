# Flashcard source of truth

MPS flashcard production has two canonical inputs:

1. Visual language: `mono-gesture-ink-pilates-v1`
2. Exercise catalog: `data/reference/MPS_Database_v1_Core.xlsx`

## Visual contract

All new flashcard artwork uses the mono gesture-ink profile and the current
`teacher-01` character reference recorded in
`data/reference/pilates_visual_manifest.json`.

The exact outfit is fixed: an off-white thin-strap cropped Pilates camisole
with dark charcoal high-waisted mid-thigh biker shorts. Keep the teacher's
shoulder-length layered bob, round dark glasses, and real body proportions.
Add a subtle natural dusty-rose pink tint to both cheeks. This is the only
intentional color accent in the mono gesture-ink profile; it must stay soft and
must not become full-color makeup styling.

## Data contract

The workbook is the authoritative exercise catalog. The JSON file
`data/reference/mps_database_v1_core.json` is a derived export for code that
cannot read `.xlsx` directly. It preserves source rows, source pages,
apparatus, families, categories, body regions, movement taxonomy, and
progression relationships.

The teaching-card copy remains a separate curated presentation layer. It may
add teaching questions, cues, regressions, and progressions, but it must not
invent or overwrite the workbook's exercise/apparatus identity.

## Cleanup rule

Do not add another visual style profile or another manually maintained exercise
catalog. New styles and data sources require an explicit change to this
contract first.
