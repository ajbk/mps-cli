const state = {
    plan: null,
    markdown: '',
    view: 'plan'
};

const presetDefaults = {
    ShoulderFreedom: {
        observations: 'thoracic stiffness, limited overhead reach, desk workers',
        level: 'BeginnerIntermediate',
        duration: '60',
        contraindications: ['wrist_pain']
    },
    HappyHips: {
        observations: 'hip tightness, limited squat depth, knee tracking needs support',
        level: 'BeginnerIntermediate',
        duration: '45',
        contraindications: ['knee_pain']
    },
    SpineReset: {
        observations: 'low back tension, limited rotation, weak core connection',
        level: 'Intermediate',
        duration: '75',
        contraindications: ['low_back_pain']
    }
};

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => Array.from(document.querySelectorAll(selector));

function escapeHtml(value) {
    const node = document.createElement('div');
    node.textContent = String(value ?? '');
    return node.innerHTML;
}

function selectedValues(containerSelector) {
    return $$(`${containerSelector} input[type="checkbox"]:checked`).map((input) => input.value);
}

function commaValues(raw) {
    return raw
        .split(',')
        .map((value) => value.trim())
        .filter(Boolean);
}

function syncToggleLabels() {
    $$('.toggle-chip').forEach((chip) => {
        const input = chip.querySelector('input');
        chip.classList.toggle('active', input.checked);
    });
}

function setCheckedValues(containerSelector, values) {
    $$(`${containerSelector} input[type="checkbox"]`).forEach((input) => {
        input.checked = values.includes(input.value);
    });
    syncToggleLabels();
}

function setPreset(experience) {
    const preset = presetDefaults[experience];
    $('#experience').value = experience;
    $('#observations').value = preset.observations;
    $('#level').value = preset.level;
    $('#duration').value = preset.duration;
    setCheckedValues('#contraindications', preset.contraindications);

    $$('.preset-button').forEach((button) => {
        button.classList.toggle('active', button.dataset.preset === experience);
    });

    generatePlan();
}

function buildRequest() {
    const equipment = selectedValues('#equipment');
    if (equipment.length === 0) {
        throw new Error('Choose at least one primary apparatus.');
    }

    const selectedContra = selectedValues('#contraindications');
    const extraContra = commaValues($('#contra-extra').value);

    return {
        students: Number.parseInt($('#students').value, 10) || 1,
        movement_experience: $('#experience').value,
        level: $('#level').value,
        equipment,
        duration_minutes: Number.parseInt($('#duration').value, 10),
        observations: commaValues($('#observations').value),
        group_safety: {
            contraindications: [...selectedContra, ...extraContra],
            risk_policy: $('#risk-policy').value
        }
    };
}

function generatePlan() {
    $('#error-message').textContent = '';

    try {
        const request = buildRequest();
        state.plan = generateClassPlan(request);
        state.markdown = renderMarkdown(state.plan);
        renderAll();
    } catch (error) {
        $('#error-message').textContent = error.message;
    }
}

function renderAll() {
    if (!state.plan) return;
    renderSummary();
    renderStrategy();
    renderCurrentView();
}

function renderSummary() {
    const plan = state.plan;
    const exerciseCount = plan.journey.reduce((sum, phase) => sum + phase.exercises.length, 0);
    const equipment = plan.equipment.join(', ');

    $('#summary-band').innerHTML = `
        <div class="summary-title">
            <h1>${escapeHtml(plan.class_title)}</h1>
            <p>${escapeHtml(equipment)} apparatus with Mat/Standing context phases</p>
        </div>
        <div class="metric"><strong>${plan.duration_minutes}</strong><span>minutes</span></div>
        <div class="metric"><strong>${plan.students}</strong><span>students</span></div>
        <div class="metric"><strong>${plan.journey.length}</strong><span>phases</span></div>
        <div class="metric"><strong>${exerciseCount}</strong><span>exercises</span></div>
    `;
}

function renderStrategy() {
    const strategy = state.plan.movement_strategy;
    const bars = Object.entries(strategy.emphasis)
        .map(([system, value]) => `
            <div class="bar-row">
                <span>${escapeHtml(system)}</span>
                <div class="bar-track"><div class="bar-fill" style="width:${Number(value)}%"></div></div>
                <strong>${Number(value)}%</strong>
            </div>
        `)
        .join('');

    $('#strategy-band').innerHTML = `
        <div class="strategy-copy">
            <h2>Strategy</h2>
            <p><strong>${escapeHtml(strategy.primary_focus)}</strong> primary focus with <strong>${escapeHtml(strategy.secondary_focus)}</strong> support.</p>
        </div>
        <div class="bar-list">${bars}</div>
    `;
}

function renderCurrentView() {
    const target = $('#content-view');
    if (state.view === 'markdown') {
        target.innerHTML = `<pre class="code-view">${escapeHtml(state.markdown)}</pre>`;
        return;
    }
    if (state.view === 'json') {
        target.innerHTML = `<pre class="code-view">${escapeHtml(JSON.stringify(state.plan, null, 2))}</pre>`;
        return;
    }
    target.innerHTML = renderPlanView();
}

