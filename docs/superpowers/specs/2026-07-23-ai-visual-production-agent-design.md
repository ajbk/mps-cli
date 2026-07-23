# MPS AI Visual Production Agent Design

**Status:** Proposed
**Date:** 2026-07-23
**Scope:** iOS-first PWA plus a Custom ChatGPT App/MCP integration for teacher-facing flashcard production.

## Objective

Allow a Pilates teacher to select an exercise from the canonical MPS catalog and ask an AI assistant to:

1. read the exercise and its apparatus/body/movement metadata;
2. read the MPS visual contract and current teacher reference pack;
3. create a structured visual brief;
4. generate a mono-gesture-ink Pilates image;
5. review the image against exercise and visual rules; and
6. save a draft for human approval.

The assistant is an authoring and review layer. It must not replace the deterministic MPS class-plan generator or become the source of truth for exercise identity.

## Product experience

The PWA is the teacher's workspace:

- **Library-first home:** search and filter exercises from the canonical catalog.
- **Single-scroll editor:** show workbook-locked exercise data first, followed by editable teaching copy, visual status, and review actions.
- **Draft review:** compare the generated image with its visual brief, see validation findings, request a revision, or approve the card.
- **Published deck:** show only cards that passed human review.

The AI conversation lives in ChatGPT through a Custom App/MCP connection. The PWA remains the place to browse, inspect, review, and publish. The PWA may show a pending AI job and a link/instruction to continue the request in ChatGPT.

## System boundary

~~~text
Teacher iPhone PWA
  |  MPS API: library, cards, jobs, review
  v
MPS backend
  |-- canonical catalog: MPS_Database_v1_Core.xlsx
  |-- derived export: mps_database_v1_core.json
  |-- visual contract: pilates_visual_styles.json
  |-- visual manifest: pilates_visual_manifest.json
  |-- card drafts, image assets, audit log
  |
  +-- remote MCP server <---- OAuth connection ---- ChatGPT Custom App
  |
  +-- ImageGenerator adapter ---- image generation provider
~~~

The browser never receives an OpenAI or image-provider secret. The MCP connection authenticates the teacher to MPS; the MPS backend owns provider credentials and job execution.

## Source-of-truth rules

- data/reference/MPS_Database_v1_Core.xlsx is the authoritative exercise catalog.
- data/reference/mps_database_v1_core.json is a derived machine-readable export.
- data/reference/pilates_visual_styles.json contains the single active style profile: mono-gesture-ink-pilates-v1.
- data/reference/pilates_visual_manifest.json identifies the current teacher, private source roles, public assets, and card review state.
- Teaching copy is editable presentation data, but it cannot overwrite exercise ID, apparatus, level, or source relationships.
- The character sheet and visual contract remain needs-review until the teacher approves them.
- A generated image is never publishable until it passes automated checks and human review.

## AI tool contract

The MCP server exposes small, typed tools rather than the raw workbook or filesystem.

### Read tools

- search_exercises(query, apparatus?, level?, body_region?)
- get_exercise_context(exercise_id)
- get_visual_contract()
- get_character_reference()
- get_flashcard(card_id)

get_exercise_context returns the canonical exercise row plus relevant family, category, progression, body-region, movement-taxonomy, and apparatus relationships. It does not return private teacher photos.

### Draft and visual tools

- create_flashcard_draft(exercise_id)
- propose_teaching_copy(card_id, constraints)
- build_visual_brief(card_id, pose_notes?)
- generate_flashcard_image(brief_id)
- review_flashcard_image(asset_id)
- regenerate_flashcard_image(asset_id, revision_notes)
- save_flashcard_draft(card_id, patch)
- submit_for_review(card_id)

Write tools must return a proposed change or draft status. They must not publish, approve the character sheet, mutate workbook data, or create a new exercise.

## Visual brief

The AI must produce a structured brief before image generation:

