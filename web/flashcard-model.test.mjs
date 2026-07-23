import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import vm from 'node:vm';

async function loadModel() {
    const source = await readFile(new URL('./flashcard-model.js', import.meta.url), 'utf8');
    const window = {};
    vm.runInNewContext(source, { window });
    return window.MPS_FLASHCARD_MODEL;
}

async function loadStaticCards() {
    const source = await readFile(new URL('./flashcards-data.js', import.meta.url), 'utf8');
    const window = {};
    vm.runInNewContext(source, { window });
    return window.MPS_FLASHCARDS;
}

async function loadStore({ apiBase = '', responses = [], storage = new Map() } = {}) {
    const model = await loadModel();
    const source = await readFile(new URL('./flashcard-store.js', import.meta.url), 'utf8');
    const calls = [];
    const responseQueue = [...responses];
    const localStorage = {
        getItem: (key) => storage.get(key) || null,
        setItem: (key, value) => storage.set(key, value),
        removeItem: (key) => storage.delete(key)
    };
    const window = {
        MPS_API_BASE: apiBase,
        MPS_FLASHCARD_MODEL: model,
        localStorage,
        fetch: async (url, options) => {
            calls.push({ url, options });
            const next = responseQueue.shift();
            if (!next) throw new Error(`Unexpected request: ${url}`);
            return {
                ok: next.ok !== false,
                status: next.status || (next.ok === false ? 400 : 200),
                json: async () => next.body
            };
        },
        setTimeout
    };
    vm.runInNewContext(source, { window, setTimeout, URLSearchParams });
    return { store: window.MPS_FLASHCARD_STORE({ sleep: async () => {} }), calls, storage };
}

const cards = [
    { id: 'R02', category: 'Reformer', level: 'Intermediate', name: 'Long Stretch', search: 'long stretch shoulder stability' },
    { id: 'M01', category: 'Mat', level: 'Beginner', name: 'Pelvic Clock', search: 'pelvic clock mobility' },
    { id: 'R01', category: 'Reformer', level: 'Beginner', name: 'Footwork', search: 'footwork knee tracking' }
];

test('filterCards filters by category and search query', async () => {
    const model = await loadModel();

    assert.deepEqual(
        model.filterCards(cards, { category: 'Reformer', query: 'shoulder' }).map((card) => card.id),
        ['R02']
    );
});

test('filterCards filters by level', async () => {
    const model = await loadModel();

    assert.deepEqual(
        model.filterCards(cards, { level: 'Beginner' }).map((card) => card.id),
        ['M01', 'R01']
    );
});

test('filterCards sorts matching cards by stable ID', async () => {
    const model = await loadModel();

    assert.deepEqual(model.filterCards(cards, {}).map((card) => card.id), ['M01', 'R01', 'R02']);
});

test('statusLabel renders the draft status', async () => {
    const model = await loadModel();

    assert.equal(model.statusLabel('draft'), 'Draft');
});

test('createLocalDraft preserves source exercise fields and creates a draft', async () => {
    const model = await loadModel();
    const sourceCard = {
        id: 'M02',
        category: 'Mat',
        name: 'Pelvic Clock',
        front: 'What does it assess?',
        level: 'Beginner',
        objective: 'Pelvic awareness',
        image: 'assets/flashcard-images/mat/m02.png',
        style_profile: 'unapproved-style',
        character_id: 'unapproved-character',
        outfit: 'unapproved-outfit',
        cheek_accent: '#000000'
    };

    const draft = model.createLocalDraft(sourceCard);

    assert.equal(draft.id, model.draftKey('M02'));
    assert.equal(draft.source_exercise_id, 'M02');
    assert.equal(draft.category, 'Mat');
    assert.equal(draft.name, 'Pelvic Clock');
    assert.equal(draft.front, 'What does it assess?');
    assert.equal(draft.status, 'draft');
    assert.equal(draft.style_profile, 'mono-gesture-ink-pilates-v1');
    assert.equal(draft.character_id, 'teacher-01');
    assert.equal(draft.outfit, 'off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts');
    assert.equal(draft.cheek_accent, '#D98F9A');
    assert.equal(draft.version, 1);
});

