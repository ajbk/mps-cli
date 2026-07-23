(function attachFlashcardModel() {
    const statusLabels = {
        draft: 'Draft',
        generating: 'Generating',
        'needs-review': 'Needs review',
        'revision-requested': 'Revision requested',
        approved: 'Approved',
        published: 'Published'
    };

    function filterCards(cards, filters) {
        const source = Array.isArray(cards) ? cards : [];
        const category = filters.category || '';
        const level = filters.level || '';
        const query = String(filters.query || '').trim().toLowerCase();

        return source
            .filter((card) => !category || card.category === category)
            .filter((card) => !level || level === 'all' || card.level === level)
            .filter((card) => !query || searchableText(card).includes(query))
            .slice()
            .sort((left, right) => String(left.id).localeCompare(String(right.id)));
    }

    function searchableText(card) {
        return String(card.search || [
            card.id,
            card.category,
            card.name,
            card.front,
            card.level,
            card.objective,
            card.principle,
            card.cue,
            card.regress,
            card.progress
        ].filter(Boolean).join(' ')).toLowerCase();
    }

    function statusLabel(status) {
        return statusLabels[status] || String(status || 'Unknown');
    }

    function draftKey(cardId) {
        return `draft:${cardId}`;
    }

    function createLocalDraft(card) {
        return {
            ...card,
            id: draftKey(card.id),
            source_exercise_id: card.source_exercise_id || card.id,
            style_profile: card.style_profile || 'mono-gesture-ink-pilates-v1',
            character_id: card.character_id || 'teacher-01',
            current_asset_id: card.current_asset_id || null,
            version: card.version || 1,
            status: 'draft'
        };
    }

    window.MPS_FLASHCARD_MODEL = {
        filterCards,
        statusLabel,
        draftKey,
        createLocalDraft
    };
}());