~~~json
{
  "cardId": "M02",
  "exerciseId": "source-exercise-id",
  "styleProfile": "mono-gesture-ink-pilates-v1",
  "characterId": "teacher-01",
  "outfit": "off-white thin-strap cropped camisole; charcoal high-waisted mid-thigh biker shorts",
  "pose": {
    "startingPosition": "supine, knees flexed, feet grounded",
    "landmarks": ["pelvis", "lumbar spine", "rib cage", "feet"],
    "contactPoints": ["mat", "both feet", "hands near pelvis"],
    "view": "three-quarter side view"
  },
  "apparatus": "Mat",
  "palette": {
    "ink": "#292526",
    "charcoal": "#3B3739",
    "cheekAccent": "#D98F9A"
  },
  "mustShow": ["neutral starting lumbar position", "subtle pink cheeks"],
  "mustNotShow": ["arrows", "text", "logos", "watermark", "extra people"],
  "status": "draft"
}
~~~

The brief is the handoff between canonical data and image generation. It makes the prompt reproducible, reviewable, and independent of a free-form chat transcript.

## Generation and review pipeline

1. Teacher selects an exercise in the PWA.
2. MPS creates a draft with a validated exercise_id.
3. ChatGPT requests the exercise context and visual contract through MCP.
4. ChatGPT proposes teaching copy and a visual brief.
5. Teacher confirms “generate” in ChatGPT or the PWA.
6. MPS queues an image-generation job using the brief plus canonical reference assets.
7. MPS stores the result under a repo-relative or object-store draft path and records a manifest entry.
8. Automated checks validate metadata, asset path, dimensions, and required references.
9. Visual review checks pose, anatomy, identity, glasses/hair, outfit, apparatus, mono linework, pink cheek accent, and absence of arrows/text/logos.
10. The teacher reviews the image in the PWA and either requests a revision or approves the card.
11. Only the PWA's explicit publish action moves a card to published.

## State model

~~~text
draft
  -> generating
  -> needs-review
  -> revision-requested -> generating
  -> approved
  -> published
~~~

Failed generation and failed validation are recorded with a reason and remain outside the published deck.

## Safety and permissions

- read_catalog: read canonical exercise data.
- write_draft: create or update teaching copy and visual briefs.
- generate_asset: queue image generation.
- submit_review: move a draft to needs-review.
- publish: PWA-only, never exposed as an AI tool.

Every write records teacher identity, source card version, brief version, asset version, validation result, and timestamp. Private source photos remain outside git and are referenced only through logical private:// IDs.

## Error handling

- Unknown exercise ID: reject before any AI generation.
- Missing required reference role: block generation and show the missing role.
- Workbook/style version mismatch: block generation and require a refreshed context.
- Image provider failure: keep the card in generating with retryable job status.
- Visual drift: mark revision-requested, preserve the rejected asset, and show concrete findings.
- AI unavailable: the PWA still supports manual teaching-copy editing and review.

## MVP acceptance criteria

- A teacher can search the canonical exercise catalog on iPhone Safari.
- Selecting an exercise creates a single-scroll flashcard draft.
- ChatGPT can retrieve the exercise context and mono visual contract through the MPS Custom App/MCP connection.
- The assistant can create a visual brief that names the exercise, character, outfit, pose landmarks, apparatus, palette, and negative constraints.
- A generation job produces a versioned draft image without exposing provider secrets to the browser.
- The PWA shows validation findings and requires explicit teacher approval before publication.
- No AI action can modify workbook identity or publish a card.
- Existing deterministic class-plan generation remains unchanged.

## Deliberate non-goals

- Replacing the workbook with an AI-authored exercise catalog.
- Automatically generating and publishing an entire deck without review.
- Allowing the AI to approve identity, anatomy, safety, or final artwork.
- Putting ChatGPT or provider credentials in client-side JavaScript.
- Adding multiple visual style profiles before the current mono profile is approved.

## Rollout

1. Build the PWA library/editor/review screens around the existing static data.
2. Add a backend card/job API and persistent draft states.
3. Expose read-only MPS MCP tools and connect the Custom ChatGPT App with OAuth.
4. Add structured brief generation and image-generation jobs.
5. Add visual QA and revision loop.
6. Enable teacher approval and publish.
