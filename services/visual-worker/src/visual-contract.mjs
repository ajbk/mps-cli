import { readFile } from 'node:fs/promises';
import { posix } from 'node:path';

export const STYLE_PROFILE = 'mono-gesture-ink-pilates-v1';
export const CHARACTER_ID = 'teacher-01';
export const LOCKED_OUTFIT = 'off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts';
export const CHEEK_ACCENT = '#D98F9A';
export const CONTRACT_VERSION = 'visual-contract-1';

export const REQUIRED_NEGATIVE_CONSTRAINTS = Object.freeze([
  'arrows',
  'text',
  'logos',
  'watermark',
]);

export const REQUIRED_VISUAL_CHECKS = Object.freeze([
  'pose',
  'anatomy',
  'identity',
  'glasses_hair',
  'outfit',
  'apparatus',
  'linework',
  'cheek_accent',
  'forbidden_overlays',
]);

export const REQUIRED_REFERENCE_ROLES = Object.freeze([
  'face_identity',
  'full_body_front',
  'full_body_side',
  'full_body_back',
  'outfit',
  'equipment_context',
]);

export const DEFAULT_ASSET_POLICY = Object.freeze({
  repoPrefixes: Object.freeze([
    'web/assets/flashcard-images/',
    'docs/assets/flashcard-images/',
  ]),
  objectStorePrefixes: Object.freeze(['object://mps-flashcards/']),
});

const SAFE_IDENTIFIER_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$/;

const FORBIDDEN_PATH_PATTERNS = [
  /^\//,
  /^\\/,
  /^[A-Za-z]:[\\/]/,
  /^~/,
  /^file:/i,
  /^https?:/i,
  /^data:/i,
];

export class VisualContractError extends Error {
  constructor(message, errors = [message]) {
    super(message);
    this.name = 'VisualContractError';
    this.errors = errors;
  }
}

export function isSafeIdentifier(value) {
  return typeof value === 'string'
    && value !== '.'
    && value !== '..'
    && SAFE_IDENTIFIER_PATTERN.test(value);
}

export function assertSafeIdentifier(value, field = 'identifier') {
  if (!isSafeIdentifier(value)) {
    throw new VisualContractError('visual worker identifiers are invalid', [
      { code: 'invalid_identifier', field },
    ]);
  }
  return value;
}

function isRecord(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function pathHasTraversal(value) {
  return value.split('/').some((part) => part === '..' || part === '.');
}

function isSafeRelativePath(value) {
  return typeof value === 'string'
    && value.length > 0
    && !/[\u0000-\u001F\u007F]/.test(value)
    && !value.includes('\\')
    && !value.split('/').some((part) => part.length === 0)
    && !FORBIDDEN_PATH_PATTERNS.some((pattern) => pattern.test(value))
    && !pathHasTraversal(value)
    && posix.normalize(value) === value;
}

export function isSafeAssetPath(value, policy = DEFAULT_ASSET_POLICY) {
  if (typeof value !== 'string' || value.length === 0 || /[\u0000-\u001F\u007F]/.test(value)) return false;
  const objectPrefix = policy.objectStorePrefixes.find((prefix) => value.startsWith(prefix));
  if (objectPrefix) {
    const key = value.slice(objectPrefix.length);
    return key.length > 0
      && !key.includes('\\')
      && !key.split('/').some((part) => part.length === 0)
      && !pathHasTraversal(key)
      && posix.normalize(key) === key;
  }
  if (!isSafeRelativePath(value)) return false;
  return policy.repoPrefixes.some((prefix) => value.startsWith(prefix));
}

export function assertSafeAssetPath(value, policy = DEFAULT_ASSET_POLICY) {
  if (!isSafeAssetPath(value, policy)) {
    throw new VisualContractError(
      'asset path must be an approved repo-relative path or object-store key',
      [{ code: 'unsafe_asset_path', field: 'asset.path' }],
    );
  }
  return value;
}

function isSafePublicPath(value) {
  return isSafeRelativePath(value) && !value.startsWith('private://');
}

function normalizedText(value) {
  return String(JSON.stringify(value ?? ''))
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ' ')
    .trim();
}

