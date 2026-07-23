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
  createWorkerPersistenceReporter,
  createWorkerPersistenceReporterFromEnv,
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
      { id: 'outfit-pilates-v1', role: 'outfit', source: 'private://teacher-01/pilates-outfit-v1', required: true },
      { id: 'cadillac-studio', role: 'equipment_context', source: 'private://studio/cadillac-reference', required: true },
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

const referenceAssets = {
  canonicalSheet: {
    id: 'teacher-01-v5',
    version: 5,
    key: 'object://mps-flashcards/character/teacher-01-v5.png',
  },
  references: [
    { id: 'face-front', role: 'face_identity', key: 'object://mps-flashcards/character/face-front.png' },
    { id: 'body-front', role: 'full_body_front', key: 'object://mps-flashcards/character/body-front.png' },
    { id: 'body-side', role: 'full_body_side', key: 'object://mps-flashcards/character/body-side.png' },
    { id: 'body-back', role: 'full_body_back', key: 'object://mps-flashcards/character/body-back.png' },
    { id: 'outfit-pilates-v1', role: 'outfit', key: 'object://mps-flashcards/character/outfit-v1.png' },
    { id: 'cadillac-studio', role: 'equipment_context', key: 'object://mps-flashcards/studio/cadillac.png' },
  ],
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

function testPersistenceReporter(onRequest = () => {}) {
  return createWorkerPersistenceReporter({
    baseUrl: 'https://mps.internal',
    workerToken: 'test-worker-token',
    fetchImpl: async (url, options) => {
      onRequest(String(url), options);
      const payload = JSON.parse(options.body);
      return {
        ok: true,
        status: 200,
        json: async () => ({
          status: String(url).endsWith('/claim') ? 'running' : payload.status,
          claim_id: payload.claim_id ?? 'claim-1',
        }),
      };
    },
  });
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
  assert.equal(isSafeAssetPath('object://mps-flashcards//card-1.png'), false);
  assert.equal(isSafeAssetPath('object://mps-flashcards/card-1.png/'), false);
  assert.equal(isSafeAssetPath('/Users/teacher/card-1.png'), false);
  assert.equal(isSafeAssetPath('web/assets/flashcard-images/../secret.png'), false);
  assert.equal(isSafeAssetPath('https://example.test/card-1.png'), false);
});

test('requires a canonical sheet and complete resolved reference pack', () => {
  assert.throws(
    () => compileGenerationPayload({ brief: brief(), exerciseContext, manifest }),
    (error) => error.errors.some((entry) => entry.code === 'invalid_reference_assets'),
  );
  assert.throws(
    () => compileGenerationPayload({
      brief: brief(),
      exerciseContext,
      manifest,
      referenceAssets: { ...referenceAssets, canonicalSheet: undefined },
    }),
    (error) => error.errors.some((entry) => entry.code === 'canonical_sheet_required'),
  );
});

test('binds resolved reference IDs and canonical sheet identity to the manifest', () => {
  assert.throws(
    () => compileGenerationPayload({
      brief: brief(),
      exerciseContext,
      manifest,
      referenceAssets: {
        ...referenceAssets,
        canonicalSheet: { ...referenceAssets.canonicalSheet, id: 'teacher-01-v4' },
      },
    }),
    (error) => error.errors.some((entry) => entry.code === 'canonical_sheet_identity_mismatch'),
  );
  assert.throws(
    () => compileGenerationPayload({
      brief: brief(),
      exerciseContext,
      manifest,
      referenceAssets: {
        ...referenceAssets,
        references: referenceAssets.references.map((reference) => reference.role === 'outfit'
          ? { ...reference, id: 'safe-but-arbitrary' }
          : reference),
      },
    }),
    (error) => error.errors.some((entry) => entry.code === 'reference_id_mismatch'),
  );
});

