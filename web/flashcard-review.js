(function attachFlashcardReview() {
    const visualContract = {
        style_profile: 'mono-gesture-ink-pilates-v1',
        character_id: 'teacher-01',
        outfit: 'off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts',
        cheek_accent: '#D98F9A'
    };

    function automatedReviewPassed(card) {
        return card?.automated_review?.status === 'passed';
    }

    function sourceIsUnchanged(card) {
        const snapshot = card?.source_snapshot;
        return Boolean(snapshot)
            && snapshot.exercise_id === card.source_exercise_id
            && snapshot.style_profile === card.style_profile;
    }

    function visualContractMatches(card) {
        return Object.entries(visualContract).every(([key, value]) => card?.[key] === value);
    }

    function canPublish(card) {
        return card?.status === 'approved'
            && automatedReviewPassed(card)
            && Boolean(card.teacher_review)
            && sourceIsUnchanged(card)
            && visualContractMatches(card);
    }

    function transition(card, action, details = {}) {
        if (!card) throw new Error('A flashcard is required');

        if (action === 'generate') {
            if (!['draft', 'revision-requested'].includes(card.status)) {
                throw new Error('Cannot generate this flashcard');
            }
            return { ...card, status: 'generating' };
        }

        if (action === 'submit-review') {
            if (card.status !== 'generating') throw new Error('Cannot submit this flashcard for review');
            return { ...card, status: 'needs-review' };
        }

        if (action === 'request-revision') {
            if (!['needs-review', 'approved'].includes(card.status)) {
                throw new Error('Cannot request a revision for this flashcard');
            }
            return { ...card, status: 'revision-requested', revision_request: details.note || '' };
        }

        if (action === 'approve') {
            if (card.status !== 'needs-review') throw new Error('Cannot approve this flashcard');
            if (!automatedReviewPassed(card)) throw new Error('Automated review must pass before approval');
            return {
                ...card,
                status: 'approved',
                teacher_review: { reviewer: details.reviewer || 'Teacher', status: 'approved' }
            };
        }

        if (action === 'publish') {
            if (!canPublish(card)) throw new Error('Cannot publish this flashcard until every review guard passes');
            return { ...card, status: 'published' };
        }

        throw new Error(`Unknown flashcard action: ${action}`);
    }

    window.MPS_FLASHCARD_REVIEW = {
        visualContract,
        automatedReviewPassed,
        sourceIsUnchanged,
        visualContractMatches,
        canPublish,
        transition
    };
}());
