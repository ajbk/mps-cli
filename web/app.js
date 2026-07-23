const STORAGE_KEY = 'mpsClientStudio.v2';

const archetypes = [
    'Office Syndrome',
    'Strong but Tight',
    'Weak Core / Cadillac Curious',
    'Athlete - Runner',
    'Athlete - Tennis',
    'Low Back / QL Dominant',
    'Beginner General Fitness',
    'Recovery / Relaxation'
];

const redFlags = [
    'Radiating arm/leg pain',
    'Numbness',
    'Weakness',
    'Dizziness',
    'Acute pain',
    'Post-surgery',
    'Pregnancy',
    'Osteoporosis',
    'Disc issue',
    'Joint replacement',
    'Blood pressure / heart'
];

const playbooks = [
    {
        name: 'Office Syndrome - Scapular Stability x Thoracic Mobility',
        archetypes: ['Office Syndrome'],
        movementExperience: 'ShoulderFreedom',
        tests: 'Arm Raise, Cervical Rotation, Standing Roll Down',
        exercises: 'Breathing, Cat Cow, Supine Arm Springs, Rowing Prep, Mermaid, Swan Prep, Chest Expansion',
        cautions: 'Watch neck gripping, shoulder elevation, loaded wrist extension, and breath holding.',
        experience: 'Neck feels longer, shoulders feel wider, and arm lift improves without rib flare.',
        observations: ['desk work', 'thoracic stiffness', 'limited overhead reach', 'shoulder elevation']
    },
    {
        name: 'Strong but Tight - Mobility with Strength',
        archetypes: ['Strong but Tight'],
        movementExperience: 'HappyHips',
        tests: 'Standing Roll Down, Hip Hinge, Thoracic Rotation, Hamstring Check',
        exercises: 'Footwork, Pelvic Curl, Feet in Straps, Elephant, Mermaid, Side Split, Barrel Stretch',
        cautions: 'Do not chase range before control; keep pelvis and ribs organized before increasing load.',
        experience: 'More range without losing control; strength work feels smoother instead of compressed.',
        observations: ['strong but tight', 'limited hip hinge', 'hamstring tension', 'needs controlled range']
    },
    {
        name: 'Cadillac Prep - Core Control Before Challenge',
        archetypes: ['Weak Core / Cadillac Curious'],
        movementExperience: 'SpineReset',
        tests: 'Toe Tap, Arm Raise, Plank Prep, Roll Back Control',
        exercises: 'Breathing, Toe Tap, Roll Back, Supine Arm Springs, Tower Prep, Push Through Bar Prep',
        cautions: 'Delay hanging or inversion if rib flare, breath holding, or shoulder elevation is visible.',
        experience: 'Student feels stronger and more organized before entering challenge work.',
        observations: ['weak core connection', 'rib flare', 'breath holding', 'cadillac preparation']
    },
    {
        name: 'Runner - Hip Stability x Single Leg Control',
        archetypes: ['Athlete - Runner'],
        movementExperience: 'HappyHips',
        tests: 'Single Leg Balance, Step Down, Hip Hinge, Calf Flexibility',
        exercises: 'Footwork, Single Leg Footwork, Bridging, Scooter, Side Lying Leg Work, Eve Lunge, Skater',
        cautions: 'Watch pelvis drop, knee valgus, ankle limitation, and hip flexor overuse.',
        experience: 'Single-leg stance feels steadier and running gait feels lighter.',
        observations: ['single leg control', 'hip stability', 'knee tracking', 'running load']
    },
    {
        name: 'Tennis - Rotation Power x Shoulder Stability',
        archetypes: ['Athlete - Tennis'],
        movementExperience: 'ShoulderFreedom',
        tests: 'Thoracic Rotation, Arm Raise, Lunge with Rotation, Single Leg Balance',
        exercises: 'Mermaid, Saw, Spine Twist, Standing Arm Springs, Rowing, Pulling Straps, Scooter',
        cautions: 'Prevent arm-dominant rotation and monitor wrist, elbow, and shoulder overuse.',
        experience: 'Rotation feels connected from legs to trunk to arm.',
        observations: ['thoracic rotation', 'shoulder stability', 'sport rotation', 'overhead reach']
    },
    {
        name: 'Low Back / QL - Rib Pelvis Hip Connection',
        archetypes: ['Low Back / QL Dominant'],
        movementExperience: 'SpineReset',
        tests: 'Standing Roll Down, Side Bend, Hip Hinge, Toe Tap, Single Leg Balance',
        exercises: 'Breathing, Pelvic Clock, Knee Sway, Cat Cow, Pelvic Curl, Feet in Straps, Scooter, Mermaid',
        cautions: 'Avoid heavy plank, high Swan, or Teaser if lumbar gripping is dominant.',
        experience: 'Low back feels lighter and hip/trunk movement improves without compression.',
        observations: ['low back tension', 'QL dominance', 'lumbar gripping', 'limited rotation']
    }
];

const experienceLabels = {
    ShoulderFreedom: 'Shoulder Freedom',
    HappyHips: 'Happy Hips',
    SpineReset: 'Spine Reset'
};

const flashcardCategories = ['Reformer', 'Mat', 'Stand', 'Chair'];

const flashcardLevelRank = {
    Beginner: 1,
    'Beginner Intermediate': 2,
    Intermediate: 3,
    'Intermediate Advanced': 4,
    Advanced: 5,
    All: 6
};

