import {
  assertSafeAssetPath,
  REQUIRED_VISUAL_CHECKS,
  VisualContractError,
} from './visual-contract.mjs';

export const VALIDATOR_VERSION = 'visual-validator-1';
export { REQUIRED_VISUAL_CHECKS };

function isRecord(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function fieldValue(asset, ...names) {
  for (const name of names) {
    if (asset?.[name] !== undefined) return asset[name];
  }
  return undefined;
}

export function metadataFindings({ asset }) {
  const findings = [];
  const path = fieldValue(asset, 'path', 'repo_path', 'assetPath');
  try {
    assertSafeAssetPath(path);
  } catch (error) {
    findings.push({
      code: 'unsafe_asset_path',
      severity: 'error',
      message: error instanceof VisualContractError ? error.message : 'asset path is not safe',
    });
  }

  const mimeType = fieldValue(asset, 'mimeType', 'mime_type');
  if (typeof mimeType !== 'string' || !/^image\/(png|jpeg|webp)$/i.test(mimeType)) {
    findings.push({
      code: 'unsupported_image_type',
      severity: 'error',
      message: 'generated asset must be a PNG, JPEG, or WebP image',
    });
  }

  for (const [field, value] of [
    ['width', fieldValue(asset, 'width')],
    ['height', fieldValue(asset, 'height')],
    ['sizeBytes', fieldValue(asset, 'sizeBytes', 'size_bytes')],
  ]) {
    if (value !== undefined && (!Number.isInteger(value) || value <= 0)) {
      findings.push({
        code: 'invalid_asset_metadata',
        severity: 'error',
        field,
        message: `${field} must be a positive integer when provided`,
      });
    }
  }

  if (['approved', 'published'].includes(asset?.status)) {
    findings.push({
      code: 'worker_cannot_approve',
      severity: 'error',
      message: 'worker output cannot be approved or published',
    });
  }
  return findings;
}

function checksFrom(review) {
  if (isRecord(review?.checks)) return review.checks;
  if (Array.isArray(review?.checks)) {
    return Object.fromEntries(review.checks
      .filter((check) => isRecord(check) && typeof check.name === 'string')
      .map((check) => [check.name, check]));
  }
  return {};
}

function sanitizedChecks(review) {
  const source = checksFrom(review);
  return Object.fromEntries(REQUIRED_VISUAL_CHECKS.map((name) => [name, {
    passed: source[name]?.passed === true,
  }]));
}

function sanitizedProviderFindings(review) {
  if (!Array.isArray(review?.findings)) return [];
  return review.findings.map((finding) => ({
    code: 'vision_finding',
    severity: finding?.severity === 'warning' ? 'warning' : 'error',
  }));
}

function visionFindings(review) {
  const findings = sanitizedProviderFindings(review);
  const source = checksFrom(review);
  const checks = sanitizedChecks(review);
  for (const name of REQUIRED_VISUAL_CHECKS) {
    const check = source[name];
    if (!isRecord(check) || check.passed !== true) {
      findings.push({
        code: 'visual_check_failed',
        severity: 'error',
        check: name,
        message: `vision reviewer failed ${name}`,
      });
    }
  }
  return { findings, checks };
}

export async function reviewGeneratedAsset({
  asset,
  brief,
  generationPayload,
  visionReviewer,
}) {
  const metadata = metadataFindings({ asset });
  let findings = metadata;
  let checks = {};
  let vision = null;

  if (metadata.length === 0) {
    if (!visionReviewer || typeof visionReviewer.review !== 'function') {
      findings = [{
        code: 'vision_reviewer_unavailable',
        severity: 'error',
        message: 'an injectable vision reviewer is required for automated QA',
      }];
    } else {
      vision = await visionReviewer.review({ asset, brief, generationPayload });
      const normalized = visionFindings(vision);
      checks = normalized.checks;
      findings = normalized.findings;
      if (vision?.passed !== true && findings.length === 0) {
        findings = [{
          code: 'vision_review_failed',
          severity: 'error',
          message: 'vision reviewer did not pass the generated asset',
        }];
      }
    }
  }

  const passed = findings.length === 0 && vision?.passed === true;
  return {
    validatorVersion: VALIDATOR_VERSION,
    passed,
    status: passed ? 'needs-review' : 'revision-requested',
    assetStatus: passed ? 'needs-review' : 'rejected',
    findings,
    checks,
  };
}
