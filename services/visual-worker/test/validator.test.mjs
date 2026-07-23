import assert from 'node:assert/strict';
import test from 'node:test';

import {
  CHARACTER_ID,
  CHEEK_ACCENT,
  LOCKED_OUTFIT,
  REQUIRED_NEGATIVE_CONSTRAINTS,
  STYLE_PROFILE,
  assertValidVisualBrief,
  compileGenerationPayload,
  isSafeAssetPath,
} from '../src/visual-contract.mjs';
import {
  REQUIRED_VISUAL_CHECKS,
  reviewGeneratedAsset,
} from '../src/validator.mjs';
import {
  createImageGenerator,
  createVisionReviewer,
  processVisualJob,
} from '../src/worker.mjs';

const manifest = {
  primaryStyleProfile: STYLE_PROFILE,
  character: {
    id: CHARACTER_ID,
    status: 'needs-review',
    version: 5,
    canonicalSheet: 'data/reference/visual/character/teacher-01-v5-grid-outfit-draft.png',
    fixedAttributes: {
      top: 'fitted off-white cropped Pilates camisole with thin straps',
      bottom: 'dark charcoal high-waisted fitted Pilates biker shorts ending mid-thigh',
    },
    references: [
      { id: 'face-front', role: 'face_identity', source: 'private://teacher-01/face-front', required: true },
      { id: 'body-front', role: 'full_body_front', source: 'private://teacher-01/full-body-front', required: true },
      { id: 'body-side', role: 'full_body_side', source: 'private://teacher-01/full-body-side', required: true },
      { id: 'body-back', role: 'full_body_back', source: 'private://teacher-01/full-body-back', required: true },
      { id: 'outfit', role: 'outfit', source: 'private://teacher-01/outfit', required: true },
      { id: 'equipment', role: 'equipment_context', source: 'private://studio/equipment', required: true },
    ],
  },
};

const exerciseContext = {
  exercise: {
    id: 'source_mat_pelvic_clock_row_1',
    name: 'Pelvic Clock',
    apparatus: 'Mat',
    equipmentKey: 'mat',
  },
};

function brief(overrides = {}) {
  return {
    id: 'brief-1',
    card_id: 'card-1',
    exercise_id: exerciseContext.exercise.id,
    style_profile: STYLE_PROFILE,
    character_id: CHARACTER_ID,
    outfit: LOCKED_OUTFIT,
    pose_json: {
      landmarks: [{ name: 'sacrum', x: 0.5, y: 0.5 }],
      contact_points: ['sacrum', 'feet'],
      position: 'supine',
    },
    apparatus: 'Mat',
    palette_json: { cheekAccent: CHEEK_ACCENT },
    must_show_json: ['neutral lumbar position', 'breathing'],
    must_not_show_json: [...REQUIRED_NEGATIVE_CONSTRAINTS],
    version: 1,
    ...overrides,
  };
}

function passingChecks() {
  return Object.fromEntries(REQUIRED_VISUAL_CHECKS.map((name) => [name, { passed: true }]));
}

test('rejects a brief that changes the visual style profile', () => {
  assert.throws(
    () => assertValidVisualBrief({
      brief: brief({ style_profile: 'other-style' }),
      exerciseContext,
      manifest,
    }),
    (error) => error.errors.some((entry) => entry.code === 'style_profile_locked'),
  );
});

test('rejects a brief that changes the teacher identity or outfit', () => {
  for (const overrides of [
    { character_id: 'other-teacher' },
    { outfit: 'white pants and a black top' },
  ]) {
    assert.throws(
      () => assertValidVisualBrief({ brief: brief(overrides), exerciseContext, manifest }),
      (error) => error.errors.some((entry) => ['character_locked', 'outfit_locked'].includes(entry.code)),
    );
  }
});