const state = {
    activeView: 'dashboard',
    selectedStudentId: null,
    planView: 'plan',
    flashcardCategory: 'Reformer',
    flashcardQuery: '',
    flashcardLevel: 'all',
    plan: null,
    markdown: '',
    data: loadData()
};

state.selectedStudentId = state.data.selectedStudentId || state.data.students[0]?.id || null;

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => Array.from(document.querySelectorAll(selector));

function loadData() {
    try {
        const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}');
        return normalizeData(parsed);
    } catch (error) {
        return normalizeData({});
    }
}

function normalizeData(raw) {
    return {
        students: Array.isArray(raw.students) ? raw.students : [],
        assessments: Array.isArray(raw.assessments) ? raw.assessments : [],
        sessions: Array.isArray(raw.sessions) ? raw.sessions : [],
        selectedStudentId: raw.selectedStudentId || null
    };
}

function saveData() {
    state.data.selectedStudentId = state.selectedStudentId;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state.data));
}

function uid() {
    return `id_${Math.random().toString(36).slice(2, 10)}`;
}

function today() {
    return new Date().toISOString().slice(0, 10);
}

function escapeHtml(value) {
    const node = document.createElement('div');
    node.textContent = String(value ?? '');
    return node.innerHTML;
}

function nl(value) {
    return escapeHtml(value || '').replace(/\n/g, '<br>');
}

