# MPS Flashcards

Use these cards to quickly reload the project model before coding or reviewing.

For exercise-teaching cards grouped by Reformer, Mat, Stand / Standing, and
Chair, see [Pilates Exercise Flashcards](EXERCISE_FLASHCARDS.md).
For the generated printable deck with cartoon artwork, open
[Exercise Flashcard Deck](exercise-flashcard-deck.html).

## Domain

**Q:** What is MPS?
**A:** Memellow Programming System, a deterministic Teaching Intelligence System
for generating Total Body Pilates class plans.

**Q:** What is the most important architecture rule?
**A:** No LLM in the generation loop. The canonical plan is produced by Rust
rules, scoring, and SQLite-backed metadata.

**Q:** What is a Movement Experience?
**A:** The felt movement outcome the class is designed to improve, such as
Shoulder Freedom, Happy Hips, or Spine Reset.

**Q:** What are the seven journey phases?
**A:** ARRIVE, PREPARE, BUILD, INTEGRATE, CHALLENGE, TRANSFER, RESET & RETEST.

**Q:** What makes BUILD special?
**A:** It is the main intervention phase and must include Prime exercises.

**Q:** What is the 80/20 rule?
**A:** The class should feel about 80 percent achievable and 20 percent
challenging through level fit, regressions, and challenge-phase design.

**Q:** What is the difference between observation and safety?
**A:** Observations modify strategy. Safety constraints exclude or modify
exercise selection.

**Q:** What are movement contexts?
**A:** Mat and Standing. They support arrival, transfer, reset, and retest work,
but they are not primary apparatus request values.

## Architecture

**Q:** What does `mps-core` own?
**A:** Domain types, validation, scoring, phase allocation, strategy building,
class generation, and markdown rendering.

**Q:** What does `mps-db` own?
**A:** SQLite migrations, seed loading, and the repository implementation that
maps database rows into core records.

**Q:** What does `mps-cli` own?
**A:** The `mps` command with `init-db`, `seed`, and `generate` subcommands.

**Q:** What does the web prototype own?
**A:** A static browser UI and JavaScript port of the deterministic generation
logic for quick demos.

**Q:** What is the repository boundary?
**A:** `mps-core` depends only on the `MpsRepository` trait, not on SQLite.

**Q:** What is the seed database used for?
**A:** Strategies, observation modifiers, exercises, roles, objectives, cues,
contraindications, and benchmarks.

## Testing

**Q:** What is the baseline command?
**A:** `cargo test`

**Q:** What Windows environment setting should be used first?
**A:** `$env:CARGO_TARGET_DIR="$env:TEMP\mps-target"`

**Q:** What behavior needs integration coverage?
**A:** End-to-end generation against the real SQLite seed data.

**Q:** What equipment behavior must never regress?
**A:** Reformer and Chair exercises must respect the selected primary apparatus,
while Mat and Standing remain available only for context phases.

## Product

**Q:** Who is the primary user?
**A:** A Pilates instructor who wants structured class programming support.

**Q:** What should the output help the instructor do?
**A:** Teach the class journey, explain why each exercise was selected, cue
movement quality, and retest the target movement experience.

**Q:** What is future LLM enhancement allowed to do?
**A:** Polish text after generation, without changing exercise selection,
phase structure, safety rules, benchmarks, or timing.
