const LEVEL_NUMERIC = {
    Beginner: 1,
    BeginnerIntermediate: 2,
    Intermediate: 3,
    IntermediateAdvanced: 4,
    Advanced: 5
};

const EXPERIENCE_LABELS = {
    ShoulderFreedom: 'Shoulder Freedom',
    HappyHips: 'Happy Hips',
    SpineReset: 'Spine Reset'
};

const LEVEL_LABELS = {
    Beginner: 'Beginner',
    BeginnerIntermediate: 'Beginner Intermediate',
    Intermediate: 'Intermediate',
    IntermediateAdvanced: 'Intermediate Advanced',
    Advanced: 'Advanced'
};

const PHASES = {
    45: [
        ['ARRIVE', 4],
        ['PREPARE', 7],
        ['BUILD', 15],
        ['INTEGRATE', 7],
        ['CHALLENGE', 6],
        ['TRANSFER', 3],
        ['RESET & RETEST', 3]
    ],
    60: [
        ['ARRIVE', 5],
        ['PREPARE', 10],
        ['BUILD', 20],
        ['INTEGRATE', 10],
        ['CHALLENGE', 8],
        ['TRANSFER', 4],
        ['RESET & RETEST', 3]
    ],
    75: [
        ['ARRIVE', 6],
        ['PREPARE', 12],
        ['BUILD', 26],
        ['INTEGRATE', 13],
        ['CHALLENGE', 10],
        ['TRANSFER', 5],
        ['RESET & RETEST', 3]
    ]
};

const PHASE_PURPOSES = {
    'ARRIVE': 'Assess baseline, settle attention, and name the class target.',
    'PREPARE': 'Prepare breath, joints, and control for the main intervention.',
    'BUILD': 'Apply the main Prime exercises that create the target change.',
    'INTEGRATE': 'Connect the target system with whole-body coordination.',
    'CHALLENGE': 'Add the planned 20 percent challenge without losing quality.',
    'TRANSFER': 'Carry the new movement quality into functional patterns.',
    'RESET & RETEST': 'Downshift effort and repeat the benchmark for felt comparison.'
};

const PHASE_ROLES = {
    'ARRIVE': 'Assess',
    'PREPARE': 'Prepare',
    'BUILD': 'Prime',
    'INTEGRATE': 'Integrate',
    'CHALLENGE': 'Challenge',
    'TRANSFER': 'Transfer',
    'RESET & RETEST': 'Restore'
};

