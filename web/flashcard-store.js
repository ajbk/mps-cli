window.MPS_FLASHCARD_STORE = function createFlashcardStore({
    apiBase = '',
    staticCards = [],
    storage = window.localStorage,
    fetchImpl = window.fetch.bind(window)
} = {}) {
    const request = async (path, options = {}) => {
        const response = await fetchImpl(`${apiBase}${path}`, options);
        if (!response.ok) throw new Error(`MPS API request failed: ${response.status}`);
        return response.json();
    };

    return {
        async listCards(filters = {}) {
            if (!apiBase) return window.MPS_FLASHCARD_MODEL.filterCards(staticCards, filters);
            const query = new URLSearchParams(filters).toString();
            return request(`/api/flashcards?${query}`);
        },
        async getCard(id) {
            if (!apiBase) return JSON.parse(storage.getItem(`mps.flashcard.${id}`) || 'null');
            return request(`/api/flashcards/${encodeURIComponent(id)}`);
        },
        async saveDraft(card) {
            if (!apiBase) {
                storage.setItem(`mps.flashcard.${card.id}`, JSON.stringify(card));
                return card;
            }
            return request(`/api/flashcards/${encodeURIComponent(card.id)}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(card)
            });
        }
    };
};