function containsConstraint(value, aliases) {
  const text = normalizedText(value);
  return aliases.some((alias) => text.includes(alias));
}

export function missingNegativeConstraints(value) {
  const aliases = {
    arrows: ['arrow', 'arrows'],
    text: ['text', 'label', 'labels', 'lettering'],
    logos: ['logo', 'logos', 'branding'],
    watermark: ['watermark', 'water mark'],
  };
  return REQUIRED_NEGATIVE_CONSTRAINTS.filter(
    (constraint) => !containsConstraint(value, aliases[constraint]),
  );
}

function cheekAccentFrom(brief) {
  return brief?.palette_json?.cheekAccent
    ?? brief?.palette_json?.cheek_accent
    ?? brief?.palette?.cheekAccent
    ?? brief?.palette?.cheek_accent;
}

function poseLandmarks(pose) {
  return pose?.landmarks ?? pose?.poseLandmarks ?? [];
}

function contactPoints(pose) {
  return pose?.contact_points ?? pose?.contactPoints ?? [];
}

function canonicalExercise(exerciseContext) {
  return exerciseContext?.exercise ?? exerciseContext ?? {};
}

export function validateVisualManifest(manifest) {
  const errors = [];
  if (!isRecord(manifest)) return ['manifest must be an object'];
  if (manifest.primaryStyleProfile !== STYLE_PROFILE) {
    errors.push('manifest primaryStyleProfile must be mono-gesture-ink-pilates-v1');
  }
  const character = manifest.character;
  if (!isRecord(character)) return [...errors, 'manifest character must be an object'];
  if (character.id !== CHARACTER_ID) errors.push('manifest character must be teacher-01');
  if (character.status === 'rejected') errors.push('manifest character reference is rejected');
  if (!isSafePublicPath(character.canonicalSheet)) {
    errors.push('manifest canonicalSheet must be a safe repo-relative public path');
  }
  const attributes = character.fixedAttributes;
  if (!isRecord(attributes)) {
    errors.push('manifest character.fixedAttributes must be an object');
  } else {
    if (attributes.top !== 'fitted off-white cropped Pilates camisole with thin straps') {
      errors.push('manifest top does not match the locked outfit');
    }
    if (attributes.bottom !== 'dark charcoal high-waisted fitted Pilates biker shorts ending mid-thigh') {
      errors.push('manifest bottom does not match the locked outfit');
    }
  }
  if (!Array.isArray(character.references)) {
    errors.push('manifest character.references must be an array');
  } else {
    const roles = new Set(character.references.map((reference) => reference.role));
    for (const role of REQUIRED_REFERENCE_ROLES) {
      if (!roles.has(role)) errors.push(`manifest is missing reference role: ${role}`);
    }
    for (const reference of character.references) {
      if (typeof reference.id !== 'string' || typeof reference.role !== 'string') {
        errors.push('manifest references require id and role');
      }
      if (typeof reference.source !== 'string' || !reference.source.startsWith('private://')) {
        errors.push(`manifest reference ${reference.id ?? '<unknown>'} must use a private:// logical ID`);
      }
    }
  }
  return errors;
}