test('library demo catalog contains the complete static card set', async () => {
    const catalog = await loadStaticCards();

    assert.equal(catalog.length, 160);
});

test('library catalog filters to Mat cards', async () => {
    const model = await loadModel();
    const catalog = await loadStaticCards();

    assert.ok(model.filterCards(catalog, { category: 'Mat' }).every((card) => card.category === 'Mat'));
});

test('library catalog finds M02 for pelvic clock search', async () => {
    const model = await loadModel();
    const catalog = await loadStaticCards();

    assert.ok(model.filterCards(catalog, { query: 'pelvic clock' }).some((card) => card.id === 'M02'));
});

test('library catalog image URLs are relative public paths', async () => {
    const catalog = await loadStaticCards();

    assert.ok(catalog.every((card) => !card.image || (!card.image.startsWith('/') && !card.image.includes('/Users/'))));
});

test('API mode lists cards through GET /api/flashcards and normalizes teaching copy', async () => {
    const { store, calls } = await loadStore({
        apiBase: 'https://mps.test/',
        responses: [{ body: [{
            id: 'card-1',
            source_exercise_id: 'M02',
            name: 'Pelvic Clock',
            category: 'Mat',
            status: 'draft',
            teaching_copy_json: { front: 'What does it assess?' }
        }] }]
    });

    const cards = await store.listCards();

    assert.equal(calls[0].url, 'https://mps.test/api/flashcards');
    assert.equal(cards[0].front, 'What does it assess?');
    assert.equal(cards[0].teaching_copy_json.front, 'What does it assess?');
});

test('missing API base keeps local demo drafts in browser storage', async () => {
    const storage = new Map();
    const { store } = await loadStore({ storage });
    const source = { id: 'M02', category: 'Mat', name: 'Pelvic Clock', level: 'Beginner' };

    const draft = await store.createDraft(source);
    const reopened = await store.getCard(draft.id);

    assert.equal(draft.id, 'draft:M02');
    assert.equal(reopened.name, 'Pelvic Clock');
    assert.equal(storage.has('mps.flashcard.draft:M02'), true);
});

test('API generation creates a job and polls it until terminal status', async () => {
    const { store, calls } = await loadStore({
        apiBase: 'https://mps.test',
        responses: [
            { status: 202, body: { id: 'job-1', card_id: 'card-1', kind: 'generate', status: 'queued' } },
            { body: { id: 'job-1', card_id: 'card-1', kind: 'generate', status: 'running' } },
            { body: { id: 'job-1', card_id: 'card-1', kind: 'generate', status: 'succeeded', output_json: { findings: [{ code: 'visual_check_failed', check: 'pose' }] } } }
        ]
    });

    const created = await store.createJob('card-1', { briefId: 'brief-1' });
    const completed = await store.pollJob('card-1', created.id, { intervalMs: 0, maxAttempts: 3 });

    assert.equal(calls[0].options.method, 'POST');
    assert.match(calls[0].options.body, /"brief_id":"brief-1"/);
    assert.equal(completed.status, 'succeeded');
    assert.equal(completed.review_findings[0].check, 'pose');
});

test('review findings are escaped before they become HTML', async () => {
    const { store } = await loadStore();
    const html = store.renderFindings([{ severity: 'error', code: 'vision_finding', message: '<img src=x onerror=alert(1)>' }]);

    assert.doesNotMatch(html, /<img/i);
    assert.match(html, /&lt;img/i);
    assert.match(html, /onerror=alert\(1\)/i);
});

test('API publish remains unavailable until the card is approved', async () => {
    const { store } = await loadStore({ apiBase: 'https://mps.test' });

    assert.equal(store.canPublish({ status: 'needs-review' }), false);
    assert.equal(store.canPublish({ status: 'approved' }), true);
});
