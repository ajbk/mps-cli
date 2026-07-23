import { randomUUID } from 'node:crypto';

import {
  assertSafeAssetPath,
  assertSafeIdentifier,
  compileGenerationPayload,
  REQUIRED_VISUAL_CHECKS,
  validateReferenceAssets,
  VisualContractError,
} from './visual-contract.mjs';
import { reviewGeneratedAsset } from './validator.mjs';

export class RetryablePersistenceError extends Error {
  constructor(claimId = undefined) {
    super('visual worker persistence is unavailable; retry the job');
    this.name = 'RetryablePersistenceError';
    this.code = 'visual_worker_persistence_retryable';
    this.retryable = true;
    if (claimId !== undefined) this.claimId = claimId;
  }
}

function createAttemptClaimId() {
  return assertSafeIdentifier(`attempt-${randomUUID()}`, 'claimId');
}

function publicError(error) {
  if (error instanceof VisualContractError) {
    return {
      code: 'visual_contract_invalid',
      message: 'visual contract validation failed',
      details: error.errors.map((entry) => ({
        code: entry.code ?? 'contract_invalid',
        field: entry.field,
      })),
    };
  }
  return {
    code: 'visual_worker_failed',
    message: 'image generation or visual review failed',
  };
}

function assertFunction(value, name) {
  if (typeof value !== 'function') throw new TypeError(`${name} must be a function`);
}

/**
 * Provider boundary. The actual provider adapter owns its credentials in a
 * server-side closure; this wrapper never reads browser input or emits them.
 */
export function createImageGenerator({ generate }) {
  assertFunction(generate, 'generate');
  return {
    async generateDraft({ brief, referenceAssets }) {
      return generate({ brief, referenceAssets });
    },
  };
}

/**
 * Vision boundary. Tests and a production vision adapter can implement the
 * same interface without making the validator depend on a network provider.
 */
export function createVisionReviewer({ review }) {
  assertFunction(review, 'review');
  return {
    async review(input) {
      return review(input);
    },
  };
}

export function createStatusReporter({ report }) {
  if (report === undefined) return async () => {};
  assertFunction(report, 'report');
  return async (event) => report(Object.freeze({ ...event }));
}

function sanitizedFinding(finding) {
  const result = {
    code: finding?.code === 'visual_check_failed' ? 'visual_check_failed' : 'visual_finding',
    severity: finding?.severity === 'warning' ? 'warning' : 'error',
  };
  if (result.code === 'visual_check_failed' && REQUIRED_VISUAL_CHECKS.includes(finding?.check)) {
    result.check = finding.check;
  } else if (result.code === 'visual_check_failed') {
    result.code = 'visual_finding';
  }
  return result;
}

async function postWorkerCallback({ fetchImpl, root, workerToken, path, payload, expectedStatus }) {
  let response;
  try {
    response = await fetchImpl(new URL(path, root), {
      method: 'POST',
      headers: {
        authorization: `Bearer ${workerToken}`,
        'content-type': 'application/json',
      },
      body: JSON.stringify(payload),
    });
  } catch {
    throw new RetryablePersistenceError();
  }
  let body;
  try {
    if (!response || typeof response.json !== 'function') throw new Error('missing response body');
    body = await response.json();
  } catch {
    throw new RetryablePersistenceError();
  }
  if (!response.ok || response.status !== 200) {
    throw new RetryablePersistenceError();
  }
  const expectedStatuses = Array.isArray(expectedStatus) ? expectedStatus : [expectedStatus];
  if (!expectedStatuses.includes(body?.status)) {
    throw new RetryablePersistenceError();
  }
  return body;
}