function generateClassPlan(request) {
    validateRequest(request);

    const allocations = PHASES[request.duration_minutes].map(([phase, minutes]) => ({ phase, minutes }));
    const base = SEED_DATA.strategies.find((strategy) => strategy.movement_experience === request.movement_experience);
    if (!base) throw new Error(`No base strategy for ${request.movement_experience}`);

    const modifiers = matchModifiers(request.observations || []);
    const strategy = buildStrategy(base, modifiers);
    const safetyTags = new Set((request.group_safety?.contraindications || []).map((tag) => tag.toLowerCase()));
    const benchmarks = SEED_DATA.benchmarks.filter((benchmark) => benchmark.movement_experience === request.movement_experience);
    const used = new Set();
    const modifiedExercises = [];
    const warnings = [];

    const journey = allocations.map((allocation) => {
        const role = PHASE_ROLES[allocation.phase];
        let ranked = rankCandidates(SEED_DATA.exercises, request, allocation.phase, role, strategy, safetyTags, used);
        if (ranked.length === 0) {
            ranked = rankCandidates(SEED_DATA.exercises, request, allocation.phase, role, strategy, safetyTags, new Set());
        }

        const maxCount = maxExerciseCount(allocation.phase);
        const minCount = allocation.phase === 'BUILD' ? 2 : 1;
        let remaining = allocation.minutes;
        const exercises = [];

        for (const { exercise, score } of ranked) {
            if (exercises.length >= maxCount || remaining === 0) break;
            let regression = exercise.regression || null;
            const safety_notes = cautionNotes(exercise, safetyTags);
            if (requiresRegression(exercise, safetyTags)) {
                safety_notes.push('Regression required by group safety constraint.');
                regression = regression || 'Reduce range, load, tempo, or support the position.';
                modifiedExercises.push({
                    exercise_id: exercise.id,
                    name: exercise.name,
                    reason: 'Regression required by group safety constraint.'
                });
            }

            const duration = Math.min(exercise.default_duration_minutes, remaining);
            remaining -= duration;
            used.add(exercise.id);
            exercises.push({
                exercise_id: exercise.id,
                name: exercise.name,
                apparatus: exercise.equipment,
                role,
                duration_minutes: duration,
                movement_objectives: exercise.objectives,
                why_selected: `Matched ${role} role for ${allocation.phase} with score ${score}.`,
                teaching_cues: exercise.cues,
                regression,
                progression: exercise.progression || null,
                safety_notes
            });
        }

        if (exercises.length < minCount) {
            throw new Error(`No candidates for ${allocation.phase} (${role})`);
        }
        if (remaining > 0) {
            warnings.push(`${allocation.phase} is under-filled by ${remaining} minute(s). Add more seed data.`);
        }

        return {
            phase: allocation.phase,
            purpose: PHASE_PURPOSES[allocation.phase],
            target_duration_minutes: allocation.minutes,
            exercises
        };
    });

    const excludedExercises = SEED_DATA.exercises
        .filter((exercise) => hardExcluded(exercise, safetyTags))
        .map((exercise) => ({
            exercise_id: exercise.id,
            name: exercise.name,
            reason: exercise.contraindications
                .filter((contra) => contra.severity === 'HardExclude' && safetyTags.has(contra.tag.toLowerCase()))
                .map((contra) => contra.note)
                .join('; ')
        }));

    const benchmarkPlan = {
        assessments: benchmarks.map((benchmark) => ({
            name: benchmark.name,
            instruction: benchmark.instruction,
            what_to_watch: benchmark.watch_points
        }))
    };

    const classTitle = `${EXPERIENCE_LABELS[request.movement_experience]} - ${request.duration_minutes} min ${LEVEL_LABELS[request.level]}`;
    return {
        class_title: classTitle,
        movement_experience: request.movement_experience,
        duration_minutes: request.duration_minutes,
        level: request.level,
        students: request.students,
        equipment: request.equipment,
        movement_strategy: strategy,
        benchmark: benchmarkPlan,
        journey,
        safety_summary: {
            risk_policy: request.group_safety?.risk_policy || 'Conservative',
            applied_contraindications: request.group_safety?.contraindications || [],
            excluded_exercises: excludedExercises,
            modified_exercises: modifiedExercises,
            safety_notes: [
                'This plan is a teaching aid, not a medical diagnosis. Instructor judgement is required.',
                `Group safety policy: ${request.group_safety?.risk_policy || 'Conservative'}`
            ]
        },
        retest: {
            assessments: benchmarkPlan.assessments,
            expected_improvement: `Students should feel clearer ${EXPERIENCE_LABELS[request.movement_experience]} movement quality and compare it against the opening benchmark.`
        },
        expected_improvement: `Improved ${EXPERIENCE_LABELS[request.movement_experience]} movement quality with whole-body support.`,
        warnings
    };
}

function validateRequest(request) {
    if (!request.students || request.students < 1) throw new Error('Students must be at least 1.');
    if (!PHASES[request.duration_minutes]) throw new Error('Duration must be 45, 60, or 75 minutes.');
    if (!request.equipment || request.equipment.length === 0) throw new Error('Choose at least one primary apparatus.');
    for (const equipment of request.equipment) {
        if (equipment !== 'Reformer' && equipment !== 'Chair') {
            throw new Error(`${equipment} is a movement context, not primary apparatus.`);
        }
    }
}

function matchModifiers(observations) {
    const text = observations.join(' ').toLowerCase();
    return SEED_DATA.modifiers.filter((modifier) =>
        modifier.keywords.some((keyword) => text.includes(keyword.toLowerCase()))
    );
}

