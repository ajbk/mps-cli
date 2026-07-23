import {
  compileGenerationPayload,
  validateReferenceAssets,
  VisualContractError,
} from './visual-contract.mjs';
import { reviewGeneratedAsset } from './validator.mjs';

function publicError(error) {
  if (error instanceof VisualContractError) {
    return { code: 'visual_contract_invalid', message: error.message, details: error.errors };
  }
  return {
    code: 'visual_worker_failed',
    message: error instanceof Error ? error.message : 'visual worker failed',
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

function normalizeGeneratedAsset(output, jobId) {
  const candidate = output?.asset ?? output;
  if (!candidate || typeof candidate !== 'object') {
    throw new Error('image generator must return an asset object');
  }
  const id = candidate.id ?? candidate.assetId;
  const path = candidate.path ?? candidate.repo_path ?? candidate.assetPath;
  if (typeof id !== 'string' || id.trim() === '') throw new Error('generated asset requires an ID');
  if (typeof path !== 'string' || path.trim() === '') throw new Error('generated asset requires a path');
  const version = candidate.version ?? 1;
  if (!Number.isInteger(version) || version < 1) throw new Error('generated asset version must be a positive integer');
  return {
    id,
    cardId: candidate.cardId,
    briefId: candidate.briefId,
    path,
    providerJobId: candidate.providerJobId ?? candidate.provider_job_id ?? null,
    version,
    mimeType: candidate.mimeType ?? candidate.mime_type,
    width: candidate.width,
    height: candidate.height,
    sizeBytes: candidate.sizeBytes ?? candidate.size_bytes,
    jobId,
  };
}

async function emitStatus(report, event) {
  await report(event);
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
}) {
  const jobId = job?.id ?? 'unidentified-job';
  const status = createStatusReporter({ report });

  try {
    await emitStatus(status, { jobId, status: 'running' });
    const referenceErrors = validateReferenceAssets(referenceAssets);
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
      referenceAssets: generationPayload.character.referenceRoles,
    });
    const asset = normalizeGeneratedAsset(generated, jobId);
    const review = await reviewGeneratedAsset({
      asset,
      brief,
      generationPayload,
      visionReviewer,
    });
    const result = {
      jobId,
      status: 'succeeded',
      cardStatus: review.status,
      asset: { ...asset, status: review.assetStatus },
      review,
      generationPayload,
    };
    await emitStatus(status, {
      jobId,
      status: 'succeeded',
      cardStatus: review.status,
      assetStatus: review.assetStatus,
    });
    return result;
  } catch (error) {
    const failure = {
      jobId,
      status: 'failed',
      cardStatus: 'revision-requested',
      error: publicError(error),
    };
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
