import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import vm from 'node:vm';

async function loadReview() {
    const source = await readFile(new URL('./flashcard-review.js', import.meta.url), 'utf8');
    const window = {};
    vm.runInNewContext(source, { window });
    return window.MPS_FLASHCARD_REVIEW;
}

function card(overrides = {}) {
    return {
        status: 'draft',
        version: 1,
        automated_review: { status: 'pending', version: 1 },
        teacher_review: null,
        source_snapshot: {
            exercise_id: 'M02',
            style_profile: 'mono-gesture-ink-pilates-v1',
            name: 'Pelvic Clock',
            category: 'Mat',
            apparatus: 'Mat',
            level: 'Beginner',
            objective: 'Pelvic awareness'
        },
        source_exercise_id: 'M02',
        style_profile: 'mono-gesture-ink-pilates-v1',
        name: 'Pelvic Clock',
        category: 'Mat',
        apparatus: 'Mat',
        level: 'Beginner',
        objective: 'Pelvic awareness',
        character_id: 'teacher-01',
        outfit: 'off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts',
        cheek_accent: '#D98F9A',
        ...overrides
    };
}

const canonicalSource = {
    exercise_id: 'M02',
    style_profile: 'mono-gesture-ink-pilates-v1',
    name: 'Pelvic Clock',
    category: 'Mat',
    apparatus: 'Mat',
    level: 'Beginner',
    objective: 'Pelvic awareness'
};

test('a draft can move to generating', async () => {
    const review = await loadReview();

    const generating = review.transition(card(), 'generate');
    assert.equal(generating.status, 'generating');
    assert.equal(generating.version, 2);
    assert.equal(generating.automated_review.status, 'pending');
    assert.equal(generating.automated_review.version, 2);
    assert.equal(generating.teacher_review, null);
});

test('a generating card cannot be approved', async () => {
    const review = await loadReview();

    assert.throws(() => review.transition(card({ status: 'generating' }), 'approve'), /cannot approve/i);
});

test('a needs-review card with failed automated review cannot be approved', async () => {
    const review = await loadReview();
    const pendingApproval = card({ status: 'needs-review', automated_review: { status: 'failed', version: 1 } });

    assert.throws(() => review.transition(pendingApproval, 'approve'), /automated review/i);
});

test('a passed needs-review card can become approved', async () => {
    const review = await loadReview();
    const pendingApproval = card({ status: 'needs-review', automated_review: { status: 'passed', version: 1 } });

    const approved = review.transition(pendingApproval, 'approve', { reviewer: 'Teacher', canonicalSource });
    assert.equal(approved.status, 'approved');
    assert.equal(approved.teacher_review.reviewer, 'Teacher');
    assert.equal(approved.teacher_review.version, 1);
});

test('only an approved card can become published', async () => {
    const review = await loadReview();

    assert.throws(() => review.transition(card({ status: 'needs-review', automated_review: { status: 'passed', version: 1 } }), 'publish', { canonicalSource }), /cannot publish/i);
    assert.equal(review.transition(card({
        status: 'approved',
        automated_review: { status: 'passed', version: 1 },
        teacher_review: { reviewer: 'Teacher', status: 'approved', version: 1 }
    }), 'publish', { canonicalSource }).status, 'published');
});

test('publish guard fails closed when source exercise or style profile changed', async () => {
    const review = await loadReview();
    const approved = card({
        status: 'approved',
        automated_review: { status: 'passed', version: 1 },
        teacher_review: { reviewer: 'Teacher', status: 'approved', version: 1 }
    });

    assert.equal(review.canPublish(approved, canonicalSource), true);
    assert.equal(review.canPublish({ ...approved, source_exercise_id: 'M03' }, canonicalSource), false);
    assert.equal(review.canPublish({ ...approved, style_profile: 'other-style' }, canonicalSource), false);
});

test('regeneration clears stale reviews and increments the asset version', async () => {
    const review = await loadReview();
    const approved = card({
        status: 'approved',
        automated_review: { status: 'passed', version: 1 },
        teacher_review: { reviewer: 'Teacher', status: 'approved', version: 1 },
        current_asset_id: 'asset-v1',
        proposed_changes: { cue: 'Old generated cue' }
    });

    const revisionRequested = review.transition(approved, 'request-revision');
    const generating = review.transition(revisionRequested, 'generate');

    assert.equal(generating.version, 2);
    assert.equal(generating.current_asset_id, null);
    assert.equal(generating.teacher_review, null);
    assert.equal(generating.proposed_changes, null);
    assert.equal(review.canPublish(generating, canonicalSource), false);
});