export function createWorkerPersistenceReporter({ baseUrl, workerToken, fetchImpl = fetch }) {
  if (typeof baseUrl !== 'string' || baseUrl.trim() === '') throw new Error('worker persistence base URL is required');
  if (typeof workerToken !== 'string' || workerToken.trim() === '') throw new Error('worker persistence token is required');
  assertFunction(fetchImpl, 'fetchImpl');
  const root = new URL(baseUrl);
  const reporter = async (event) => {
    if (!['succeeded', 'failed'].includes(event?.status)) {
      throw new Error('worker persistence accepts terminal events only');
    }
    const cardId = assertSafeIdentifier(event.cardId, 'cardId');
    const jobId = assertSafeIdentifier(event.jobId, 'jobId');
    const payload = { status: event.status };
    if (event.status === 'succeeded') {
      if (!event.asset || !event.review) throw new Error('successful worker events require asset and review');
      const assetId = assertSafeIdentifier(event.asset.id, 'assetId');
      const briefId = assertSafeIdentifier(event.asset.briefId, 'briefId');
      const claimId = assertSafeIdentifier(event.claimId, 'claimId');
      assertSafeAssetPath(event.asset.path);
      if (event.asset.cardId !== undefined && event.asset.cardId !== cardId) {
        throw new VisualContractError('worker asset ownership is invalid', [
          { code: 'ownership_mismatch', field: 'asset.cardId' },
        ]);
      }
      if (event.asset.providerJobId !== null && event.asset.providerJobId !== undefined) {
        assertSafeIdentifier(event.asset.providerJobId, 'providerJobId');
      }
      payload.asset = {
        id: assetId,
        brief_id: briefId,
        repo_path: event.asset.path,
        provider_job_id: event.asset.providerJobId,
        version: event.asset.version,
      };
      payload.claim_id = claimId;
      payload.review = {
        id: `review-${jobId}`,
        asset_id: assetId,
        passed: event.review.passed === true,
        findings_json: Array.isArray(event.review.findings)
          ? event.review.findings.map(sanitizedFinding)
          : [],
      };
    }
    if (event.status === 'failed') {
      payload.claim_id = assertSafeIdentifier(event.claimId, 'claimId');
    }
    return postWorkerCallback({
      fetchImpl,
      root,
      workerToken,
      path: `/api/internal/flashcards/${encodeURIComponent(cardId)}/jobs/${encodeURIComponent(jobId)}/complete`,
      payload,
      expectedStatus: event.status,
    });
  };
  Object.defineProperty(reporter, 'durable', { value: true });
  Object.defineProperty(reporter, 'claim', {
    value: async ({ jobId, cardId, claimId }) => {
      const safeCardId = assertSafeIdentifier(cardId, 'cardId');
      const safeJobId = assertSafeIdentifier(jobId, 'jobId');
      const safeClaimId = assertSafeIdentifier(claimId, 'claimId');
      const body = await postWorkerCallback({
        fetchImpl,
        root,
        workerToken,
        path: `/api/internal/flashcards/${encodeURIComponent(safeCardId)}/jobs/${encodeURIComponent(safeJobId)}/claim`,
        payload: { claim_id: safeClaimId },
        expectedStatus: ['running', 'succeeded', 'failed'],
      });
      assertSafeIdentifier(body?.claim_id, 'claimId');
      return body;
    },
  });
  return reporter;
}

export function createWorkerPersistenceReporterFromEnv({ env = process.env, fetchImpl = fetch } = {}) {
  return createWorkerPersistenceReporter({
    baseUrl: env.MPS_VISUAL_WORKER_API_BASE_URL,
    workerToken: env.MPS_VISUAL_WORKER_TOKEN,
    fetchImpl,
  });
}

function normalizeGeneratedAsset(output, { jobId, cardId, briefId }) {
  const candidate = output?.asset ?? output;
  if (!candidate || typeof candidate !== 'object') {
    throw new Error('image generator must return an asset object');
  }
  const id = candidate.id ?? candidate.assetId;
  const path = candidate.path ?? candidate.repo_path ?? candidate.assetPath;
  if (typeof id !== 'string' || id.trim() === '') throw new Error('generated asset requires an ID');
  assertSafeIdentifier(id, 'assetId');
  if (typeof path !== 'string' || path.trim() === '') throw new Error('generated asset requires a path');
  assertSafeAssetPath(path);
  const version = candidate.version ?? 1;
  if (!Number.isInteger(version) || version < 1) throw new Error('generated asset version must be a positive integer');
  const providerJobId = candidate.providerJobId ?? candidate.provider_job_id ?? null;
  if (providerJobId !== null) assertSafeIdentifier(providerJobId, 'providerJobId');
  return {
    id,
    cardId,
    briefId,
    path,
    providerJobId,
    version,
    mimeType: candidate.mimeType ?? candidate.mime_type,
    width: candidate.width,
    height: candidate.height,
    sizeBytes: candidate.sizeBytes ?? candidate.size_bytes,
    jobId,
  };
}

async function emitStatus(report, event) {
  try {
    await report(event);
  } catch {
    // Status observers are telemetry; persistence is handled by the required
    // terminal persistence reporter and must not be replaced by a no-op.
  }
}

function resolveJobIdentifier(field, values) {
  const present = values.filter((value) => value !== undefined && value !== null);
  const value = present[0];
  assertSafeIdentifier(value, field);
  if (present.some((candidate) => candidate !== value)) {
    throw new VisualContractError('worker job ownership is inconsistent', [
      { code: 'ownership_mismatch', field },
    ]);
  }
  return value;
}

function resolveAttemptClaimId({ job, claimId, attemptId }) {
  const present = [
    claimId,
    attemptId,
    job?.claimId,
    job?.attemptId,
    job?.claim_id,
    job?.attempt_id,
    job?.input_json?.claim_id,
    job?.input_json?.attempt_id,
    job?.input?.claim_id,
    job?.input?.attempt_id,
  ].filter((value) => value !== undefined && value !== null);
  if (present.length === 0) return createAttemptClaimId();
  return resolveJobIdentifier('claimId', present);
}

/**
 * Runs one generation/review job. The returned status is deliberately limited
 * to worker outcomes: needs-review, revision-requested, or failed. Approval
 * and publication remain teacher/API actions and cannot be produced here.
 */
