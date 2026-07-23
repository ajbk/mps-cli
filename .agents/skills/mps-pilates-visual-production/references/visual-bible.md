# MPS Pilates visual bible

This is the visual contract for teacher-based Pilates flashcards.

## Identity lock

- Treat the approved canonical character sheet as the identity authority.
- Use the teacher's face, glasses, hair silhouette, and body proportions from the reference pack.
- The locked teacher character has a shoulder-length layered bob with a natural center part, dark brown-black hair, round dark glasses, and the exact outfit: off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts. No alternate clothing or outfit variation is allowed.
- Add a subtle, natural dusty-rose pink tint to both cheeks; it should be soft, symmetrical, and remain an accent within the otherwise monochrome ink language.
- Character-sheet v5 measurement baseline: 167 cm, 62 kg, bust 34 in, waist 29 in, hips 37 in; layout is a 2x3 identity grid.
- Preserve the real teacher proportions from the full-body reference pack exactly. Do not make the character thinner, taller, younger, more muscular, more sexualized, or otherwise stylized than the references.
- Do not infer hidden anatomy from a cropped portrait. Use full-body front, side, and back references for proportions.

## Generation lock

Every image request must identify:

1. The canonical character sheet.
2. The face/identity reference.
3. At least one full-body reference.
4. The outfit reference showing the exact off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts.
5. The equipment or studio reference when apparatus is visible.
6. The exact exercise and the body landmarks that must be readable.
7. The subtle dusty-rose cheek accent.

Generate one card at a time while the character sheet is under review. Batch generation starts only after `character.status` is `approved`.

## Review gate

Reject and regenerate when any of these drift:

- face identity or glasses;
- bob length, parting, or hair silhouette;
- shoulder, waist, hip, leg, or overall body proportions;
- exact off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts shape or coverage;
- cheek accent is missing, heavy, asymmetrical, or reads as strong makeup;
- apparatus geometry, hand/foot contact, or exercise setup;
- anatomy, joints, fingers, or feet.

Visual similarity is not fully deterministic. The manifest validator checks the process and metadata; a human must still approve the canonical sheet and each final image.

## Privacy and repository rules

- Never commit teacher photos, absolute paths, generated-image cache paths, or private metadata.
- Use `private://...` logical IDs in committed manifests.
- Store approved public card art only under the project's normal web/docs asset paths.
- Keep rejected generations out of the public deck and record the reason in a local review note, not in the public manifest unless it is useful for reproducibility.