function safeAssetUrl(value) {
    return String(value || '').replace(/["'()\\\n\r\f]/g, '');
}

function value(id) {
    return $(`#${id}`).value;
}

function checkedValues(selector) {
    return $$(`${selector} input[type="checkbox"]:checked`).map((input) => input.value);
}

function setCheckedValues(selector, values) {
    $$(`${selector} input[type="checkbox"]`).forEach((input) => {
        input.checked = values.includes(input.value);
    });
    syncChecks();
}

function syncChecks() {
    $$('.check').forEach((chip) => {
        const input = chip.querySelector('input');
        if (input) chip.classList.toggle('active', input.checked);
    });
}

function refreshIcons() {
    if (window.lucide) window.lucide.createIcons();
}

function selectedStudent() {
    return state.data.students.find((student) => student.id === state.selectedStudentId) || null;
}

function getLastSession(studentId) {
    return state.data.sessions
        .filter((session) => session.studentId === studentId)
        .sort((a, b) => b.date.localeCompare(a.date))[0] || null;
}

function studentSessions(studentId) {
    return state.data.sessions
        .filter((session) => session.studentId === studentId)
        .sort((a, b) => a.date.localeCompare(b.date));
}

function recommendForStudent(student) {
    if (!student) return playbooks[0];
    return playbooks.find((playbook) => playbook.archetypes.some((item) => (student.archetypes || []).includes(item))) || playbooks[0];
}

function switchView(view) {
    state.activeView = view;
    $$('.view').forEach((section) => section.classList.toggle('hidden', section.id !== view));
    $$('.studio-nav button').forEach((button) => button.classList.toggle('active', button.dataset.view === view));
    renderAll();
}

function setupStaticControls() {
    $('#archetype-checks').innerHTML = archetypes
        .map((item) => `<label class="check"><input class="arch-check" type="checkbox" value="${escapeHtml(item)}"><span>${escapeHtml(item)}</span></label>`)
        .join('');
    $('#red-flag-checks').innerHTML = redFlags
        .map((item) => `<label class="check"><input class="red-check" type="checkbox" value="${escapeHtml(item)}"><span>${escapeHtml(item)}</span></label>`)
        .join('');
    $('#l_playbook').innerHTML = playbooks.map((item) => `<option>${escapeHtml(item.name)}</option>`).join('');
    setupFlashcardControls();
}

function setupFlashcardControls() {
    const cards = flashcards();
    const levels = [...new Set(cards.map((card) => card.level))].sort((a, b) => {
        return (flashcardLevelRank[a] || 99) - (flashcardLevelRank[b] || 99) || a.localeCompare(b);
    });
    $('#flashcard-level').innerHTML = '<option value="all">All levels</option>' + levels
        .map((level) => `<option value="${escapeHtml(level)}">${escapeHtml(level)}</option>`)
        .join('');
}

function renderAll() {
    renderSelectors();
    renderSidebar();
    renderDashboard();
    renderStudentList();
    fillStudentForm();
    renderAssessmentHistory();
    renderSessionPlan();
    renderProgress();
    renderPlaybook();
    renderThemeTable();
    renderReport();
    renderFlashcards();
    syncChecks();
    refreshIcons();
}

function renderSelectors() {
    const options = state.data.students
        .map((student) => `<option value="${student.id}" ${student.id === state.selectedStudentId ? 'selected' : ''}>${escapeHtml(student.name || 'Unnamed')}</option>`)
        .join('');

    ['assessment-student', 'session-student', 'progress-student', 'playbook-student', 'report-student'].forEach((id) => {
        const select = $(`#${id}`);
        select.innerHTML = options;
        select.value = state.selectedStudentId || '';
    });
}

function renderSidebar() {
    const student = selectedStudent();
    const last = student ? getLastSession(student.id) : null;
    $('#sidebar-student').textContent = student ? student.name : 'No student selected';
    $('#sidebar-next').textContent = last?.next || student?.goal || 'Load demo data or create a student.';
}

function renderDashboard() {
    const active = state.data.students.filter((student) => student.status !== 'Inactive').length;
    const cautions = state.data.students.filter((student) => student.caution || (student.redFlags || []).length).length;
    const sessionCount = state.data.sessions.length;
    const avgResult = sessionCount
        ? Math.round(state.data.sessions.reduce((sum, session) => sum + (Number(session.result) || 0), 0) / sessionCount)
        : 0;

    $('#dashboard-metrics').innerHTML = [
        ['Private Students', active],
        ['Students with Caution', cautions],
        ['Logged Sessions', sessionCount],
        ['Avg Class Result', `${avgResult}%`]
    ].map(([label, metric]) => `
        <article class="metric-card">
            <strong class="metric">${escapeHtml(metric)}</strong>
            <span>${escapeHtml(label)}</span>
        </article>
    `).join('');

    $('#dashboard-students').innerHTML = state.data.students.length
        ? state.data.students.map((student) => {
            const last = getLastSession(student.id);
            const cautionClass = student.caution || (student.redFlags || []).length ? 'danger' : 'good';
            return `
                <button class="student-item ${student.id === state.selectedStudentId ? 'selected' : ''}" type="button" data-select-student="${student.id}">
                    <span>
                        <strong>${escapeHtml(student.name || 'Unnamed')}</strong>
                        <span class="muted">${escapeHtml((student.archetypes || []).slice(0, 2).join(' / ') || 'No archetype')}<br>Last: ${escapeHtml(last?.date || '-')}</span>
                    </span>
                    <span class="tag ${cautionClass}">${escapeHtml(student.status || 'Active')}</span>
                </button>
            `;
        }).join('')
        : '<p class="muted">No student yet. Load demo data or create a new profile.</p>';

    $('#dashboard-focus').innerHTML = state.data.students.length
        ? state.data.students.map((student) => {
            const last = getLastSession(student.id);
            return `
                <article class="recommendation">
                    <strong>${escapeHtml(student.name || 'Unnamed')}</strong><br>
                    <span class="muted">Goal:</span> ${escapeHtml(student.goal || '-')}<br>
                    <span class="muted">Caution:</span> ${escapeHtml(student.caution || '-')}<br>
                    <span class="muted">Next Plan:</span> ${escapeHtml(last?.next || recommendForStudent(student).name)}
                </article>
            `;
        }).join('')
        : '<p class="muted">Dashboard will show goals, cautions, and next plan once students are added.</p>';
}

function renderStudentList() {
    $('#student-list').innerHTML = state.data.students.length
        ? state.data.students.map((student) => `
            <button class="student-item ${student.id === state.selectedStudentId ? 'selected' : ''}" type="button" data-select-student="${student.id}">
                <span>
                    <strong>${escapeHtml(student.name || 'Unnamed')}</strong>
                    <span class="muted">${escapeHtml(student.occupation || '-')} / ${escapeHtml(student.classType || 'Private')}</span>
                </span>
                <span class="tag">${escapeHtml(student.status || 'Active')}</span>
            </button>
        `).join('')
        : '<p class="muted">No student yet.</p>';
}

function fillStudentForm() {
    const student = selectedStudent() || {};
    const fields = ['name', 'age', 'occupation', 'start', 'classType', 'frequency', 'goal', 'lifestyle', 'painArea', 'painLevel', 'status', 'caution', 'pattern'];
    fields.forEach((field) => {
        const element = $(`#s_${field}`);
        if (element) element.value = student[field] || '';
    });
    setCheckedValues('#archetype-checks', student.archetypes || []);
    setCheckedValues('#red-flag-checks', student.redFlags || []);
}

function newStudent() {
    const student = {
        id: uid(),
        name: 'New Student',
        start: today(),
        classType: 'Private',
        status: 'Active',
        archetypes: [],
        redFlags: []
    };
    state.data.students.unshift(student);
    state.selectedStudentId = student.id;
    saveData();
    switchView('students');
}

function saveStudent() {
    let student = selectedStudent();
    if (!student) {
        newStudent();
        student = selectedStudent();
    }

    ['name', 'age', 'occupation', 'start', 'classType', 'frequency', 'goal', 'lifestyle', 'painArea', 'painLevel', 'status', 'caution', 'pattern'].forEach((field) => {
        student[field] = value(`s_${field}`);
    });
    student.archetypes = checkedValues('#archetype-checks');
    student.redFlags = checkedValues('#red-flag-checks');
    saveData();
    renderAll();
    showToast('Student saved');
}

function deleteSelectedStudent() {
    const student = selectedStudent();
    if (!student) return;
    if (!window.confirm(`Delete ${student.name || 'selected student'}?`)) return;
    state.data.students = state.data.students.filter((item) => item.id !== student.id);
    state.data.assessments = state.data.assessments.filter((item) => item.studentId !== student.id);
    state.data.sessions = state.data.sessions.filter((item) => item.studentId !== student.id);
    state.selectedStudentId = state.data.students[0]?.id || null;
    saveData();
    renderAll();
}

function saveAssessment() {
    const studentId = value('assessment-student');
    if (!studentId) return showToast('Select a student first');
    state.selectedStudentId = studentId;
    state.data.assessments.push({
        id: uid(),
        studentId,
        date: value('a_date') || today(),
        focus: value('a_focus'),
        posture: value('a_posture'),
        tests: value('a_tests'),
        control: value('a_control'),
        mobility: value('a_mobility'),
        stability: value('a_stability'),
        breath: value('a_breath'),
        awareness: value('a_awareness'),
        endurance: value('a_endurance'),
        note: value('a_note')
    });
    ['a_focus', 'a_posture', 'a_tests', 'a_control', 'a_mobility', 'a_stability', 'a_breath', 'a_awareness', 'a_endurance', 'a_note'].forEach((id) => {
        $(`#${id}`).value = '';
    });
    saveData();
    renderAll();
    showToast('Assessment saved');
}

function renderAssessmentHistory() {
    const studentId = value('assessment-student') || state.selectedStudentId;
    const items = state.data.assessments
        .filter((assessment) => assessment.studentId === studentId)
        .sort((a, b) => b.date.localeCompare(a.date));

    $('#assessment-history').innerHTML = items.length
        ? items.map((item) => `
            <article class="timeline-item">
                <strong>${escapeHtml(item.date)} / ${escapeHtml(item.focus || 'Assessment')}</strong><br>
                <span class="muted">Score:</span> Control ${escapeHtml(item.control || '-')} / Mobility ${escapeHtml(item.mobility || '-')} / Stability ${escapeHtml(item.stability || '-')} / Breath ${escapeHtml(item.breath || '-')}<br><br>
                <strong>Tests</strong><br>${nl(item.tests)}<br><br>
                <strong>Note</strong><br>${nl(item.note)}
            </article>
        `).join('')
        : '<p class="muted">No assessment history yet.</p>';
}

function prefillSessionFromStudent() {
    const student = selectedStudent();
    if (!student) return;
    const rec = recommendForStudent(student);
    const sessions = studentSessions(student.id);
    $('#l_date').value = today();
    $('#l_no').value = String(sessions.length + 1);
    $('#l_theme').value = rec.name;
    $('#l_playbook').value = rec.name;
    $('#l_experience_select').value = rec.movementExperience;
    $('#l_experience').value = rec.experience;
    $('#l_before').value = rec.tests.split(', ').map((test) => `${test}:`).join('\n');
    $('#l_exercises').value = rec.exercises.split(', ').map((exercise, index) => `${index + 1}. ${exercise}`).join('\n');
    $('#l_painBefore').value = student.painLevel || '';
    generateSessionPlan();
}

function safetyTagsForStudent(student) {
    const tags = [];
    const raw = `${student?.painArea || ''} ${student?.caution || ''} ${(student?.redFlags || []).join(' ')}`.toLowerCase();
    if (raw.includes('wrist')) tags.push('wrist_pain');
    if (raw.includes('knee')) tags.push('knee_pain');
    if (raw.includes('back') || raw.includes('ql') || raw.includes('disc')) tags.push('low_back_pain');
    if (raw.includes('blood') || raw.includes('heart')) tags.push('high_blood_pressure');
    return [...new Set(tags)];
}

function buildSessionRequest() {
    const student = selectedStudent();
    const rec = recommendForStudent(student);
    const equipment = checkedValues('#session-equipment');
    if (!equipment.length) throw new Error('Choose at least one primary apparatus.');

    const observations = [
        ...(rec.observations || []),
        ...(student?.archetypes || []),
        student?.painArea,
        student?.pattern,
        value('l_theme')
    ].filter(Boolean);

    return {
        students: 1,
        movement_experience: value('l_experience_select') || rec.movementExperience,
        level: value('l_level'),
        equipment,
        duration_minutes: Number(value('l_duration')) || 60,
        observations,
        group_safety: {
            contraindications: safetyTagsForStudent(student),
            risk_policy: 'Conservative'
        }
    };
}

function generateSessionPlan() {
    $('#session-plan-error').textContent = '';
    try {
        state.plan = generateClassPlan(buildSessionRequest());
        state.markdown = renderMarkdown(state.plan);
        renderSessionPlan();
        showToast('Plan generated');
    } catch (error) {
        $('#session-plan-error').textContent = error.message;
    }
}

function saveSession() {
    const studentId = value('session-student');
    if (!studentId) return showToast('Select a student first');
    state.selectedStudentId = studentId;
    if (!state.plan) generateSessionPlan();

    state.data.sessions.push({
        id: uid(),
        studentId,
        date: value('l_date') || today(),
        no: value('l_no'),
        duration: `${value('l_duration') || 60} min`,
        energy: value('l_energy'),
        painBefore: value('l_painBefore'),
        painAfter: value('l_painAfter'),
        level: value('l_level'),
        movementExperience: value('l_experience_select'),
        equipment: checkedValues('#session-equipment'),
        theme: value('l_theme'),
        playbook: value('l_playbook'),
        experience: value('l_experience'),
        before: value('l_before'),
        exercises: value('l_exercises'),
        easy: value('l_easy'),
        hard: value('l_hard'),
        comp: value('l_comp'),
        cue: value('l_cue'),
        after: value('l_after'),
        result: value('l_result'),
        feeling: value('l_feeling'),
        next: value('l_next'),
        generatedPlan: state.plan,
        generatedMarkdown: state.markdown
    });
    saveData();
    renderAll();
    showToast('Session saved');
}

function renderSessionPlan() {
    const target = $('#session-plan');
    if (!state.plan) {
        target.innerHTML = '<p class="muted">Generate a plan from the selected student to preview the class journey.</p>';
        return;
    }
    if (state.planView === 'markdown') {
        target.innerHTML = `<pre class="code-view">${escapeHtml(state.markdown)}</pre>`;
        return;
    }
    if (state.planView === 'json') {
        target.innerHTML = `<pre class="code-view">${escapeHtml(JSON.stringify(state.plan, null, 2))}</pre>`;
        return;
    }
    target.innerHTML = renderPlanView(state.plan);
}

function renderPlanView(plan) {
    const strategy = plan.movement_strategy;
    const bars = Object.entries(strategy.emphasis)
        .map(([system, value]) => `
            <div class="bar-row">
                <span>${escapeHtml(system)}</span>
                <div class="bar-track"><div class="bar-fill" style="width:${Number(value)}%"></div></div>
                <strong>${Number(value)}%</strong>
            </div>
        `)
        .join('');
    const phases = plan.journey.map((phase, index) => `
        <article class="phase-block">
            <header class="phase-head">
                <span class="phase-index">${index + 1}</span>
                <div>
                    <h3>${escapeHtml(phase.phase)}</h3>
                    <p>${escapeHtml(phase.purpose)}</p>
                </div>
                <span class="phase-time">${phase.target_duration_minutes} min</span>
            </header>
            <div class="exercise-table">
                ${phase.exercises.map(renderExerciseRow).join('')}
            </div>
        </article>
    `).join('');

    return `
        <div class="strategy-grid">
            <div class="recommendation">
                <strong>${escapeHtml(plan.class_title)}</strong><br>
                <span class="muted">${escapeHtml(strategy.primary_focus)} primary focus with ${escapeHtml(strategy.secondary_focus)} support.</span>
            </div>
            <div class="bar-list">${bars}</div>
        </div>
        <div class="phase-list">${phases}</div>
        ${renderBenchmarks(plan)}
        ${renderSafety(plan)}
    `;
}

function renderExerciseRow(exercise) {
    const objectives = exercise.movement_objectives
        .slice(0, 2)
        .map((objective) => `<span class="tag good">${escapeHtml(objective)}</span>`)
        .join(' ');
    const safety = exercise.safety_notes
        .map((note) => `<span class="tag danger">${escapeHtml(note)}</span>`)
        .join(' ');
    return `
        <div class="exercise-row">
            <div>
                <div class="exercise-name">${escapeHtml(exercise.name)}</div>
                <div class="exercise-meta">${escapeHtml(exercise.exercise_id)} / ${escapeHtml(exercise.role)}</div>
            </div>
            <div><span class="tag info">${escapeHtml(exercise.apparatus)}</span></div>
            <div class="exercise-meta">${exercise.duration_minutes} min</div>
            <div class="exercise-cues">${objectives}<br>${escapeHtml(exercise.teaching_cues.slice(0, 2).join(' '))} ${safety}</div>
        </div>
    `;
}

function renderBenchmarks(plan) {
    return `
        <section class="benchmark-grid" aria-label="Benchmarks">
            ${plan.benchmark.assessments.map((assessment) => `
                <article class="benchmark-item">
                    <h3>${escapeHtml(assessment.name)}</h3>
                    <p>${escapeHtml(assessment.instruction)}</p>
                    <p>${assessment.what_to_watch.map(escapeHtml).join(' / ')}</p>
                </article>
            `).join('')}
        </section>
    `;
}

function renderSafety(plan) {
    const summary = plan.safety_summary;
    const excluded = summary.excluded_exercises.length
        ? summary.excluded_exercises.map((item) => `<article class="safety-item"><h3>${escapeHtml(item.name)}</h3><p>${escapeHtml(item.reason)}</p></article>`).join('')
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

function renderProgress() {
    const studentId = value('progress-student') || state.selectedStudentId;
    const student = state.data.students.find((item) => item.id === studentId);
    const sessions = studentSessions(studentId);
    const last = sessions[sessions.length - 1];
    const avg = sessions.length
        ? Math.round(sessions.reduce((sum, session) => sum + (Number(session.result) || 0), 0) / sessions.length)
        : 0;

    $('#progress-snapshot').innerHTML = student
        ? `
            <strong>${escapeHtml(student.name)}</strong>
            <div class="tag-wrap">${(student.archetypes || []).map((tag) => `<span class="tag">${escapeHtml(tag)}</span>`).join('')}</div>
            <br><span class="muted">Goal</span><br>${nl(student.goal) || '-'}
            <br><br><span class="muted">Sessions</span><strong class="metric">${sessions.length}</strong>
            <span class="muted">Average result</span><strong class="metric">${avg}%</strong>
            <span class="muted">Latest next plan</span><br>${nl(last?.next) || '-'}
        `
        : '<p class="muted">No student selected.</p>';

    $('#progress-timeline').innerHTML = sessions.length
        ? sessions.map((session) => `
            <article class="timeline-item">
                <strong>Session ${escapeHtml(session.no || '-')} / ${escapeHtml(session.date)}</strong><br>
                <span class="tag">${escapeHtml(session.theme || session.playbook || 'No theme')}</span><br><br>
                <strong>Before</strong><br>${nl(session.before) || '-'}<br><br>
                <strong>After</strong><br>${nl(session.after) || '-'}<br><br>
                <strong>Compensation</strong><br>${nl(session.comp) || '-'}<br><br>
                <strong>Next Plan</strong><br>${nl(session.next) || '-'}
            </article>
        `).join('')
        : '<p class="muted">No session log yet.</p>';
}

function renderPlaybook() {
    const studentId = value('playbook-student') || state.selectedStudentId;
    const student = state.data.students.find((item) => item.id === studentId);
    if (!student) {
        $('#detected-pattern').innerHTML = '<p class="muted">No student selected.</p>';
        $('#playbook-recommendation').innerHTML = '<p class="muted">Create or select a student first.</p>';
        return;
    }
    const rec = recommendForStudent(student);
    $('#detected-pattern').innerHTML = `
        <strong>${escapeHtml(student.name)}</strong><br><br>
        <span class="muted">Archetype</span>
        <div class="tag-wrap">${(student.archetypes || []).map((tag) => `<span class="tag">${escapeHtml(tag)}</span>`).join('') || '<span class="muted">No archetype</span>'}</div><br>
        <span class="muted">Pain / caution</span><br>${escapeHtml(student.painArea || '-')} / pain ${escapeHtml(student.painLevel || '-')}/10<br>${nl(student.caution) || '-'}<br><br>
        <span class="muted">Movement pattern</span><br>${nl(student.pattern) || '-'}
    `;
    $('#playbook-recommendation').innerHTML = `
        <article class="recommendation">
            <strong>${escapeHtml(rec.name)}</strong><br><br>
            <strong>Movement Experience</strong><br>${escapeHtml(rec.experience)}<br><br>
            <strong>Before/After Test</strong><br>${escapeHtml(rec.tests)}<br><br>
            <strong>Exercise Family</strong><br>${escapeHtml(rec.exercises)}<br><br>
            <strong>Caution</strong><br>${escapeHtml(rec.cautions)}
        </article>
    `;
}

function renderThemeTable() {
    $('#theme-table').innerHTML = `
        <tr><th>Theme</th><th>Archetype</th><th>Tests</th><th>Exercise Family</th><th>Caution</th></tr>
        ${playbooks.map((playbook) => `
            <tr>
                <td><strong>${escapeHtml(playbook.name)}</strong></td>
                <td>${escapeHtml(playbook.archetypes.join(', '))}</td>
                <td>${escapeHtml(playbook.tests)}</td>
                <td>${escapeHtml(playbook.exercises)}</td>
                <td>${escapeHtml(playbook.cautions)}</td>
            </tr>
        `).join('')}
    `;
}

function renderReport() {
    const studentId = value('report-student') || state.selectedStudentId;
    const student = state.data.students.find((item) => item.id === studentId);
    if (!student) {
        $('#student-report').textContent = 'No student selected.';
        return;
    }
    const sessions = studentSessions(student.id);
    const first = sessions[0];
    const last = sessions[sessions.length - 1];
    const avg = sessions.length
        ? Math.round(sessions.reduce((sum, session) => sum + (Number(session.result) || 0), 0) / sessions.length)
        : '-';
    const themes = [...new Set(sessions.map((session) => session.theme || session.playbook).filter(Boolean))];
    const rec = recommendForStudent(student);

    $('#student-report').textContent = `MPS Client Studio - Student Progress Report

Student: ${student.name || '-'}
Period: ${first?.date || '-'} to ${last?.date || '-'}
Sessions Logged: ${sessions.length}
Average Class Result: ${avg}%

Main Goal:
${student.goal || '-'}

Archetype:
- ${(student.archetypes || []).join('\n- ') || '-'}

Caution:
${student.caution || '-'}

Main Themes Used:
- ${themes.join('\n- ') || '-'}

Initial / Early Pattern:
${first?.comp || student.pattern || '-'}

Latest Progress Evidence:
After Test:
${last?.after || '-'}

Cue That Worked:
${last?.cue || '-'}

Still Needs Work:
${last?.hard || last?.comp || '-'}

Next Phase / Recommended Focus:
${last?.next || rec.name}

Teacher Note:
This report summarizes movement observations for Pilates programming and progress tracking. It is not a medical diagnosis.`;
}

function flashcards() {
    return Array.isArray(window.MPS_FLASHCARDS) ? window.MPS_FLASHCARDS : [];
}

function filteredFlashcards() {
    const store = window.MPS_FLASHCARD_STORE({ staticCards: flashcards() });
    return store.listCards({
        category: state.flashcardCategory,
        level: state.flashcardLevel,
        query: state.flashcardQuery
    });
}

async function renderFlashcards() {
    const cards = flashcards();
    $('#flashcard-metrics').innerHTML = flashcardCategories.map((category) => {
        const count = cards.filter((card) => card.category === category).length;
        return `
            <button class="metric-card flashcard-metric ${category === state.flashcardCategory ? 'active' : ''}" type="button" data-flashcard-category="${escapeHtml(category)}">
                <strong class="metric">${count}</strong>
                <span>${escapeHtml(category)}</span>
            </button>
        `;
    }).join('');

    $$('.flashcard-tabs button').forEach((button) => {
        button.classList.toggle('active', button.dataset.flashcardCategory === state.flashcardCategory);
    });

    const filtered = await filteredFlashcards();
    $('#flashcard-deck').innerHTML = filtered.length
        ? filtered.map(renderFlashcard).join('')
        : '<p class="muted">No flashcards match the current filters.</p>';
}

function renderFlashcard(card) {
    const slug = card.category.toLowerCase();
    const artClass = card.image ? 'flashcard-art has-image' : 'flashcard-art';
    const artStyle = card.image ? ` style="background-image: url('${safeAssetUrl(card.image)}')"` : '';
    const rows = [
        ['Level', card.level],
        ['Objective', card.objective],
        ['Principle', card.principle],
        ['Cue', card.cue],
        ['Regress', card.regress],
        ['Progress', card.progress]
    ];

    return `
        <article class="studio-flashcard flashcard-${slug}">
            <header class="flashcard-head">
                <div class="${artClass}"${artStyle} aria-hidden="true"></div>
                <div>
                    <strong>${escapeHtml(card.name)}</strong>
                    <span>${escapeHtml(card.category)} / ${escapeHtml(card.id)}</span>
                </div>
            </header>
            <section class="flashcard-front">
                <span class="muted">Front</span>
                <p>${escapeHtml(card.front)}</p>
            </section>
            <section class="flashcard-back">
                ${rows.map(([label, value]) => `
                    <div>
                        <b>${escapeHtml(label)}</b>
                        <span>${escapeHtml(value || '-')}</span>
                    </div>
                `).join('')}
            </section>
        </article>
    `;
}

function exportData() {
    const blob = new Blob([JSON.stringify(state.data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'mps-client-studio-data.json';
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
}

function importData(event) {
    const file = event.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
        try {
            state.data = normalizeData(JSON.parse(reader.result));
            state.selectedStudentId = state.data.students[0]?.id || null;
            saveData();
            renderAll();
            showToast('Data imported');
        } catch (error) {
            showToast('Invalid JSON');
        }
    };
    reader.readAsText(file);
}

function seedDemoData() {
    const natt = {
        id: uid(),
        name: 'K. Natt',
        occupation: 'Office lead',
        start: '2026-07-01',
        classType: 'Private',
        frequency: '1x/week',
        goal: 'Improve hip stability, pelvis-lumbar dissociation, and reduce lumbar compensation.',
        lifestyle: 'Desk work with occasional strength training.',
        painArea: 'Low back / hip',
        painLevel: '2',
        status: 'Active',
        caution: 'Watch lumbar gripping and pelvis rotation during single-leg work.',
        pattern: 'Pelvis control is inconsistent; lumbar extension appears when range increases.',
        archetypes: ['Low Back / QL Dominant', 'Strong but Tight'],
        redFlags: []
    };
    const eff = {
        id: uid(),
        name: 'K. Eff',
        occupation: 'IT',
        start: '2026-06-05',
        classType: 'Private',
        frequency: '1x/week',
        goal: 'Reduce neck and shoulder tension while keeping wrist-friendly core work.',
        lifestyle: 'Long computer hours; right wrist gets irritated with loaded extension.',
        painArea: 'Neck / shoulder / right wrist',
        painLevel: '4',
        status: 'Active',
        caution: 'Right wrist pain; avoid loaded wrist extension and modify plank or quadruped.',
        pattern: 'Shoulder elevation, neck gripping, and rib flare during arm raise.',
        archetypes: ['Office Syndrome'],
        redFlags: []
    };
    const august = {
        id: uid(),
        name: 'K. August',
        occupation: 'Active client',
        start: '2026-07-05',
        classType: 'Private',
        frequency: '1x/week',
        goal: 'Increase spine mobility and active flexibility while keeping enough challenge.',
        lifestyle: 'Strong but tight; likes harder classes.',
        painArea: 'Back stiffness',
        painLevel: '1',
        status: 'Active',
        caution: 'Avoid pushing range beyond control.',
        pattern: 'Spine articulation is stiff; tends to borrow range from lumbar extension.',
        archetypes: ['Strong but Tight'],
        redFlags: []
    };

    state.data.students = [natt, eff, august];
    state.selectedStudentId = natt.id;
    state.data.assessments = [
        {
            id: uid(),
            studentId: natt.id,
            date: '2026-07-05',
            focus: 'Low Back / QL',
            posture: 'Mild rib flare and pelvis shift in standing.',
            tests: 'Standing Roll Down: back tension\nSide Bend: left side restricted\nSingle Leg Balance: pelvis rotation',
            control: '3',
            mobility: '3',
            stability: '2',
            breath: '3',
            awareness: '3',
            endurance: '3',
            note: 'Start with rib-pelvis-hip connection and retest roll down.'
        }
    ];
    state.data.sessions = [
        {
            id: uid(),
            studentId: natt.id,
            date: '2026-07-05',
            no: '1',
            duration: '60 min',
            energy: '3',
            painBefore: '2',
            painAfter: '1',
            level: 'BeginnerIntermediate',
            movementExperience: 'SpineReset',
            equipment: ['Reformer', 'Chair'],
            theme: 'Spine Mobility x Hip Stability',
            playbook: 'Low Back / QL - Rib Pelvis Hip Connection',
            experience: 'Roll down feels smoother and pelvis control improves.',
            before: 'Standing Roll Down: back tension / pelvis shift\nTwist: mildly limited',
            exercises: 'Cat Cow\nFootwork\nPelvic Curl\nFeet in Straps\nScooter\nMermaid\nBarrel Stretch',
            easy: 'Cat Cow',
            hard: 'Single leg control',
            comp: 'Pelvis rotation and lumbar gripping',
            cue: 'Heavy pelvis on both sides / keep the waist long',
            after: 'Standing Roll Down improved / Twist smoother',
            result: '75',
            feeling: 'Lighter back',
            next: 'Lateral Hip Stability x QL Rebalance'
        },
        {
            id: uid(),
            studentId: eff.id,
            date: '2026-07-07',
            no: '3',
            duration: '60 min',
            energy: '3',
            painBefore: '4',
            painAfter: '2',
            level: 'BeginnerIntermediate',
            movementExperience: 'ShoulderFreedom',
            equipment: ['Reformer', 'Chair'],
            theme: 'Office Syndrome x Wrist-Friendly Core',
            playbook: 'Office Syndrome - Scapular Stability x Thoracic Mobility',
            experience: 'Neck feels lighter and core work avoids wrist compression.',
            before: 'Arm Raise: rib flare / shoulder elevation\nCervical Rotation: right limited',
            exercises: 'Breathing\nPelvic Curl\nSupine Arm Springs\nFeet in Straps\nMermaid\nRoll Back\nBarrel Extension',
            easy: 'Pelvic Curl',
            hard: 'Roll Back',
            comp: 'Shoulder elevation and neck gripping',
            cue: 'Long neck, wide shoulders, soft ribs',
            after: 'Arm Raise shoulder elevation decreased\nCervical Rotation improved',
            result: '75',
            feeling: 'Shoulders lighter',
            next: 'Add scapular pulling and gentle thoracic extension; avoid loaded wrist'
        }
    ];
    saveData();
    prefillSessionFromStudent();
    renderAll();
    showToast('Demo data loaded');
}

async function copyReport() {
    try {
        await navigator.clipboard.writeText($('#student-report').textContent);
        showToast('Report copied');
    } catch (error) {
        showToast('Clipboard unavailable');
    }
}

function showToast(message) {
    const toast = document.createElement('div');
    toast.className = 'toast';
    toast.textContent = message;
    document.body.appendChild(toast);
    window.setTimeout(() => toast.remove(), 1800);
}

function attachEvents() {
    $$('.studio-nav button').forEach((button) => {
        button.addEventListener('click', () => switchView(button.dataset.view));
    });

    document.body.addEventListener('click', (event) => {
        const button = event.target.closest('[data-select-student]');
        if (!button) return;
        state.selectedStudentId = button.dataset.selectStudent;
        saveData();
        renderAll();
    });

    $('#load-demo').addEventListener('click', seedDemoData);
    $('#export-data').addEventListener('click', exportData);
    $('#import-trigger').addEventListener('click', () => $('#import-file').click());
    $('#import-file').addEventListener('change', importData);
    $('#new-student').addEventListener('click', newStudent);
    $('#save-student').addEventListener('click', saveStudent);
    $('#delete-student').addEventListener('click', deleteSelectedStudent);
    $('#save-assessment').addEventListener('click', saveAssessment);
    $('#generate-session-plan').addEventListener('click', generateSessionPlan);
    $('#save-session').addEventListener('click', saveSession);
    $('#copy-report').addEventListener('click', copyReport);
    $('#print-report').addEventListener('click', () => window.print());
    $('#print-flashcards').addEventListener('click', () => window.print());
    $('#flashcard-search').addEventListener('input', (event) => {
        state.flashcardQuery = event.target.value;
        renderFlashcards();
    });
    $('#flashcard-level').addEventListener('change', (event) => {
        state.flashcardLevel = event.target.value;
        renderFlashcards();
    });

    document.body.addEventListener('click', (event) => {
        const categoryTarget = event.target.closest('[data-flashcard-category]');
        if (!categoryTarget) return;
        state.flashcardCategory = categoryTarget.dataset.flashcardCategory;
        renderFlashcards();
    });

    ['assessment-student', 'session-student', 'progress-student', 'playbook-student', 'report-student'].forEach((id) => {
        $(`#${id}`).addEventListener('change', (event) => {
            state.selectedStudentId = event.target.value || state.selectedStudentId;
            saveData();
            if (id === 'session-student') prefillSessionFromStudent();
            renderAll();
        });
    });

    ['l_duration', 'l_level', 'l_experience_select', 'l_theme'].forEach((id) => {
        $(`#${id}`).addEventListener('change', generateSessionPlan);
    });

    $$('#session-equipment .check').forEach((chip) => {
        chip.addEventListener('click', () => {
            window.setTimeout(() => {
                syncChecks();
                generateSessionPlan();
            }, 0);
        });
    });

    $$('.check').forEach((chip) => {
        chip.addEventListener('click', () => window.setTimeout(syncChecks, 0));
    });

    $$('.tab-button[data-plan-view]').forEach((button) => {
        button.addEventListener('click', () => {
            state.planView = button.dataset.planView;
            $$('.tab-button[data-plan-view]').forEach((tab) => tab.classList.toggle('active', tab === button));
            renderSessionPlan();
        });
    });
}

$('#a_date').value = today();
$('#l_date').value = today();
setupStaticControls();
attachEvents();
if (state.selectedStudentId) prefillSessionFromStudent();
renderAll();
