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
        automated_review: { status: 'pending' },
        teacher_review: null,
        source_snapshot: { exercise_id: 'M02', style_profile: 'mono-gesture-ink-pilates-v1' },
        source_exercise_id: 'M02',
        style_profile: 'mono-gesture-ink-pilates-v1',
        character_id: 'teacher-01',
        outfit: 'off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts',
        cheek_accent: '#D98F9A',
        ...overrides
    };
}

test('a draft can move to generating', async () => {
    const review = await loadReview();

    assert.equal(review.transition(card(), 'generate').status, 'generating');
});

test('a generating card cannot be approved', async () => {
    const review = await loadReview();

    assert.throws(() => review.transition(card({ status: 'generating' }), 'approve'), /cannot approve/i);
});

test('a needs-review card with failed automated review cannot be approved', async () => {
    const review = await loadReview();
    const pendingApproval = card({ status: 'needs-review', automated_review: { status: 'failed' } });

    assert.throws(() => review.transition(pendingApproval, 'approve'), /automated review/i);
});

test('a passed needs-review card can become approved', async () => {
    const review = await loadReview();
    const pendingApproval = card({ status: 'needs-review', automated_review: { status: 'passed' } });

    const approved = review.transition(pendingApproval, 'approve', { reviewer: 'Teacher' });
    assert.equal(approved.status, 'approved');
    assert.equal(approved.teacher_review.reviewer, 'Teacher');
});

test('only an approved card can become published', async () => {
    const review = await loadReview();

    assert.throws(() => review.transition(card({ status: 'needs-review', automated_review: { status: 'passed' } }), 'publish'), /cannot publish/i);
    assert.equal(review.transition(card({
        status: 'approved',
        automated_review: { status: 'passed' },
        teacher_review: { reviewer: 'Teacher' }
    }), 'publish').status, 'published');
});

test('publish guard fails closed when source exercise or style profile changed', async () => {
    const review = await loadReview();
    const approved = card({
        status: 'approved',
        automated_review: { status: 'passed' },
        teacher_review: { reviewer: 'Teacher' }
    });

    assert.equal(review.canPublish(approved), true);
    assert.equal(review.canPublish({ ...approved, source_exercise_id: 'M03' }), false);
    assert.equal(review.canPublish({ ...approved, style_profile: 'other-style' }), false);
});