export function validateReferenceAssets(
  referenceAssets,
  policy = DEFAULT_ASSET_POLICY,
  manifest = undefined,
) {
  if (!isRecord(referenceAssets)) {
    return [{ code: 'invalid_reference_assets', field: 'referenceAssets' }];
  }
  const errors = [];
  const canonicalSheet = referenceAssets.canonicalSheet;
  if (!isRecord(canonicalSheet)) {
    errors.push({ code: 'canonical_sheet_required', field: 'referenceAssets.canonicalSheet' });
  } else {
    const path = canonicalSheet.path ?? canonicalSheet.assetPath ?? canonicalSheet.key;
    if (!isSafeIdentifier(canonicalSheet.id)) {
      errors.push({ code: 'invalid_canonical_sheet', field: 'referenceAssets.canonicalSheet.id' });
    }
    if (!Number.isInteger(canonicalSheet.version) || canonicalSheet.version < 1) {
      errors.push({ code: 'invalid_canonical_sheet', field: 'referenceAssets.canonicalSheet.version' });
    }
    if (!isSafeAssetPath(path, policy)) {
      errors.push({ code: 'unsafe_canonical_sheet_path', field: 'referenceAssets.canonicalSheet.path' });
    }
  }
  if (!Array.isArray(referenceAssets.references)) {
    errors.push({ code: 'reference_pack_required', field: 'referenceAssets.references' });
    return errors;
  }
  const roles = new Set();
  const ids = new Set();
  const manifestReferences = isRecord(manifest?.character)
    && Array.isArray(manifest.character.references)
    ? manifest.character.references
    : [];
  const requiredManifestReferences = manifestReferences.filter((reference) => reference.required !== false);
  const manifestByRole = new Map(manifestReferences.map((reference) => [reference.role, reference]));
  if (manifestByRole.size > 0) {
    const expectedCanonicalSheetId = `${manifest.character.id}-v${manifest.character.version}`;
    if (canonicalSheet?.id !== expectedCanonicalSheetId) {
      errors.push({ code: 'canonical_sheet_identity_mismatch', field: 'referenceAssets.canonicalSheet.id' });
    }
  }
  for (const [index, reference] of referenceAssets.references.entries()) {
    if (!isRecord(reference)) {
      errors.push({ code: 'invalid_reference_asset', field: `referenceAssets[${index}]` });
      continue;
    }
    const path = reference.path ?? reference.assetPath ?? reference.key;
    if (!isSafeIdentifier(reference.id) || typeof reference.role !== 'string' || reference.role.length === 0) {
      errors.push({ code: 'invalid_reference_asset', field: `referenceAssets[${index}]` });
    }
    roles.add(reference.role);
    if (ids.has(reference.id)) {
      errors.push({ code: 'duplicate_reference_id', field: `referenceAssets[${index}].id` });
    }
    ids.add(reference.id);
    const manifestReference = manifestByRole.get(reference.role);
    if (manifestByRole.size > 0 && !manifestReference) {
      errors.push({ code: 'reference_role_not_in_manifest', field: `referenceAssets[${index}].role` });
    } else if (manifestReference && reference.id !== manifestReference.id) {
      errors.push({ code: 'reference_id_mismatch', field: `referenceAssets[${index}].id` });
    }
    if (!isSafeAssetPath(path, policy)) {
      errors.push({ code: 'unsafe_reference_asset_path', field: `referenceAssets[${index}].path` });
    }
  }
  for (const role of REQUIRED_REFERENCE_ROLES) {
    if (!roles.has(role)) errors.push({ code: 'reference_role_missing', field: `referenceAssets.references.${role}` });
  }
  if (manifestByRole.size > 0) {
    for (const reference of requiredManifestReferences) {
      if (!roles.has(reference.role)) {
        errors.push({ code: 'reference_role_missing', field: `referenceAssets.references.${reference.role}` });
      }
    }
  }
  return errors;
}

export function assertValidVisualBrief({ brief, exerciseContext, manifest }) {
  const errors = [];
  if (!isRecord(brief)) errors.push({ code: 'invalid_brief', field: 'brief' });
  if (!isRecord(manifest)) errors.push({ code: 'invalid_manifest', field: 'manifest' });
  if (!isRecord(exerciseContext)) errors.push({ code: 'invalid_exercise_context', field: 'exerciseContext' });

  if (isRecord(brief)) {
    if (brief.style_profile !== STYLE_PROFILE) {
      errors.push({ code: 'style_profile_locked', field: 'style_profile' });
    }
    if (brief.character_id !== CHARACTER_ID) {
      errors.push({ code: 'character_locked', field: 'character_id' });
    }
    if (brief.outfit !== LOCKED_OUTFIT) {
      errors.push({ code: 'outfit_locked', field: 'outfit' });
    }
    if (cheekAccentFrom(brief) !== CHEEK_ACCENT) {
      errors.push({ code: 'cheek_accent_locked', field: 'palette_json.cheekAccent' });
    }
    if (!isRecord(brief.pose_json)) {
      errors.push({ code: 'pose_required', field: 'pose_json' });
    }
    const missing = missingNegativeConstraints(brief.must_not_show_json);
    for (const constraint of missing) {
      errors.push({ code: 'negative_constraint_missing', field: 'must_not_show_json', value: constraint });
    }
  }

  if (isRecord(exerciseContext) && isRecord(brief)) {
    const exercise = canonicalExercise(exerciseContext);
    if (brief.exercise_id !== exercise.id) {
      errors.push({ code: 'exercise_locked', field: 'exercise_id' });
    }
    if (brief.apparatus !== exercise.apparatus) {
      errors.push({ code: 'apparatus_locked', field: 'apparatus' });
    }
  }

  if (isRecord(manifest)) {
    errors.push(...validateVisualManifest(manifest).map((message) => ({
      code: 'manifest_invalid',
      field: 'manifest',
      message,
    })));
  }

  if (errors.length > 0) {
    throw new VisualContractError('visual brief does not satisfy the locked contract', errors);
  }
  return true;
}

