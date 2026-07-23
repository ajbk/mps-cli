(function attachFlashcardStore() {
    class MpsApiError extends Error {
        constructor(message, status, body) {
            super(message);
            this.name = 'MpsApiError';
            this.status = status;
            this.body = body;
        }
    }

    const COPY_FIELDS = ['front', 'cue', 'regress', 'progress', 'principle', 'objective'];
    const TERMINAL_JOB_STATUSES = new Set(['succeeded', 'failed']);

    function configuredApiBase() {
        return String(window.MPS_API_BASE || '').trim().replace(/\/$/, '');
    }

    function jsonObject(value) {
        return value && typeof value === 'object' && !Array.isArray(value) ? value : {};
    }

    function normalizeFindings(value) {
        if (!Array.isArray(value)) return [];
        return value.slice(0, 128).map((finding) => {
            if (typeof finding === 'string') return { message: finding };
            if (!finding || typeof finding !== 'object') return { message: String(finding ?? '') };
            return {
                code: typeof finding.code === 'string' ? finding.code : undefined,
                check: typeof finding.check === 'string' ? finding.check : undefined,
                severity: finding.severity === 'warning' ? 'warning' : 'error',
                message: typeof finding.message === 'string'
                    ? finding.message
                    : typeof finding.detail === 'string' ? finding.detail : ''
            };
        });
    }

    function escapeHtmlText(value) {
        return String(value ?? '').replace(/[&<>"']/g, (character) => ({
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#39;'
        }[character]));
    }

    function resolveAssetUrl(value, apiBase) {
        if (!value) return null;
        const url = String(value);
        if (/^https?:\/\//i.test(url) || !apiBase) return url;
        if (url.startsWith('/')) return `${String(apiBase).replace(/\/$/, '')}${url}`;
        try {
            return new URL(url, `${apiBase}/`).toString();
        } catch (error) {
            return null;
        }
    }

    function renderFindings(value) {
        const findings = normalizeFindings(value);
        if (!findings.length) return '<p class="muted">No validation findings returned yet.</p>';
        return '<ul class="review-findings" aria-label="Validation findings">' + findings.map((finding) => {
            const label = [finding.severity, finding.check || finding.code].filter(Boolean).join(' · ');
            return `<li><span class="finding-label">${escapeHtmlText(label || 'finding')}</span> ${escapeHtmlText(finding.message || 'Review finding')}</li>`;
        }).join('') + '</ul>';
    }

    function canPublish(card) {
        return card?.status === 'approved';
    }

    function normalizeCard(payload, apiBase = '') {
        if (!payload) return null;
        const card = payload.card || payload;
        const copy = jsonObject(card.teaching_copy_json || card.teachingCopy);
        const asset = jsonObject(card.asset || card.generated_asset);
        const review = card.automated_review || card.latest_automated_review || card.latest_review || {};
        const findings = normalizeFindings(
            card.review_findings || card.findings || card.findings_json || review.findings || review.findings_json
        );
        return {
            ...copy,
            ...card,
            teaching_copy_json: copy,
            automated_review: card.automated_review || (review.status ? {
                ...review,
                findings,
                findings_json: findings
            } : undefined),
            review_findings: findings,
            asset: Object.keys(asset).length ? {
                ...asset,
                url: resolveAssetUrl(asset.url || asset.image_url, apiBase)
            } : null,
            image: resolveAssetUrl(
                card.image || card.image_url || card.asset_url || asset.url || asset.image_url,
                apiBase
            )
        };
    }

    function normalizeJob(payload, apiBase = '') {
        if (!payload) return null;
        const job = payload.job || payload;
        const output = jsonObject(job.output_json || job.output);
        const asset = jsonObject(job.asset || output.asset || output.generated_asset);
        const review = job.automated_review || output.automated_review || output.review || {};
        return {
            ...job,
            output_json: output,
            asset: Object.keys(asset).length ? {
                ...asset,
                url: resolveAssetUrl(asset.url || asset.image_url, apiBase)
            } : null,
            automated_review: review.status ? {
                ...review,
                findings: normalizeFindings(review.findings || review.findings_json)
            } : null,
            review_findings: normalizeFindings(
                job.review_findings || job.findings || review.findings || review.findings_json || output.findings || output.review?.findings
            )
        };
    }

    function catalogIdentity(card) {
        return card?.source_exercise_id || card?.api_source_exercise_id || card?.id || '';
    }

    function sameCatalogCard(catalogCard, persistedCard) {
        const catalogId = catalogIdentity(catalogCard);
        const persistedId = catalogIdentity(persistedCard);
        if (catalogId && catalogId === persistedId) return true;
        if (catalogCard?.id && catalogCard.id === persistedCard?.id) return true;
        return String(catalogCard?.name || '').trim().toLowerCase() === String(persistedCard?.name || '').trim().toLowerCase()
            && String(catalogCard?.category || catalogCard?.apparatus || '').trim().toLowerCase()
                === String(persistedCard?.category || persistedCard?.apparatus || '').trim().toLowerCase();
    }

    function mergeCatalogCards(catalog, persisted, apiBase = '') {
        const staticCards = Array.isArray(catalog) ? catalog : [];
        const apiCards = Array.isArray(persisted) ? persisted : [];
        const matched = new Set();
        const merged = staticCards.map((catalogCard) => {
            const index = apiCards.findIndex((persistedCard, candidateIndex) => {
                return !matched.has(candidateIndex) && sameCatalogCard(catalogCard, persistedCard);
            });
            if (index < 0) {
                return {
                    ...catalogCard,
                    catalog_image: catalogCard.image || null,
                    source_exercise_id: catalogCard.source_exercise_id || catalogCard.api_source_exercise_id || null
                };
            }
            matched.add(index);
            const persistedCard = normalizeCard(apiCards[index], apiBase);
            return {
                ...catalogCard,
                ...persistedCard,
                catalog_image: catalogCard.image || null,
                source_exercise_id: persistedCard.source_exercise_id || catalogCard.source_exercise_id || catalogCard.id
            };
        });
        return merged.concat(apiCards.filter((_, index) => !matched.has(index)));
    }

    function teachingCopy(card) {
        const copy = { ...jsonObject(card.teaching_copy_json) };
        COPY_FIELDS.forEach((field) => {
            if (card[field] !== undefined) copy[field] = card[field];
        });
        return copy;
    }

    function responseBody(body, apiBase = '') {
        return body?.card ? normalizeCard(body.card, apiBase) : body;
    }

    window.MPS_FLASHCARD_STORE = function createFlashcardStore({
        apiBase = configuredApiBase(),
        staticCards = [],
        storage = window.localStorage,
        fetchImpl = window.fetch.bind(window),
        sleep = (milliseconds) => new Promise((resolve) => window.setTimeout(resolve, milliseconds))
    } = {}) {
        const base = String(apiBase || '').trim().replace(/\/$/, '');
        const isApiMode = Boolean(base);

        const request = async (path, options = {}) => {
            const response = await fetchImpl(`${base}${path}`, {
                credentials: 'include',
                ...options,
                headers: {
                    Accept: 'application/json',
                    ...(options.body ? { 'Content-Type': 'application/json' } : {}),
                    ...(options.headers || {})
                }
            });
            let body = null;
            try {
                body = await response.json();
            } catch (error) {
                if (response.status !== 204) body = null;
            }
            if (!response.ok) {
                const message = response.status === 401
                    ? 'MPS session is missing or expired. Sign in through the configured BFF before using the MPS API.'
                    : body?.error?.message || `MPS API request failed: ${response.status}`;
                throw new MpsApiError(
                    message,
                    response.status,
                    body
                );
            }
            return body;
        };

        async function listCards(filters = {}) {
            if (!isApiMode) return window.MPS_FLASHCARD_MODEL.filterCards(staticCards, filters);
            const query = new URLSearchParams(
                Object.entries(filters).filter(([, value]) => value !== undefined && value !== '' && value !== 'all')
            ).toString();
            const body = await request(query ? `/api/flashcards?${query}` : '/api/flashcards');
            return (Array.isArray(body) ? body : body?.cards || []).map((card) => normalizeCard(card, base));
        }

        async function getCard(id) {
            if (!isApiMode) return JSON.parse(storage.getItem(`mps.flashcard.${id}`) || 'null');
            try {
                return normalizeCard(await request(`/api/flashcards/${encodeURIComponent(id)}`), base);
            } catch (error) {
                if (error.status === 404) return null;
                throw error;
            }
        }

        async function saveDraft(card) {
            if (!isApiMode) {
                storage.setItem(`mps.flashcard.${card.id}`, JSON.stringify(card));
                return card;
            }
            return normalizeCard(await request(`/api/flashcards/${encodeURIComponent(card.id)}`, {
                method: 'PATCH',
                body: JSON.stringify({
                    category: card.category,
                    teaching_copy_json: teachingCopy(card)
                })
            }), base);
        }

        async function createDraft(source, fields = {}) {
            const sourceId = source?.source_exercise_id || source?.api_source_exercise_id || source?.id;
            if (!sourceId) throw new Error('A canonical source exercise is required');
            if (!isApiMode) {
                const draft = {
                    ...window.MPS_FLASHCARD_MODEL.createLocalDraft(source),
                    ...fields
                };
                storage.setItem(`mps.flashcard.${draft.id}`, JSON.stringify(draft));
                return draft;
            }
            return normalizeCard(await request('/api/flashcards', {
                method: 'POST',
                body: JSON.stringify({
                    source_exercise_id: sourceId,
                    category: fields.category || source.category,
                    teaching_copy_json: teachingCopy({ ...source, ...fields })
                })
            }), base);
        }

        async function createVisualBrief(cardId, brief) {
            if (!isApiMode) return { id: 'demo-brief', card_id: cardId, ...brief };
            return responseBody(await request(`/api/flashcards/${encodeURIComponent(cardId)}/visual-briefs`, {
                method: 'POST',
                body: JSON.stringify(brief)
            }), base);
        }

        async function createJob(cardId, { kind = 'generate', briefId, revisionNotes } = {}) {
            if (!isApiMode) throw new Error('Jobs require MPS_API_BASE');
            return normalizeJob(await request(`/api/flashcards/${encodeURIComponent(cardId)}/jobs`, {
                method: 'POST',
                body: JSON.stringify({
                    kind,
                    ...(briefId ? { brief_id: briefId } : {}),
                    ...(revisionNotes ? { revision_notes: revisionNotes } : {})
                })
            }), base);
        }

        async function getJob(cardId, jobId) {
            if (!isApiMode) return null;
            return normalizeJob(await request(
                `/api/flashcards/${encodeURIComponent(cardId)}/jobs/${encodeURIComponent(jobId)}`
            ), base);
        }

        async function pollJob(cardId, jobId, { intervalMs = 1500, maxAttempts = 40 } = {}) {
            if (!isApiMode) return null;
            let job = null;
            for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
                job = await getJob(cardId, jobId);
                if (!job || TERMINAL_JOB_STATUSES.has(job.status)) return job;
                if (attempt + 1 < maxAttempts) await sleep(intervalMs);
            }
            throw new Error('Generation is still running; refresh this card to continue polling');
        }

        async function submitReview(cardId, card) {
            if (!isApiMode) throw new Error('Submit review is only available in API mode');
            if (card?.status === 'generating') {
                throw new Error('Wait for the visual worker to finish before submitting review');
            }
            return responseBody(await request(`/api/flashcards/${encodeURIComponent(cardId)}/submit-review`, {
                method: 'POST'
            }), base);
        }

        function canSubmitReview(card, apiMode = isApiMode) {
            return !apiMode && card?.status === 'generating';
        }

        async function approve(cardId, { reviewId, findings = [] } = {}) {
            if (!isApiMode) throw new Error('Approval is only available in API mode');
            return responseBody(await request(`/api/flashcards/${encodeURIComponent(cardId)}/approve`, {
                method: 'POST',
                body: JSON.stringify({
                    review_id: reviewId || `teacher-review-${Date.now()}`,
                    findings_json: normalizeFindings(findings)
                })
            }), base);
        }

        async function publish(cardId) {
            if (!isApiMode) throw new Error('Publishing requires the MPS API');
            return responseBody(await request(`/api/flashcards/${encodeURIComponent(cardId)}/publish`, {
                method: 'POST'
            }), base);
        }

        return {
            apiBase: base,
            isApiMode,
            sessionHandoffUrl: String(window.MPS_SESSION_HANDOFF_URL || '').trim(),
            listCards,
            getCard,
            saveDraft,
            createDraft,
            createVisualBrief,
            createJob,
            getJob,
            pollJob,
            submitReview,
            approve,
            publish,
            normalizeCard: (payload) => normalizeCard(payload, base),
            normalizeJob: (payload) => normalizeJob(payload, base),
            normalizeFindings,
            renderFindings,
            canPublish,
            canSubmitReview,
            mergeCatalogCards: (catalog, persisted) => mergeCatalogCards(catalog, persisted, base)
        };
    };
}());