test('compiles a deterministic payload with canonical refs, contact points, and extra negatives', () => {
  const payload = compileGenerationPayload({
    brief: brief({ must_not_show_json: [...REQUIRED_NEGATIVE_CONSTRAINTS, 'photorealism', 'gradients'] }),
    exerciseContext,
    manifest,
    referenceAssets,
  });

  assert.equal(payload.styleProfile, STYLE_PROFILE);
  assert.equal(payload.character.id, CHARACTER_ID);
  assert.equal(payload.character.outfit, LOCKED_OUTFIT);
  assert.equal(payload.character.cheekAccent, CHEEK_ACCENT);
  assert.equal(payload.character.canonicalSheet.path, referenceAssets.canonicalSheet.key);
  assert.deepEqual(payload.pose.contactPoints, ['sacrum', 'feet']);
  assert.deepEqual(payload.negativeConstraints, [...REQUIRED_NEGATIVE_CONSTRAINTS, 'photorealism', 'gradients']);
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
  assert.equal(JSON.stringify(result).includes('glasses are missing'), false);
});

test('worker emits status events and leaves a passing asset in needs-review', async () => {
  const events = [];
  const persistenceRequests = [];
  const generator = createImageGenerator({
    generate: async ({ brief: compiledBrief, referenceAssets }) => {
      assert.equal(compiledBrief.styleProfile, STYLE_PROFILE);
      assert.equal(referenceAssets.canonicalSheet.path, referenceAssetsForGenerator.canonicalSheet.path);
      assert.equal(referenceAssets.references[0].role, 'face_identity');
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
  const referenceAssetsForGenerator = {
    canonicalSheet: {
      id: 'teacher-01-v5',
      version: 5,
      path: referenceAssets.canonicalSheet.key,
    },
    references: referenceAssets.references.map((reference) => ({
      id: reference.id,
      role: reference.role,
      path: reference.key,
    })),
  };
  const result = await processVisualJob({
    job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
    brief: brief(),
    exerciseContext,
    manifest,
    referenceAssets,
    generator,
    visionReviewer: createVisionReviewer({
      review: async () => ({ passed: true, checks: passingChecks(), findings: [] }),
    }),
    report: async (event) => events.push(event),
    persistenceReporter: testPersistenceReporter((url, options) => {
      persistenceRequests.push({ url, body: JSON.parse(options.body) });
    }),
  });

  assert.equal(result.status, 'succeeded');
  assert.equal(result.cardStatus, 'needs-review');
  assert.equal(result.asset.status, 'needs-review');
  assert.equal(result.claimId, persistenceRequests[0].body.claim_id);
  assert.deepEqual(events.map((event) => event.status), ['running', 'succeeded']);
  assert.equal(persistenceRequests[0].url.endsWith('/claim'), true);
  assert.match(persistenceRequests[0].body.claim_id, /^attempt-/);
  assert.deepEqual(persistenceRequests.slice(1).map((request) => request.body.status), ['succeeded']);
  assert.equal(persistenceRequests[1].body.claim_id, persistenceRequests[0].body.claim_id);
});

test('worker rejects production execution without an atomic persistence reporter', async () => {
  await assert.rejects(
    () => processVisualJob({
      job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
      brief: brief(),
      exerciseContext,
      manifest,
      referenceAssets,
    }),
    /secure worker persistence reporter is required/,
  );
  await assert.rejects(
    () => processVisualJob({
      job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
      brief: brief(),
      exerciseContext,
      manifest,
      referenceAssets,
      persistenceReporter: async () => {},
    }),
    /secure worker persistence reporter is required/,
  );
});

test('unsafe provider paths fail closed without echoing the raw path', async () => {
  const events = [];
  const result = await processVisualJob({
    job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
    brief: brief(),
    exerciseContext,
    manifest,
    referenceAssets,
    generator: createImageGenerator({
      generate: async () => ({
        asset: {
          id: 'asset-1',
          cardId: 'attacker-card',
          briefId: 'attacker-brief',
          path: '/Users/secret/provider-output.png',
          mimeType: 'image/png',
        },
      }),
    }),
    visionReviewer: createVisionReviewer({ review: async () => ({ passed: true, checks: passingChecks() }) }),
    report: async (event) => events.push(event),
    persistenceReporter: testPersistenceReporter(),
  });

  const output = JSON.stringify({ result, events });
  assert.equal(result.status, 'failed');
  assert.equal(output.includes('/Users/secret/provider-output.png'), false);
  assert.equal(output.includes('provider-output'), false);
});

test('provider and vision exceptions are reduced to generic worker errors', async () => {
  const providerFailure = await processVisualJob({
    job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
    brief: brief(),
    exerciseContext,
    manifest,
    referenceAssets,
    generator: createImageGenerator({
      generate: async () => { throw new Error('provider secret response body'); },
    }),
    persistenceReporter: testPersistenceReporter(),
  });
  assert.equal(providerFailure.error.message, 'image generation or visual review failed');
  assert.equal(JSON.stringify(providerFailure).includes('provider secret'), false);

  const visionFailure = await processVisualJob({
    job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
    brief: brief(),
    exerciseContext,
    manifest,
    referenceAssets,
    generator: createImageGenerator({
      generate: async () => ({
        asset: { id: 'asset-1', path: 'object://mps-flashcards/card-1.png', mimeType: 'image/png' },
      }),
    }),
    visionReviewer: createVisionReviewer({
      review: async () => { throw new Error('vision provider private detail'); },
    }),
    persistenceReporter: testPersistenceReporter(),
  });
  assert.equal(visionFailure.error.message, 'image generation or visual review failed');
  assert.equal(JSON.stringify(visionFailure).includes('private detail'), false);
});

test('callback persistence failures propagate as retryable errors', async () => {
  const failingReporter = createWorkerPersistenceReporter({
    baseUrl: 'https://mps.internal',
    workerToken: 'worker-secret',
    fetchImpl: async () => { throw new Error('private transport detail'); },
  });
  await assert.rejects(
    () => processVisualJob({
      job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
      brief: brief(),
      exerciseContext,
      manifest,
      referenceAssets,
      generator: createImageGenerator({
        generate: async () => { throw new Error('provider detail'); },
      }),
      persistenceReporter: failingReporter,
    }),
    (error) => error.code === 'visual_worker_persistence_retryable' && error.retryable === true,
  );
});

test('success callback plus fallback failure propagates as retryable persistence', async () => {
  const reporter = async () => { throw new Error('private transport detail'); };
  Object.defineProperty(reporter, 'durable', { value: true });
  Object.defineProperty(reporter, 'claim', { value: async ({ claimId }) => ({ status: 'running', claim_id: claimId }) });
  await assert.rejects(
    () => processVisualJob({
      job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
      brief: brief(),
      exerciseContext,
      manifest,
      referenceAssets,
      generator: createImageGenerator({
        generate: async () => ({
          asset: {
            id: 'asset-1',
            path: 'object://mps-flashcards/card-1.png',
            mimeType: 'image/png',
          },
        }),
      }),
      visionReviewer: createVisionReviewer({
        review: async () => ({ passed: true, checks: passingChecks(), findings: [] }),
      }),
      persistenceReporter: reporter,
    }),
    (error) => error.code === 'visual_worker_persistence_retryable' && error.retryable === true,
  );
});

test('lost success response retries the original terminal event instead of failing the job', async () => {
  const terminalEvents = [];
  const reporter = async (event) => {
    terminalEvents.push(event);
    if (terminalEvents.length === 1) throw new Error('response lost after commit');
  };
  Object.defineProperty(reporter, 'durable', { value: true });
  Object.defineProperty(reporter, 'claim', {
    value: async ({ claimId }) => ({ status: 'running', claim_id: claimId }),
  });
  const result = await processVisualJob({
    job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
    brief: brief(),
    exerciseContext,
    manifest,
    referenceAssets,
    generator: createImageGenerator({
      generate: async () => ({
        asset: {
          id: 'asset-1',
          path: 'object://mps-flashcards/card-1.png',
          mimeType: 'image/png',
        },
      }),
    }),
    visionReviewer: createVisionReviewer({
      review: async () => ({ passed: true, checks: passingChecks(), findings: [] }),
    }),
    persistenceReporter: reporter,
  });

  assert.equal(result.status, 'succeeded');
  assert.deepEqual(terminalEvents.map((event) => event.status), ['succeeded', 'succeeded']);
  assert.match(terminalEvents[0].claimId, /^attempt-/);
  assert.deepEqual(terminalEvents.map((event) => event.claimId), [terminalEvents[0].claimId, terminalEvents[0].claimId]);
});

test('lost claim response can retry with the same persisted claim ID', async () => {
  const claimAttempts = [];
  let persistedClaimId;
  const reporter = createWorkerPersistenceReporter({
    baseUrl: 'https://mps.internal',
    workerToken: 'worker-secret',
    fetchImpl: async (url, options) => {
      const payload = JSON.parse(options.body);
      if (String(url).endsWith('/claim')) {
        claimAttempts.push(payload.claim_id);
        persistedClaimId ??= payload.claim_id;
        if (claimAttempts.length === 1) throw new Error('response lost after commit');
        return {
          ok: true,
          status: 200,
          json: async () => ({ status: 'running', claim_id: persistedClaimId }),
        };
      }
      return {
        ok: true,
        status: 200,
        json: async () => ({ status: payload.status }),
      };
    },
  });
  const job = { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' };
  const generation = {
    generator: createImageGenerator({
      generate: async () => ({
        asset: {
          id: 'asset-1',
          path: 'object://mps-flashcards/card-1.png',
          mimeType: 'image/png',
        },
      }),
    }),
    visionReviewer: createVisionReviewer({
      review: async () => ({ passed: true, checks: passingChecks(), findings: [] }),
    }),
    persistenceReporter: reporter,
  };
  let retryClaimId;
  await assert.rejects(
    () => processVisualJob({
      job,
      brief: brief(),
      exerciseContext,
      manifest,
      referenceAssets,
      ...generation,
    }),
    (error) => {
      retryClaimId = error.claimId;
      return error.code === 'visual_worker_persistence_retryable'
        && error.retryable === true
        && /^attempt-/.test(error.claimId)
        && error.claimId === persistedClaimId;
    },
  );

  const result = await processVisualJob({
    job: { ...job, claimId: retryClaimId },
    brief: brief(),
    exerciseContext,
    manifest,
    referenceAssets,
    ...generation,
  });

  assert.equal(result.status, 'succeeded');
  assert.equal(result.claimId, retryClaimId);
  assert.deepEqual(claimAttempts, [retryClaimId, retryClaimId]);
});

test('a full retry reconciles an already committed terminal claim before generation', async () => {
  let generated = false;
  const reporter = async () => { throw new Error('terminal callback should not run'); };
  Object.defineProperty(reporter, 'durable', { value: true });
  Object.defineProperty(reporter, 'claim', {
    value: async () => ({ status: 'succeeded', claim_id: 'claim-1' }),
  });
  const result = await processVisualJob({
    job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
    brief: brief(),
    exerciseContext,
    manifest,
    referenceAssets,
    generator: createImageGenerator({
      generate: async () => {
        generated = true;
        return { asset: { id: 'asset-1', path: 'object://mps-flashcards/card-1.png', mimeType: 'image/png' } };
      },
    }),
    persistenceReporter: reporter,
  });

  assert.equal(result.status, 'succeeded');
  assert.equal(result.reconciled, true);
  assert.equal(generated, false);
});

test('persistence reporter receives trusted ownership and terminal-only data', async () => {
  let persisted;
  const result = await processVisualJob({
    job: { id: 'job-1', cardId: 'card-1', briefId: 'brief-1' },
    brief: brief(),
    exerciseContext,
    manifest,
    referenceAssets,
    generator: createImageGenerator({
      generate: async () => ({
        asset: {
          id: 'asset-1',
          cardId: 'attacker-card',
          briefId: 'attacker-brief',
          path: 'object://mps-flashcards/card-1.png',
          mimeType: 'image/png',
        },
      }),
    }),
    visionReviewer: createVisionReviewer({
      review: async () => ({ passed: true, checks: passingChecks(), findings: [] }),
    }),
    persistenceReporter: testPersistenceReporter((_url, options) => {
      persisted = JSON.parse(options.body);
    }),
  });

  assert.equal(result.status, 'succeeded');
  assert.equal(persisted.status, 'succeeded');
  assert.equal(persisted.asset.brief_id, 'brief-1');
  assert.equal('cardId' in persisted, false);
  assert.equal('generationPayload' in persisted, false);
});

test('HTTP persistence reporter sends only the server callback contract', async () => {
  let request;
  const reporter = createWorkerPersistenceReporter({
    baseUrl: 'https://mps.internal',
    workerToken: 'worker-secret',
    fetchImpl: async (url, options) => {
      request = { url: String(url), options };
      return { ok: true, status: 200, json: async () => ({ status: 'failed' }) };
    },
  });
  await reporter({ jobId: 'job-1', cardId: 'card-1', claimId: 'claim-1', status: 'failed', error: { message: 'hidden' } });

  assert.equal(request.url, 'https://mps.internal/api/internal/flashcards/card-1/jobs/job-1/complete');
  assert.equal(request.options.headers.authorization, 'Bearer worker-secret');
  assert.deepEqual(JSON.parse(request.options.body), { status: 'failed', claim_id: 'claim-1' });
});

test('HTTP claim reporter sends the worker-created attempt ID', async () => {
  let request;
  const reporter = createWorkerPersistenceReporter({
    baseUrl: 'https://mps.internal',
    workerToken: 'worker-secret',
    fetchImpl: async (url, options) => {
      request = { url: String(url), options };
      return { ok: true, status: 200, json: async () => ({ status: 'running', claim_id: 'attempt-1' }) };
    },
  });
  await reporter.claim({ jobId: 'job-1', cardId: 'card-1', claimId: 'attempt-1' });

  assert.equal(request.url, 'https://mps.internal/api/internal/flashcards/card-1/jobs/job-1/claim');
  assert.deepEqual(JSON.parse(request.options.body), { claim_id: 'attempt-1' });
});

test('HTTP persistence reporter verifies the terminal response status', async () => {
  const reporter = createWorkerPersistenceReporter({
    baseUrl: 'https://mps.internal',
    workerToken: 'worker-secret',
    fetchImpl: async () => ({
      ok: true,
      status: 200,
      json: async () => ({ status: 'succeeded' }),
    }),
  });
  await assert.rejects(
    () => reporter({ jobId: 'job-1', cardId: 'card-1', claimId: 'claim-1', status: 'failed' }),
    (error) => error.code === 'visual_worker_persistence_retryable',
  );
});

test('HTTP persistence reporter rejects a non-200 callback response', async () => {
  const reporter = createWorkerPersistenceReporter({
    baseUrl: 'https://mps.internal',
    workerToken: 'worker-secret',
    fetchImpl: async () => ({
      ok: true,
      status: 202,
      json: async () => ({ status: 'failed' }),
    }),
  });
  await assert.rejects(
    () => reporter({ jobId: 'job-1', cardId: 'card-1', claimId: 'claim-1', status: 'failed' }),
    (error) => error.code === 'visual_worker_persistence_retryable',
  );
});

test('persistence findings keep allowlisted QA check names without provider messages', async () => {
  let payload;
  const reporter = createWorkerPersistenceReporter({
    baseUrl: 'https://mps.internal',
    workerToken: 'worker-secret',
    fetchImpl: async (_url, options) => {
      payload = JSON.parse(options.body);
      return { ok: true, status: 200, json: async () => ({ status: 'succeeded' }) };
    },
  });
  await reporter({
    jobId: 'job-1',
    cardId: 'card-1',
    claimId: 'claim-1',
    status: 'succeeded',
    asset: {
      id: 'asset-1',
      cardId: 'card-1',
      briefId: 'brief-1',
      path: 'object://mps-flashcards/card-1.png',
      providerJobId: null,
      version: 1,
    },
    review: {
      passed: false,
      findings: [
        { code: 'visual_check_failed', check: 'identity', severity: 'error', message: 'secret provider text' },
        { code: 'visual_check_failed', check: 'not-allowed', severity: 'error', message: 'other secret' },
      ],
    },
  });
  assert.deepEqual(payload.review.findings_json, [
    { code: 'visual_check_failed', severity: 'error', check: 'identity' },
    { code: 'visual_finding', severity: 'error' },
  ]);
  assert.equal(JSON.stringify(payload).includes('secret provider text'), false);
});

test('worker persistence reporter reads its dedicated server-only environment contract', async () => {
  let request;
  const reporter = createWorkerPersistenceReporterFromEnv({
    env: {
      MPS_VISUAL_WORKER_API_BASE_URL: 'https://mps.internal',
      MPS_VISUAL_WORKER_TOKEN: 'worker-secret',
    },
    fetchImpl: async (url, options) => {
      request = { url: String(url), options };
      return { ok: true, status: 200, json: async () => ({ status: 'failed' }) };
    },
  });
  await reporter({ jobId: 'job-1', cardId: 'card-1', claimId: 'claim-1', status: 'failed' });
  assert.equal(request.options.headers.authorization, 'Bearer worker-secret');
});