function stringConstraints(value) {
  if (typeof value === 'string' && value.trim() !== '') return [value.trim()];
  if (Array.isArray(value)) return value.flatMap(stringConstraints);
  return [];
}

export function compileGenerationPayload({ brief, exerciseContext, manifest, referenceAssets }) {
  assertValidVisualBrief({ brief, exerciseContext, manifest });
  const referenceErrors = validateReferenceAssets(referenceAssets, DEFAULT_ASSET_POLICY, manifest);
  if (referenceErrors.length > 0) {
    throw new VisualContractError('reference assets do not satisfy the asset policy', referenceErrors);
  }

  const exercise = canonicalExercise(exerciseContext);
  const references = referenceAssets.references.map((reference) => ({
      id: reference.id,
      role: reference.role,
      path: reference.path ?? reference.assetPath ?? reference.key,
    }));
  const canonicalSheet = referenceAssets.canonicalSheet;
  if (canonicalSheet.version !== manifest.character.version) {
    throw new VisualContractError('canonical character sheet version is not current', [
      { code: 'canonical_sheet_version_mismatch', field: 'referenceAssets.canonicalSheet.version' },
    ]);
  }
  const pose = brief.pose_json;
  const mustShow = Array.isArray(brief.must_show_json)
    ? brief.must_show_json
    : [brief.must_show_json].filter((value) => value !== undefined);

  return {
    schemaVersion: 1,
    contractVersion: CONTRACT_VERSION,
    styleProfile: STYLE_PROFILE,
    character: {
      id: CHARACTER_ID,
      referenceVersion: manifest.character.version,
      canonicalSheet: {
        id: canonicalSheet.id,
        version: canonicalSheet.version,
        path: canonicalSheet.path ?? canonicalSheet.assetPath ?? canonicalSheet.key,
      },
      referenceRoles: references,
      outfit: LOCKED_OUTFIT,
      cheekAccent: CHEEK_ACCENT,
    },
    exercise: {
      id: exercise.id,
      name: exercise.name ?? exercise.exercise,
      apparatus: exercise.apparatus,
      equipmentKey: exercise.equipment_key ?? exercise.equipmentKey,
    },
    pose: {
      source: pose,
      landmarks: poseLandmarks(pose),
      contactPoints: contactPoints(pose),
    },
    palette: {
      paper: '#FFFFFF',
      ink: '#292526',
      charcoal: '#3B3739',
      skinAccent: '#D7B7A0',
      cheekAccent: CHEEK_ACCENT,
    },
    mustShow,
    negativeConstraints: [...new Set([
      ...REQUIRED_NEGATIVE_CONSTRAINTS,
      ...stringConstraints(brief.must_not_show_json),
    ])],
    output: {
      background: 'clean white paper',
      noTextLayer: true,
      noLogoLayer: true,
      noWatermarkLayer: true,
    },
  };
}

export async function loadVisualManifest(filePath) {
  const manifest = JSON.parse(await readFile(filePath, 'utf8'));
  const errors = validateVisualManifest(manifest);
  if (errors.length > 0) {
    throw new VisualContractError('visual manifest is invalid', errors);
  }
  return manifest;
}
