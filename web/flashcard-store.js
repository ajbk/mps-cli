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

    function normalizeCard(payload) {
        if (!payload) return null;
        const card = payload.card || payload;
        const copy = jsonObject(card.teaching_copy_json || card.teachingCopy);
        const review = card.automated_review || card.latest_review || {};
        const findings = normalizeFindings(
            card.review_findings || card.findings || card.findings_json || review.findings
        );
        return {
            ...copy,
            ...card,
            teaching_copy_json: copy,
            automated_review: card.automated_review || (review.status ? {
                ...review,
                findings
            } : undefined),
            review_findings: findings,
            image: card.image || card.image_url || card.asset_url || null
        };
    }

    function normalizeJob(payload) {
        if (!payload) return null;
        const job = payload.job || payload;
        const output = jsonObject(job.output_json || job.output);
        return {
            ...job,
            output_json: output,
            review_findings: normalizeFindings(
                job.review_findings || job.findings || output.findings || output.review?.findings
            )
        };
    }

    function teachingCopy(card) {
        const copy = { ...jsonObject(card.teaching_copy_json) };
        COPY_FIELDS.forEach((field) => {
            if (card[field] !== undefined) copy[field] = card[field];
        });
        return copy;
    }

    function responseBody(body) {
        return body?.card ? normalizeCard(body.card) : body;
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
                throw new MpsApiError(
                    body?.error?.message || `MPS API request failed: ${response.status}`,
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
            return (Array.isArray(body) ? body : body?.cards || []).map(normalizeCard);
        }

        async function getCard(id) {
            if (!isApiMode) return JSON.parse(storage.getItem(`mps.flashcard.${id}`) || 'null');
            try {
                return normalizeCard(await request(`/api/flashcards/${encodeURIComponent(id)}`));
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
            }));
        }

        async function createDraft(source, fields = {}) {
            const sourceId = source?.source_exercise_id || source?.id;
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
            }));
        }

        async function createVisualBrief(cardId, brief) {
            if (!isApiMode) return { id: 'demo-brief', card_id: cardId, ...brief };
            return responseBody(await request(`/api/flashcards/${encodeURIComponent(cardId)}/visual-briefs`, {
                method: 'POST',
                body: JSON.stringify(brief)
            }));
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
            }));
        }

        async function getJob(cardId, jobId) {
            if (!isApiMode) return null;
            return normalizeJob(await request(
                `/api/flashcards/${encodeURIComponent(cardId)}/jobs/${encodeURIComponent(jobId)}`
            ));
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

        async function submitReview(cardId) {
            if (!isApiMode) throw new Error('Submit review is only available in API mode');
            return responseBody(await request(`/api/flashcards/${encodeURIComponent(cardId)}/submit-review`, {
                method: 'POST'
            }));
        }

        async function approve(cardId, { reviewId, findings = [] } = {}) {
            if (!isApiMode) throw new Error('Approval is only available in API mode');
            return responseBody(await request(`/api/flashcards/${encodeURIComponent(cardId)}/approve`, {
                method: 'POST',
                body: JSON.stringify({
                    review_id: reviewId || `teacher-review-${Date.now()}`,
                    findings_json: normalizeFindings(findings)
                })
            }));
        }

        async function publish(cardId) {
            if (!isApiMode) throw new Error('Publishing requires the MPS API');
            return responseBody(await request(`/api/flashcards/${encodeURIComponent(cardId)}/publish`, {
                method: 'POST'
            }));
        }

        return {
            apiBase: base,
            isApiMode,
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
            normalizeCard,
            normalizeJob,
            normalizeFindings,
            renderFindings,
            canPublish
        };
    };
}());