function buildStrategy(base, modifiers) {
    const emphasis = { ...base.emphasis };
    const objectives = [...base.objectives];
    const preferred = [];
    const explanation = [base.explanation_template];

    for (const modifier of modifiers) {
        for (const [system, adjustment] of Object.entries(modifier.emphasis)) {
            emphasis[system] = (emphasis[system] || 0) + adjustment;
        }
        for (const objective of modifier.added_objectives) {
            if (!objectives.includes(objective)) objectives.push(objective);
        }
        for (const objective of modifier.preferred_objectives) {
            if (!preferred.includes(objective)) preferred.push(objective);
        }
        explanation.push(modifier.explanation_fragment);
    }

    return {
        primary_focus: base.primary_focus,
        secondary_focus: base.secondary_focus,
        emphasis: normalizeEmphasis(emphasis),
        key_objectives: objectives,
        preferred_exercise_objectives: preferred,
        explanation: explanation.join(' ')
    };
}

function normalizeEmphasis(input) {
    const entries = Object.entries(input).map(([key, value]) => [key, Math.max(0, value)]);
    const total = entries.reduce((sum, [, value]) => sum + value, 0);
    if (total === 0) return {};
    const rows = entries.map(([key, value]) => {
        const exact = (value / total) * 100;
        return { key, pct: Math.floor(exact), rem: exact % 1 };
    });
    let remainder = 100 - rows.reduce((sum, row) => sum + row.pct, 0);
    rows.sort((a, b) => b.rem - a.rem || a.key.localeCompare(b.key));
    for (const row of rows) {
        if (remainder === 0) break;
        row.pct += 1;
        remainder -= 1;
    }
    return Object.fromEntries(rows.sort((a, b) => a.key.localeCompare(b.key)).map((row) => [row.key, row.pct]));
}

function rankCandidates(exercises, request, phase, role, strategy, safetyTags, used) {
    return exercises
        .filter((exercise) => !used.has(exercise.id))
        .filter((exercise) => exercise.roles.includes(role))
        .filter((exercise) => equipmentAllowed(exercise.equipment, request.equipment, phase))
        .filter((exercise) => levelAllowed(exercise, request.level, phase))
        .filter((exercise) => !hardExcluded(exercise, safetyTags))
        .map((exercise) => ({ exercise, score: scoreExercise(exercise, request, role, strategy, safetyTags) }))
        .sort((a, b) => b.score - a.score || a.exercise.difficulty - b.exercise.difficulty || a.exercise.id.localeCompare(b.exercise.id));
}

function equipmentAllowed(equipment, selected, phase) {
    if (equipment === 'Reformer' || equipment === 'Chair') return selected.includes(equipment);
    return ['ARRIVE', 'PREPARE', 'TRANSFER', 'RESET & RETEST'].includes(phase);
}

function levelAllowed(exercise, level, phase) {
    const classLevel = LEVEL_NUMERIC[level];
    if (LEVEL_NUMERIC[exercise.min_level] > classLevel) return false;
    if (classLevel <= LEVEL_NUMERIC[exercise.max_level]) return true;
    return phase === 'CHALLENGE' && exercise.difficulty <= classLevel + 1 && Boolean(exercise.regression);
}

function scoreExercise(exercise, request, role, strategy, safetyTags) {
    let score = 0;
    if (exercise.roles.includes(role)) score += 20;
    score += difficultyFitScore(exercise.difficulty, request.level);
    if (exercise.experiences.includes(request.movement_experience)) score += 10;
    if (exercise.systems.includes(strategy.primary_focus)) score += 8;
    if (exercise.systems.includes(strategy.secondary_focus)) score += 4;
    for (const objective of exercise.objectives) {
        if (strategy.preferred_exercise_objectives.some((wanted) => objective.toLowerCase().includes(wanted.toLowerCase()))) {
            score += 5;
        }
    }
    if (request.equipment.includes(exercise.equipment)) score += 5;
    score -= safetyPenalty(exercise, safetyTags);
    return score;
}

function difficultyFitScore(difficulty, level) {
    const delta = difficulty - LEVEL_NUMERIC[level];
    if (delta <= -1) return 3;
    if (delta === 0) return 5;
    if (delta === 1) return 2;
    return 0;
}

function maxExerciseCount(phase) {
    return {
        'ARRIVE': 2,
        'PREPARE': 3,
        'BUILD': 4,
        'INTEGRATE': 3,
        'CHALLENGE': 2,
        'TRANSFER': 2,
        'RESET & RETEST': 2
    }[phase];
}

function hardExcluded(exercise, safetyTags) {
    return exercise.contraindications.some((contra) =>
        contra.severity === 'HardExclude' && safetyTags.has(contra.tag.toLowerCase())
    );
}