test('requires the cheek accent and all forbidden overlay constraints', () => {
  assert.throws(
    () => assertValidVisualBrief({
      brief: brief({
        palette_json: { cheekAccent: '#000000' },
        must_not_show_json: ['arrows', 'text'],
      }),
      exerciseContext,
      manifest,
    }),
    (error) => error.errors.some((entry) => entry.code === 'cheek_accent_locked')
      && error.errors.filter((entry) => entry.code === 'negative_constraint_missing').length === 2,
  );
});

test('accepts only approved repo-relative or object-store asset paths', () => {
  assert.equal(isSafeAssetPath('web/assets/flashcard-images/mat/card-1.png'), true);
  assert.equal(isSafeAssetPath('object://mps-flashcards/card-1.png'), true);
  assert.equal(isSafeAssetPath('/Users/teacher/card-1.png'), false);
  assert.equal(isSafeAssetPath('web/assets/flashcard-images/../secret.png'), false);
  assert.equal(isSafeAssetPath('https://example.test/card-1.png'), false);
});

test('compiles a deterministic payload with contact points, identity, and negative constraints', () => {
  const payload = compileGenerationPayload({ brief: brief(), exerciseContext, manifest });

  assert.equal(payload.styleProfile, STYLE_PROFILE);
  assert.equal(payload.character.id, CHARACTER_ID);
  assert.equal(payload.character.outfit, LOCKED_OUTFIT);
  assert.equal(payload.character.cheekAccent, CHEEK_ACCENT);
  assert.deepEqual(payload.pose.contactPoints, ['sacrum', 'feet']);
  assert.deepEqual(payload.negativeConstraints, [...REQUIRED_NEGATIVE_CONSTRAINTS]);
  assert.equal(payload.output.noWatermarkLayer, true);
});

test('metadata failure skips the vision reviewer and requests revision', async () => {
  let called = false;
  const result = await reviewGeneratedAsset({
    asset: {
      id: 'asset-1',
      path: '/tmp/unsafe.png',
      mimeType: 'image/png',
    },
    brief: brief(),
    generationPayload: {},
    visionReviewer: { review: async () => { called = true; return { passed: true, checks: passingChecks() }; } },
  });

  assert.equal(called, false);
  assert.equal(result.passed, false);
  assert.equal(result.status, 'revision-requested');
  assert.equal(result.assetStatus, 'rejected');
});

test('failed visual checks become revision-requested and never approval', async () => {
  const result = await reviewGeneratedAsset({
    asset: {
      id: 'asset-1',
      path: 'web/assets/flashcard-images/mat/card-1.png',
      mimeType: 'image/png',
    },
    brief: brief(),
    generationPayload: {},
    visionReviewer: createVisionReviewer({
      review: async () => ({
        passed: false,
        checks: { ...passingChecks(), identity: { passed: false, message: 'glasses are missing' } },
        findings: [{ code: 'identity_mismatch', severity: 'error' }],
      }),
    }),
  });

  assert.equal(result.status, 'revision-requested');
  assert.equal(result.assetStatus, 'rejected');
  assert.notEqual(result.status, 'approved');
  assert.notEqual(result.status, 'published');
});

test('worker emits status events and leaves a passing asset in needs-review', async () => {
  const events = [];
  const generator = createImageGenerator({
    generate: async ({ brief: compiledBrief, referenceAssets }) => {
      assert.equal(compiledBrief.styleProfile, STYLE_PROFILE);
      assert.equal(referenceAssets[0].role, 'face_identity');
      return {
        asset: {
          id: 'asset-1',
          path: 'object://mps-flashcards/card-1-v1.png',
          mimeType: 'image/png',
          version: 1,
        },
      };
    },
  });
  const result = await processVisualJob({
    job: { id: 'job-1' },
    brief: brief(),
    exerciseContext,
    manifest,
    generator,
    visionReviewer: createVisionReviewer({
      review: async () => ({ passed: true, checks: passingChecks(), findings: [] }),
    }),
    report: async (event) => events.push(event),
  });

  assert.equal(result.status, 'succeeded');
  assert.equal(result.cardStatus, 'needs-review');
  assert.equal(result.asset.status, 'needs-review');
  assert.deepEqual(events.map((event) => event.status), ['running', 'succeeded']);
});