function renderPlanView() {
    const phaseBlocks = state.plan.journey
        .map((phase, index) => {
            const rows = phase.exercises.length
                ? phase.exercises.map(renderExerciseRow).join('')
                : '<div class="exercise-row"><div class="exercise-meta">No exercise selected for this phase.</div></div>';

            return `
                <article class="phase-block">
                    <header class="phase-head">
                        <span class="phase-index">${index + 1}</span>
                        <div>
                            <h3>${escapeHtml(phase.phase)}</h3>
                            <p>${escapeHtml(phase.purpose)}</p>
                        </div>
                        <span class="phase-time">${phase.target_duration_minutes} min</span>
                    </header>
                    <div class="exercise-table">${rows}</div>
                </article>
            `;
        })
        .join('');

    return `
        <div class="phase-list">${phaseBlocks}</div>
        ${renderBenchmarks()}
        ${renderSafety()}
    `;
}

function renderExerciseRow(exercise) {
    const cue = exercise.teaching_cues.slice(0, 2).join(' ');
    const safety = exercise.safety_notes
        .map((note) => `<span class="tag warning-tag">${escapeHtml(note)}</span>`)
        .join(' ');

    return `
        <div class="exercise-row">
            <div>
                <div class="exercise-name">${escapeHtml(exercise.name)}</div>
                <div class="exercise-meta">${escapeHtml(exercise.exercise_id)}</div>
            </div>
            <div><span class="tag">${escapeHtml(exercise.apparatus)}</span></div>
            <div class="exercise-meta">${exercise.duration_minutes} min</div>
            <div class="exercise-cues">${escapeHtml(cue)} ${safety}</div>
        </div>
    `;
}

function renderBenchmarks() {
    const items = state.plan.benchmark.assessments
        .map((assessment) => `
            <article class="benchmark-item">
                <h3>${escapeHtml(assessment.name)}</h3>
                <p>${escapeHtml(assessment.instruction)}</p>
                <p>${assessment.what_to_watch.map(escapeHtml).join(' / ')}</p>
            </article>
        `)
        .join('');

    return `
        <section class="benchmark-grid" aria-label="Benchmarks">
            ${items}
        </section>
    `;
}

function renderSafety() {
    const summary = state.plan.safety_summary;
    const excluded = summary.excluded_exercises.length
        ? summary.excluded_exercises.map((item) => `
            <article class="safety-item">
                <h3>${escapeHtml(item.name)}</h3>
                <p>${escapeHtml(item.reason)}</p>
            </article>
        `).join('')
        : '<article class="safety-item"><h3>No hard exclusions</h3><p>The current request did not trigger hard-excluded exercises.</p></article>';

    return `
        <section class="benchmark-grid" aria-label="Safety summary">
            <article class="safety-item">
                <h3>Risk policy</h3>
                <p>${escapeHtml(summary.risk_policy)}</p>
                <p>${summary.applied_contraindications.map(escapeHtml).join(', ') || 'No group contraindications.'}</p>
            </article>
            ${excluded}
        </section>
    `;
}

function showToast(message) {
    const toast = document.createElement('div');
    toast.className = 'toast';
    toast.textContent = message;
    document.body.appendChild(toast);
    window.setTimeout(() => toast.remove(), 1800);
}

async function copyMarkdown() {
    if (!state.markdown) return;
    await navigator.clipboard.writeText(state.markdown);
    showToast('Markdown copied');
}

function downloadJson() {
    if (!state.plan) return;
    const blob = new Blob([JSON.stringify(state.plan, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${state.plan.movement_experience}-${state.plan.duration_minutes}.json`;
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
}

function attachEvents() {
    $('#generate-plan').addEventListener('click', generatePlan);
    $('#copy-markdown').addEventListener('click', copyMarkdown);
    $('#download-json').addEventListener('click', downloadJson);
    $('#reset-request').addEventListener('click', () => setPreset('ShoulderFreedom'));

    $('#experience').addEventListener('change', (event) => {
        setPreset(event.target.value);
    });

    $$('.preset-button').forEach((button) => {
        button.addEventListener('click', () => setPreset(button.dataset.preset));
    });

    $$('.toggle-chip').forEach((chip) => {
        chip.addEventListener('click', () => {
            window.setTimeout(() => {
                syncToggleLabels();
                generatePlan();
            }, 0);
        });
    });

    ['duration', 'students', 'level', 'risk-policy', 'contra-extra', 'observations'].forEach((id) => {
        $(`#${id}`).addEventListener('change', generatePlan);
    });

    $$('.tab-button').forEach((button) => {
        button.addEventListener('click', () => {
            state.view = button.dataset.view;
            $$('.tab-button').forEach((tab) => tab.classList.toggle('active', tab === button));
            renderCurrentView();
        });
    });
}

attachEvents();
syncToggleLabels();
generatePlan();