test('canonical source prevents a mutable snapshot bypass', async () => {
    const review = await loadReview();
    const approved = card({
        status: 'approved',
        source_exercise_id: 'M03',
        style_profile: 'other-style',
        source_snapshot: { exercise_id: 'M03', style_profile: 'other-style' },
        automated_review: { status: 'passed', version: 1 },
        teacher_review: { reviewer: 'Teacher', status: 'approved', version: 1 }
    });

    assert.equal(review.sourceIsUnchanged(approved, canonicalSource), false);
    assert.equal(review.canPublish(approved, canonicalSource), false);
});

test('canonical source rejects a hostile mutation of every workbook-locked field', async () => {
    const review = await loadReview();
    const approved = card({
        status: 'approved',
        name: 'Hostile name',
        category: 'Chair',
        apparatus: 'Chair',
        level: 'Advanced',
        objective: 'Hostile objective',
        source_snapshot: {
            ...canonicalSource,
            name: 'Hostile name',
            category: 'Chair',
            apparatus: 'Chair',
            level: 'Advanced',
            objective: 'Hostile objective'
        },
        automated_review: { status: 'passed', version: 1 },
        teacher_review: { reviewer: 'Teacher', status: 'approved', version: 1 }
    });

    assert.equal(review.sourceIsUnchanged(approved, canonicalSource), false);
    assert.equal(review.canPublish(approved, canonicalSource), false);
});

test('a rejected or stale teacher review cannot publish', async () => {
    const review = await loadReview();
    const approved = card({
        status: 'approved',
        automated_review: { status: 'passed', version: 1 },
        teacher_review: { reviewer: 'Teacher', status: 'rejected', version: 1 }
    });

    assert.equal(review.canPublish(approved, canonicalSource), false);
    assert.equal(review.canPublish({
        ...approved,
        teacher_review: { reviewer: 'Teacher', status: 'approved', version: 0 }
    }, canonicalSource), false);
});

test('an explicit teacher edit invalidates an approved card until it is regenerated and reviewed', async () => {
    const review = await loadReview();
    const approved = card({
        status: 'approved',
        automated_review: { status: 'passed', version: 1 },
        teacher_review: { reviewer: 'Teacher', status: 'approved', version: 1 },
        current_asset_id: 'asset-v1',
        image: 'assets/flashcard-images/mat/m02.png'
    });

    const edited = review.invalidateTeacherEdits({ ...approved, cue: 'Teacher edit' });

    assert.equal(edited.status, 'draft');
    assert.equal(edited.version, 2);
    assert.equal(edited.automated_review, null);
    assert.equal(edited.teacher_review, null);
    assert.equal(edited.current_asset_id, null);
    assert.equal(edited.image, null);
    assert.equal(review.canPublish(edited, canonicalSource), false);
});

test('a card must match the trusted selected draft ID before canonical resolution', async () => {
    const review = await loadReview();

    assert.equal(review.matchesTrustedDraft(card({ id: 'draft:M02' }), 'draft:M02'), true);
    assert.equal(review.matchesTrustedDraft(card({ id: 'draft:M03' }), 'draft:M02'), false);
});

test('a legacy draft hydrates only missing locked source fields from its canonical source', async () => {
    const review = await loadReview();
    const legacy = card({
        apparatus: undefined,
        source_snapshot: {
            exercise_id: 'M02',
            style_profile: 'mono-gesture-ink-pilates-v1'
        },
        teacher_review: { reviewer: 'Teacher', status: 'approved', version: 1 }
    });

    const hydrated = review.hydrateLegacySource(legacy, canonicalSource);

    assert.equal(hydrated.apparatus, 'Mat');
    assert.equal(hydrated.source_snapshot.objective, 'Pelvic awareness');
    assert.equal(hydrated.teacher_review.reviewer, 'Teacher');
    assert.equal(review.hydrateLegacySource(card({ name: 'Mismatched name' }), canonicalSource), null);
});
