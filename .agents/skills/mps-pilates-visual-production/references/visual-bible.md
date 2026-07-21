# MPS Pilates visual bible

This is the visual contract for teacher-based Pilates flashcards.

## Identity lock

- Treat the approved canonical character sheet as the identity authority.
- Use the teacher's face, glasses, hair silhouette, and body proportions from the reference pack.
- The current teacher variant is a shoulder-length layered bob, dark brown-black hair, round dark frames, and white Pilates clothing.
- Do not make the character thinner, taller, younger, more muscular, or more sexualized than the references.
- Do not infer hidden anatomy from a cropped portrait. Use full-body front, side, and back references for proportions.

## Generation lock

Every image request must identify:

1. The canonical character sheet.
2. The face/identity reference.
3. At least one full-body reference.
4. The outfit reference.
5. The equipment or studio reference when apparatus is visible.
6. The exact exercise and the body landmarks that must be readable.

Generate one card at a time while the character sheet is under review. Batch generation starts only after `character.status` is `approved`.

## Review gate

Reject and regenerate when any of these drift:

- face identity or glasses;
- bob length, parting, or hair silhouette;
- shoulder, waist, hip, leg, or overall body proportions;
- white outfit shape or coverage;
- apparatus geometry, hand/foot contact, or exercise setup;
- anatomy, joints, fingers, or feet.

Visual similarity is not fully deterministic. The manifest validator checks the process and metadata; a human must still approve the canonical sheet and each final image.

## Privacy and repository rules

- Never commit teacher photos, absolute paths, generated-image cache paths, or private metadata.
- Use `private://...` logical IDs in committed manifests.
- Store approved public card art only under the project's normal web/docs asset paths.
- Keep rejected generations out of the public deck and record the reason in a local review note, not in the public manifest unless it is useful for reproducibility.
