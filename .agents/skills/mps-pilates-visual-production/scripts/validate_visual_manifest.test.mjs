import assert from 'node:assert/strict';
import test from 'node:test';

import { validateManifest } from './validate_visual_manifest.mjs';

const validManifest = {
  schemaVersion: 1,
  character: {
    id: 'teacher-01',
    status: 'approved',
    canonicalSheet: 'visual/character/teacher-01-v1.png',
    references: [
      { id: 'face-front', role: 'face_identity', source: 'private://teacher-01/face-front.jpg', required: true },
      { id: 'body-front', role: 'full_body_front', source: 'private://teacher-01/body-front.jpg', required: true },
      { id: 'body-side', role: 'full_body_side', source: 'private://teacher-01/body-side.jpg', required: true },
      { id: 'body-back', role: 'full_body_back', source: 'private://teacher-01/body-back.jpg', required: true },
      { id: 'outfit', role: 'outfit', source: 'private://teacher-01/outfit.jpg', required: true },
      { id: 'equipment', role: 'equipment_context', source: 'private://studio/equipment.jpg', required: true },
    ],
  },
  cards: [
    {
      id: 'mat-01',
      status: 'draft',
      image: 'assets/flashcard-images/mat/m01.png',
      referenceIds: ['face-front', 'body-front', 'outfit', 'equipment'],
    },
  ],
};

test('accepts a manifest with the complete reference pack and relative asset paths', () => {
  assert.deepEqual(validateManifest(validManifest), []);
});

test('reports missing required reference roles', () => {
  const manifest = structuredClone(validManifest);
  manifest.character.references = manifest.character.references.filter((reference) => reference.role !== 'full_body_back');

  const errors = validateManifest(manifest);

  assert.ok(errors.some((error) => error.includes('full_body_back')));
});

test('rejects machine-local paths and approved cards before character approval', () => {
  const manifest = structuredClone(validManifest);
  manifest.character.status = 'needs-review';
  manifest.character.canonicalSheet = '/Users/teacher/character.png';
  manifest.cards[0].status = 'approved';
  manifest.cards[0].image = '/tmp/card.png';

  const errors = validateManifest(manifest);

  assert.ok(errors.some((error) => error.includes('machine-local')));
  assert.ok(errors.some((error) => error.includes('approved character')));
});
