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
        image: 'assets/flashcard-images/mat/m02.png'
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
    assert.equal(draft.version, 1);
});
