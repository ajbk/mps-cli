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