function requiresRegression(exercise, safetyTags) {
    return exercise.contraindications.some((contra) =>
        contra.severity === 'RequireRegression' && safetyTags.has(contra.tag.toLowerCase())
    );
}

function cautionNotes(exercise, safetyTags) {
    return exercise.contraindications
        .filter((contra) => contra.severity === 'Caution' && safetyTags.has(contra.tag.toLowerCase()))
        .map((contra) => contra.note);
}

function safetyPenalty(exercise, safetyTags) {
    return exercise.contraindications
        .filter((contra) => safetyTags.has(contra.tag.toLowerCase()))
        .reduce((sum, contra) => {
            if (contra.severity === 'HardExclude') return sum + 1000;
            if (contra.severity === 'RequireRegression') return sum + 8;
            return sum + 4;
        }, 0);
}

function renderMarkdown(plan) {
    let md = `# ${plan.class_title}\n\n`;
    md += `## Class Summary\n\n`;
    md += `- Movement experience: ${EXPERIENCE_LABELS[plan.movement_experience]}\n`;
    md += `- Duration: ${plan.duration_minutes} min\n`;
    md += `- Level: ${LEVEL_LABELS[plan.level]}\n`;
    md += `- Students: ${plan.students}\n`;
    md += `- Equipment: ${plan.equipment.join(', ')}\n\n`;
    md += `> Teaching aid only. This is not medical advice or diagnosis; instructor judgement is required.\n\n`;
    md += `## Movement Strategy\n\n`;
    md += `- Primary focus: ${plan.movement_strategy.primary_focus}\n`;
    md += `- Secondary focus: ${plan.movement_strategy.secondary_focus}\n\n`;
    md += `| System | Emphasis |\n|---|---:|\n`;
    for (const [system, value] of Object.entries(plan.movement_strategy.emphasis)) {
        md += `| ${system} | ${value}% |\n`;
    }
    md += `\n${plan.movement_strategy.explanation}\n\n`;
    md += `## Before Class Benchmark\n\n`;
    for (const assessment of plan.benchmark.assessments) {
        md += `### ${assessment.name}\n\n${assessment.instruction}\n\n`;
        md += `Watch for:\n${assessment.what_to_watch.map((point) => `- ${point}`).join('\n')}\n\n`;
    }
    md += `## Class Journey\n\n`;
    for (const phase of plan.journey) {
        md += `### ${phase.phase} - ${phase.target_duration_minutes} min\n\n${phase.purpose}\n\n`;
        for (const exercise of phase.exercises) {
            md += `**${exercise.name}** | ${exercise.apparatus} | ${exercise.duration_minutes} min | ${exercise.role}\n\n`;
            md += `- Objectives: ${exercise.movement_objectives.join(', ')}\n`;
            md += `- Why selected: ${exercise.why_selected}\n`;
            md += `- Cues:\n${exercise.teaching_cues.map((cue) => `  - ${cue}`).join('\n')}\n`;
            if (exercise.regression) md += `- Regression: ${exercise.regression}\n`;
            if (exercise.progression) md += `- Progression: ${exercise.progression}\n`;
            for (const note of exercise.safety_notes) md += `- Safety: ${note}\n`;
            md += `\n`;
        }
    }
    md += `## Safety Summary\n\n`;
    md += `- Risk policy: ${plan.safety_summary.risk_policy}\n`;
    if (plan.safety_summary.applied_contraindications.length) {
        md += `- Applied contraindications: ${plan.safety_summary.applied_contraindications.join(', ')}\n`;
    }
    for (const item of plan.safety_summary.excluded_exercises) md += `- Excluded ${item.name}: ${item.reason}\n`;
    for (const item of plan.safety_summary.modified_exercises) md += `- Modified ${item.name}: ${item.reason}\n`;
    for (const note of plan.safety_summary.safety_notes) md += `- ${note}\n`;
    md += `\n## After Class Retest\n\n`;
    for (const assessment of plan.retest.assessments) {
        md += `### ${assessment.name}\n\n${assessment.instruction}\n\n`;
    }
    md += `**Expected improvement:** ${plan.retest.expected_improvement}\n`;
    return md;
}
