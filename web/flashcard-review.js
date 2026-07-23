(function attachFlashcardReview() {
    const visualContract = {
        style_profile: 'mono-gesture-ink-pilates-v1',
        character_id: 'teacher-01',
        outfit: 'off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts',
        cheek_accent: '#D98F9A'
    };
    const lockedSourceFields = [
        ['exercise_id', 'source_exercise_id'],
        ['style_profile', 'style_profile'],
        ['name', 'name'],
        ['category', 'category'],
        ['apparatus', 'apparatus'],
        ['level', 'level'],
        ['objective', 'objective']
    ];

    function automatedReviewPassed(card) {
        return card?.automated_review?.status === 'passed'
            && card.automated_review.version === card.version;
    }

    function sourceIsUnchanged(card, canonicalSource) {
        const snapshot = card?.source_snapshot;
        return Boolean(snapshot && canonicalSource)
            && lockedSourceFields.every(([sourceField, cardField]) =>
                snapshot[sourceField] === canonicalSource[sourceField]
                && card?.[cardField] === canonicalSource[sourceField]
            );
    }

    function matchesTrustedDraft(card, trustedDraftId) {
        return Boolean(trustedDraftId) && card?.id === trustedDraftId;
    }

    function hydrateLegacySource(card, canonicalSource) {
        if (!card || !canonicalSource) return null;
        const snapshot = card.source_snapshot || {};
        const canHydrate = lockedSourceFields.every(([sourceField, cardField]) => {
            const cardValue = card[cardField];
            const snapshotValue = snapshot[sourceField];
            const allowsMissingApparatus = sourceField === 'apparatus' && cardValue === undefined;
            return (allowsMissingApparatus || cardValue === canonicalSource[sourceField])
                && (snapshotValue === undefined || snapshotValue === canonicalSource[sourceField]);
        });
        const needsHydration = card.apparatus === undefined
            || lockedSourceFields.some(([sourceField]) => snapshot[sourceField] === undefined);
        if (!canHydrate || !needsHydration) return null;
        return {
            ...card,
            apparatus: card.apparatus === undefined ? canonicalSource.apparatus : card.apparatus,
            source_snapshot: { ...canonicalSource }
        };
    }

    function hydrateTrustedDraft(card, trustedDraftId, canonicalSource) {
        if (!matchesTrustedDraft(card, trustedDraftId)) return null;
        return hydrateLegacySource(card, canonicalSource);
    }

    function invalidateTeacherEdits(card) {
        const version = (Number(card?.version) || 0) + 1;
        return {
            ...card,
            status: 'draft',
            version,
            image: null,
            current_asset_id: null,
            asset_version: null,
            automated_review: null,
            teacher_review: null,
            proposed_changes: null
        };
    }

    function visualContractMatches(card) {
        return Object.entries(visualContract).every(([key, value]) => card?.[key] === value);
    }

    function canPublish(card, canonicalSource) {
        return card?.status === 'approved'
            && automatedReviewPassed(card)
            && card.teacher_review?.status === 'approved'
            && card.teacher_review.version === card.version
            && sourceIsUnchanged(card, canonicalSource)
            && visualContractMatches(card);
    }

    function transition(card, action, details = {}) {
        if (!card) throw new Error('A flashcard is required');

        if (action === 'generate') {
            if (!['draft', 'revision-requested'].includes(card.status)) {
                throw new Error('Cannot generate this flashcard');
            }
            const version = (Number(card.version) || 0) + 1;
            return {
                ...card,
                status: 'generating',
                version,
                image: null,
                current_asset_id: null,
                asset_version: null,
                automated_review: { status: 'pending', version },
                teacher_review: null,
                proposed_changes: null
            };
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
            if (!sourceIsUnchanged(card, details.canonicalSource)) {
                throw new Error('Canonical source must match before approval');
            }
            if (!visualContractMatches(card)) throw new Error('Visual contract must match before approval');
            return {
                ...card,
                status: 'approved',
                teacher_review: {
                    reviewer: details.reviewer || 'Teacher',
                    status: 'approved',
                    version: card.version
                }
            };
        }

        if (action === 'publish') {
            if (!canPublish(card, details.canonicalSource)) throw new Error('Cannot publish this flashcard until every review guard passes');
            return { ...card, status: 'published' };
        }

        throw new Error(`Unknown flashcard action: ${action}`);
    }

    window.MPS_FLASHCARD_REVIEW = {
        visualContract,
        automatedReviewPassed,
        sourceIsUnchanged,
        matchesTrustedDraft,
        hydrateLegacySource,
        hydrateTrustedDraft,
        invalidateTeacherEdits,
        visualContractMatches,
        canPublish,
        transition
    };
}());