export async function processVisualJob({
  job,
  brief,
  exerciseContext,
  manifest,
  referenceAssets,
  generator,
  visionReviewer,
  report,
  persistenceReporter,
  claimId: requestedClaimId,
  attemptId: requestedAttemptId,
}) {
  if (typeof persistenceReporter !== 'function' || persistenceReporter.durable !== true) {
    throw new Error('a secure worker persistence reporter is required');
  }
  if (typeof persistenceReporter.claim !== 'function') {
    throw new Error('a secure worker claim reporter is required');
  }
  const jobId = resolveJobIdentifier('jobId', [job?.id]);
  const cardId = resolveJobIdentifier('cardId', [job?.cardId, job?.card_id, job?.input_json?.card_id, job?.input?.card_id]);
  const briefId = resolveJobIdentifier('briefId', [
    job?.briefId,
    job?.brief_id,
    job?.input_json?.brief_id,
    job?.input?.brief_id,
  ]);
  assertSafeIdentifier(brief?.id, 'brief.id');
  assertSafeIdentifier(brief?.card_id, 'brief.card_id');
  if (brief?.id !== briefId || brief?.card_id !== cardId) {
    throw new VisualContractError('worker job ownership does not match the visual brief', [
      { code: 'ownership_mismatch', field: 'brief' },
    ]);
  }
  const attemptClaimId = resolveAttemptClaimId({
    job,
    claimId: requestedClaimId,
    attemptId: requestedAttemptId,
  });
  let claim;
  try {
    claim = await persistenceReporter.claim({ jobId, cardId, claimId: attemptClaimId });
  } catch (error) {
    if (error instanceof RetryablePersistenceError) {
      throw new RetryablePersistenceError(attemptClaimId);
    }
    throw error;
  }
  const claimStatus = claim?.status;
  const claimId = assertSafeIdentifier(claim?.claim_id ?? claim?.claimId, 'claimId');
  if (claimStatus === 'running' && claimId !== attemptClaimId) {
    throw new RetryablePersistenceError(attemptClaimId);
  }
  if (claimStatus === 'succeeded' || claimStatus === 'failed') {
    return {
      jobId,
      cardId,
      claimId,
      status: claimStatus,
      reconciled: true,
    };
  }
  const status = createStatusReporter({ report });
  let persistenceAttempted = false;

  try {
    await emitStatus(status, { jobId, claimId, status: 'running' });
    const referenceErrors = validateReferenceAssets(referenceAssets, undefined, manifest);
    if (referenceErrors.length > 0) {
      throw new VisualContractError('reference assets do not satisfy the asset policy', referenceErrors);
    }
    const generationPayload = compileGenerationPayload({
      brief,
      exerciseContext,
      manifest,
      referenceAssets,
    });
    if (!generator || typeof generator.generateDraft !== 'function') {
      throw new Error('an image generator is required for generation jobs');
    }
    const generated = await generator.generateDraft({
      brief: generationPayload,
      referenceAssets: {
        canonicalSheet: generationPayload.character.canonicalSheet,
        references: generationPayload.character.referenceRoles,
      },
    });
    const asset = normalizeGeneratedAsset(generated, { jobId, cardId, briefId });
    const review = await reviewGeneratedAsset({
      asset,
      brief,
      generationPayload,
      visionReviewer,
    });
    const result = {
      jobId,
      cardId,
      claimId,
      status: 'succeeded',
      cardStatus: review.status,
      asset: { ...asset, status: review.assetStatus },
      review,
      generationPayload,
    };
    const terminalSuccess = {
      jobId,
      cardId,
      claimId,
      status: result.status,
      asset: result.asset,
      review: result.review,
    };
    try {
      await persistenceReporter(terminalSuccess);
      persistenceAttempted = true;
    } catch {
      try {
        await persistenceReporter(terminalSuccess);
        persistenceAttempted = true;
      } catch {
        throw new RetryablePersistenceError(claimId);
      }
    }
    await emitStatus(status, {
      jobId,
      cardId,
      claimId,
      status: 'succeeded',
      cardStatus: review.status,
      assetStatus: review.assetStatus,
    });
    return result;
  } catch (error) {
    if (error instanceof RetryablePersistenceError) {
      if (error.claimId === claimId) throw error;
      throw new RetryablePersistenceError(claimId);
    }
    const failure = {
      jobId,
      cardId,
      claimId,
      status: 'failed',
      cardStatus: 'revision-requested',
      error: publicError(error),
    };
    if (!persistenceAttempted) {
      persistenceAttempted = true;
      try {
        await persistenceReporter(failure);
      } catch {
        throw new RetryablePersistenceError(claimId);
      }
    }
    await emitStatus(status, failure);
    return failure;
  }
}

export function readWorkerProviderConfig(env = process.env) {
  const keyName = env.MPS_IMAGE_PROVIDER_KEY_NAME?.trim() || 'MPS_IMAGE_PROVIDER_API_KEY';
  const apiKey = env[keyName];
  if (!apiKey) throw new Error(`${keyName} must be configured in the visual worker environment`);
  return Object.freeze({
    provider: env.MPS_IMAGE_PROVIDER?.trim() || 'injected',
    apiKey,
  });
}
