---
name: mps-pilates-visual-production
description: Use when creating or reviewing MPS Pilates flashcard artwork that must keep a teacher's face, hair, body proportions, outfit, apparatus, and studio context consistent across images.
---

# MPS Pilates visual production

Use this skill for teacher-based Pilates character sheets, exercise flashcards, reference packs, and visual asset reviews. It turns a personal reference set into a controlled, reviewable asset workflow; it does not promise pixel-perfect identity from an image model.

## Required workflow

1. Locate the project manifest at `data/reference/pilates_visual_manifest.json` and read the visual contract in `references/visual-bible.md`.
2. Keep source photos outside git. Give each source a stable logical `private://...` ID and record its role in the manifest. Required roles are `face_identity`, `full_body_front`, `full_body_side`, `full_body_back`, `outfit`, and `equipment_context`.
3. Create or update one canonical character sheet before generating a deck. The sheet must show the shoulder-length layered bob, round dark glasses, the exact locked outfit of an off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts, the subtle dusty-rose cheek accent, and front/side/back proportions. Preserve the teacher's real proportions with no clothing or body variation. Set `character.status` to `needs-review` until the teacher approves it.
4. Run the validator before any generation:

   ```bash
   node .agents/skills/mps-pilates-visual-production/scripts/validate_visual_manifest.mjs
   ```

5. For each new card, use the canonical sheet plus the relevant reference IDs as image references. Describe the exercise landmarks and apparatus setup explicitly. Do not rely on text-only descriptions of the person's body.
6. Keep new cards at `draft` or `needs-review` until the image passes a human review for identity, proportions, the exact off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts, anatomy, apparatus, exercise setup, and the subtle dusty-rose cheek accent. Only set `character.status` to `approved` after the canonical sheet is accepted; only then may cards become `approved`.
7. Record the generated image as a repo-relative public path in the manifest. Never record `/Users/...`, `/home/...`, `~`, cache paths, or source-photo paths.
8. Before commit, run the validator, the manifest test, `git diff --check`, and the project's existing JavaScript checks. Stage only the skill, manifest, documentation, and approved public assets intended for this change.

## Image-generation rules

- Use every reference image for its declared role; do not ask the model to guess proportions from a face crop.
- Prefer one exercise image per request while the canonical sheet is being calibrated.
- Keep the teacher's real body proportions. Do not make her thinner, taller, younger, or more muscular for visual polish.
- Preserve the shoulder-length layered bob, round dark glasses, and the exact locked outfit: off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts. Add the subtle dusty-rose cheek accent. No alternate clothing, outfit variation, body-proportion variation, slimming, lengthening, or stylization is allowed.
- Use neutral, professional, anatomy-readable Pilates compositions with no text, logos, watermark, or extra people.
- If the result drifts, reject it and regenerate from the canonical sheet and the missing role reference; do not silently accept a close-but-different body.

## Resource commands

The validator is intentionally dependency-free and exports `validateManifest` for Node tests. The repository's sample manifest is a process template, not proof that the teacher has approved the canonical sheet.

```bash
node --test .agents/skills/mps-pilates-visual-production/scripts/validate_visual_manifest.test.mjs
node .agents/skills/mps-pilates-visual-production/scripts/validate_visual_manifest.mjs data/reference/pilates_visual_manifest.json
```
