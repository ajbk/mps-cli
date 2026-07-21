#!/usr/bin/env node

import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const CHARACTER_STATUSES = new Set(['draft', 'needs-review', 'approved', 'rejected']);
const CARD_STATUSES = new Set(['draft', 'needs-review', 'approved', 'rejected']);
const REQUIRED_REFERENCE_ROLES = new Set([
  'face_identity',
  'full_body_front',
  'full_body_side',
  'full_body_back',
  'outfit',
  'equipment_context',
]);

function isRecord(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function isMachineLocalPath(value) {
  return typeof value === 'string' && (
    value.startsWith('/') ||
    value.startsWith('\\') ||
    /^[A-Za-z]:[\\/]/.test(value) ||
    value.startsWith('~') ||
    value.includes('/Users/') ||
    value.includes('/home/') ||
    value.includes('\\Users\\') ||
    value.includes('\\Users\\')
  );
}

function checkProjectPath(value, label, errors) {
  if (typeof value !== 'string' || value.length === 0) {
    errors.push(`${label} must be a non-empty path`);
    return;
  }
  if (isMachineLocalPath(value)) {
    errors.push(`${label} contains a machine-local path; use a repo-relative path or private:// asset ID`);
  }
}

function checkUniqueIds(items, label, errors) {
  const ids = new Set();
  for (const [index, item] of items.entries()) {
    if (!isRecord(item) || typeof item.id !== 'string' || item.id.length === 0) {
      errors.push(`${label}[${index}].id must be a non-empty string`);
      continue;
    }
    if (ids.has(item.id)) errors.push(`${label} contains duplicate id: ${item.id}`);
    ids.add(item.id);
  }
}

export function validateManifest(manifest) {
  const errors = [];

  if (!isRecord(manifest)) return ['manifest must be a JSON object'];
  if (manifest.schemaVersion !== 1) errors.push('schemaVersion must be 1');
  if (!isRecord(manifest.character)) {
    errors.push('character must be an object');
    return errors;
  }

  const character = manifest.character;
  if (typeof character.id !== 'string' || character.id.length === 0) errors.push('character.id must be a non-empty string');
  if (!CHARACTER_STATUSES.has(character.status)) errors.push(`character.status must be one of: ${[...CHARACTER_STATUSES].join(', ')}`);
  checkProjectPath(character.canonicalSheet, 'character.canonicalSheet', errors);

  if (!Array.isArray(character.references)) {
    errors.push('character.references must be an array');
  } else {
    checkUniqueIds(character.references, 'character.references', errors);
    const roles = new Set();
    for (const [index, reference] of character.references.entries()) {
      if (!isRecord(reference)) {
        errors.push(`character.references[${index}] must be an object`);
        continue;
      }
      if (typeof reference.role !== 'string' || reference.role.length === 0) errors.push(`character.references[${index}].role must be a non-empty string`);
      if (typeof reference.source !== 'string' || reference.source.length === 0) errors.push(`character.references[${index}].source must be a non-empty string`);
      if (typeof reference.role === 'string') roles.add(reference.role);
      if (typeof reference.source === 'string' && isMachineLocalPath(reference.source)) {
        errors.push(`character.references[${index}].source contains a machine-local path; use a private:// asset ID`);
      }
    }
    for (const role of REQUIRED_REFERENCE_ROLES) {
      if (!roles.has(role)) errors.push(`character reference pack is missing required role: ${role}`);
    }
  }

  if (manifest.cards !== undefined && !Array.isArray(manifest.cards)) {
    errors.push('cards must be an array when provided');
  } else if (Array.isArray(manifest.cards)) {
    checkUniqueIds(manifest.cards, 'cards', errors);
    const referenceIds = new Set((character.references || []).map((reference) => reference.id));
    for (const [index, card] of manifest.cards.entries()) {
      if (!isRecord(card)) {
        errors.push(`cards[${index}] must be an object`);
        continue;
      }
      if (!CARD_STATUSES.has(card.status)) errors.push(`cards[${index}].status must be one of: ${[...CARD_STATUSES].join(', ')}`);
      checkProjectPath(card.image, `cards[${index}].image`, errors);
      if (!Array.isArray(card.referenceIds) || card.referenceIds.length === 0) {
        errors.push(`cards[${index}].referenceIds must list the references used for the image`);
      } else {
        for (const id of card.referenceIds) {
          if (!referenceIds.has(id)) errors.push(`cards[${index}] references unknown reference id: ${id}`);
        }
        const rolesUsed = new Set((character.references || [])
          .filter((reference) => card.referenceIds.includes(reference.id))
          .map((reference) => reference.role));
        if (!rolesUsed.has('face_identity')) errors.push(`cards[${index}] must use a face_identity reference`);
        if (![...rolesUsed].some((role) => role.startsWith('full_body_'))) errors.push(`cards[${index}] must use a full_body_* reference`);
      }
      if (card.status === 'approved' && character.status !== 'approved') {
        errors.push(`cards[${index}] cannot be approved before the approved character sheet`);
      }
    }
  }

  return errors;
}

async function main() {
  const manifestPath = process.argv[2] || 'data/reference/pilates_visual_manifest.json';
  let manifest;
  try {
    manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  } catch (error) {
    console.error(`Could not read JSON manifest ${manifestPath}: ${error.message}`);
    process.exitCode = 1;
    return;
  }

  const errors = validateManifest(manifest);
  if (errors.length > 0) {
    console.error(`Visual manifest invalid: ${manifestPath}`);
    for (const error of errors) console.error(`- ${error}`);
    process.exitCode = 1;
    return;
  }
  console.log(`Visual manifest valid: ${manifestPath}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
