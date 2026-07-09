// MPS Engine — Client-side deterministic class plan generator
// Ported from Rust mps-core

// ── Domain Types ──────────────────────────────────────────────

const LEVEL_NUMERIC = {
    'Beginner': 1,
    'BeginnerIntermediate': 2,
    'Intermediate': 3,
    'IntermediateAdvanced': 4,
    'Advanced': 5
};

const EQUIPMENT_DB_TO_JS = {
    'reformer': 'Reformer',
    'chair': 'Chair',
    'mat': 'Mat',
    'standing': 'Standing'
};

// ── Phase Allocation ──────────────────────────────────────────

function phaseAllocations(duration) {
    const templates = {
        45: [
            { phase: 'ARRIVE', minutes: 3 },
            { phase: 'PREPARE', minutes: 7 },
            { phase: 'BUILD', minutes: 15 },
            { phase: 'INTEGRATE', minutes: 8 },
            { phase: 'CHALLENGE', minutes: 5 },
            { phase: 'TRANSFER', minutes: 3 },
            { phase: 'RESET & RETEST', minutes: 4 }
        ],
        60: [
            { phase: 'ARRIVE', minutes: 5 },
            { phase: 'PREPARE', minutes: 10 },
            { phase: 'BUILD', minutes: 20 },
            { phase: 'INTEGRATE', minutes: 10 },
            { phase: 'CHALLENGE', minutes: 8 },
            { phase: 'TRANSFER', minutes: 4 },
            { phase: 'RESET & RETEST', minutes: 3 }
        ],
        75: [
            { phase: 'ARRIVE', minutes: 5 },
            { phase: 'PREPARE', minutes: 12 },
            { phase: 'BUILD', minutes: 25 },
            { phase: 'INTEGRATE', minutes: 13 },
            { phase: 'CHALLENGE', minutes: 10 },
            { phase: 'TRANSFER', minutes: 5 },
            { phase: 'RESET & RETEST', minutes: 5 }
        ]
    };
    const alloc = templates[duration];
    if (!alloc) throw new Error(`Unsupported duration: ${duration}`);
    return alloc;
}

// ── Strategy Building ─────────────────────────────────────────

function buildStrategy(base, modifiers) {
    const emphasis = { ...base.emphasis };
    const explanationParts = [base.explanation_template];
    const addedObjectives = [...(base.objectives || [])];
    const preferredObjectives = [];

    for (const mod of modifiers) {
        explanationParts.push(mod.explanation_fragment);
        for (const [sys, adj] of Object.entries(mod.emphasis || {})) {
            emphasis[sys] = (emphasis[sys] || 0) + adj;
        }
        for (const obj of (mod.added_objectives || [])) {
            if (!addedObjectives.includes(obj)) addedObjectives.push(obj);
        }
        for (const obj of (mod.preferred_objectives || [])) {
            if (!preferredObjectives.includes(obj)) preferredObjectives.push(obj);
        }
    }

    // Clamp negatives and normalize to 100
    for (const k in emphasis) {
        emphasis[k] = Math.max(0, emphasis[k]);
    }
    const total = Object.values(emphasis).reduce((a, b) => a + b, 0);
    if (total > 0) {
        for (const k in emphasis) {
            emphasis[k] = Math.round((emphasis[k] / total) * 100);
        }
    }

    // Fix rounding to sum to 100
    const currentTotal = Object.values(emphasis).reduce((a, b) => a + b, 0);
    if (currentTotal !== 100) {
        const maxKey = Object.keys(emphasis).reduce((a, b) => emphasis[a] > emphasis[b] ? a : b);
        emphasis[maxKey] += (100 - currentTotal);
    }

    // Sort by emphasis descending
    const sortedEmphasis = {};
    Object.entries(emphasis)
        .sort((a, b) => b[1] - a[1])
        .forEach(([k, v]) => sortedEmphasis[k] = v);

    return {
        primary_focus: base.primary_focus,
        secondary_focus: base.secondary_focus,
        emphasis: sortedEmphasis,
        explanation: explanationParts.join(' '),
        added_objectives: addedObjectives,
        preferred_exercise_objectives: preferredObjectives
    };
}

// ── Scoring ───────────────────────────────────────────────────

function difficultyFitScore(exerciseDifficulty, classLevel) {
    const diff = exerciseDifficulty;
    const level = LEVEL_NUMERIC[classLevel];
    const delta = diff - level;
    if (delta <= -1) return 3;
    if (delta === 0) return 5;
    if (delta === 1) return 2;
    return 0;
}

function roleMatchScore(roles, targetRole) {
    return roles.includes(targetRole) ? 20 : 0;
}

function objectiveMatchScore(exerciseObjectives, preferred) {
    return exerciseObjectives.filter(obj =>
        preferred.some(p => p.toLowerCase() === obj.toLowerCase())
    ).length * 5;
}

// ── Phase Role Mapping ────────────────────────────────────────

const PHASE_ROLES = {
    'ARRIVE': 'Assess',
    'PREPARE': 'Prepare',
    'BUILD': 'Prime',
    'INTEGRATE': 'Integrate',
    'CHALLENGE': 'Challenge',
    'TRANSFER': 'Transfer',
    'RESET & RETEST': 'Restore'
};

const PHASE_PURPOSES = {
    'ARRIVE': 'Assess baseline and prepare the body',
    'PREPARE': 'Warm up and mobilize target systems',
    'BUILD': 'Build strength and movement capacity',
    'INTEGRATE': 'Connect movements across systems',
    'CHALLENGE': 'Push 80% success / 20% challenge',
    'TRANSFER': 'Transfer to functional movement',
    'RESET & RETEST': 'Retest and confirm improvement'
};

// ── Main Generator ────────────────────────────────────────────

function generateClassPlan(request) {
    // Validate
    if (!request.movement_experience) throw new Error('movement_experience required');
    if (!request.duration_minutes) throw new Error('duration_minutes required');
    if (!request.level) throw new Error('level required');

    const allocations = phaseAllocations(request.duration_minutes);

    // Find base strategy
    const experienceMap = {
        'ShoulderFreedom': 'shoulder_freedom',
        'HappyHips': 'happy_hips',
        'SpineReset': 'spine_reset'
    };
    const expId = experienceMap[request.movement_experience];
    const base = SEED_DATA.strategies.find(s => s.movement_experience === expId);
    if (!base) throw new Error(`No strategy for ${request.movement_experience}`);

    // Match observation modifiers
    const observations = (request.observations || []).map(o => o.toLowerCase());
    const matchedModifiers = SEED_DATA.modifiers.filter(mod =>
        mod.keywords.some(kw => observations.some(obs => obs.includes(kw.toLowerCase())))
    );

    // Build strategy
    const strategy = buildStrategy(base, matchedModifiers);

    // Get benchmarks
    const benchmarks = SEED_DATA.benchmarks.filter(b => b.movement_experience === expId);

    // Get all exercises (convert DB enum strings)
    const allExercises = SEED_DATA.exercises.map(e => ({
        ...e,
        equipment: EQUIPMENT_DB_TO_JS[e.equipment] || e.equipment,
        contraindications: e.contraindications || []
    }));

    // Safety filter
    const contra = (request.group_safety?.contraindications || []).map(c => c.toLowerCase());
    const safeExercises = allExercises.filter(ex =>
        !ex.contraindications.some(c =>
            c.severity === 'HardExclude' && contra.includes(c.tag.toLowerCase())
        )
    );

    // Build journey phases
    const journey = [];
    const warnings = [];

    for (const alloc of allocations) {
        const targetRole = PHASE_ROLES[alloc.phase];
        const purpose = PHASE_PURPOSES[alloc.phase];

        // Score and select
        let scored = safeExercises
            .filter(ex => ex.roles.includes(targetRole))
            .filter(ex => {
                const levelNum = LEVEL_NUMERIC[request.level];
                const minNum = LEVEL_NUMERIC[ex.min_level];
                const maxNum = LEVEL_NUMERIC[ex.max_level];
                return minNum <= levelNum && levelNum <= maxNum;
            })
            .map(ex => ({
                ex,
                score: roleMatchScore(ex.roles, targetRole)
                    + difficultyFitScore(ex.difficulty, request.level)
                    + objectiveMatchScore(ex.objectives, strategy.preferred_exercise_objectives)
                    + ((request.equipment || []).includes(ex.equipment) ? 5 : 0)
            }))
            .sort((a, b) => b.score - a.score);

        const phaseExercises = [];
        let remaining = alloc.minutes;

        for (const { ex } of scored) {
            if (remaining === 0) break;
            const dur = Math.min(ex.default_duration_minutes, remaining);
            phaseExercises.push({
                exercise_id: ex.id,
                name: ex.name,
                apparatus: ex.equipment,
                role: targetRole,
                duration_minutes: dur,
                movement_objectives: ex.objectives,
                why_selected: `Score-based selection for ${alloc.phase} phase`,
                teaching_cues: ex.teaching_cues,
                regression: ex.regression || null,
                progression: ex.progression || null,
                safety_notes: ex.contraindications
                    .filter(c => c.severity === 'Caution' && contra.includes(c.tag.toLowerCase()))
                    .map(c => c.note)
            });
            remaining -= dur;
        }

        if (phaseExercises.length === 0 && alloc.phase === 'BUILD') {
            throw new Error(`No candidates for BUILD phase with role ${targetRole}`);
        }

        if (phaseExercises.length === 0) {
            warnings.push(`No exercises found for ${alloc.phase} phase`);
        }

        journey.push({
            phase: alloc.phase,
            purpose,
            target_duration_minutes: alloc.minutes,
            exercises: phaseExercises
        });
    }

    // Safety summary
    const excludedExercises = allExercises.filter(ex =>
        ex.contraindications.some(c =>
            c.severity === 'HardExclude' && contra.includes(c.tag.toLowerCase())
        )
    ).map(ex => ({
        exercise_id: ex.id,
        name: ex.name,
        reason: ex.contraindications
            .filter(c => c.severity === 'HardExclude')
            .map(c => c.note).join('; ')
    }));

    const benchmarkPlan = {
        assessments: benchmarks.map(b => ({
            name: b.name,
            instruction: b.instruction,
            what_to_watch: b.watch_points
        }))
    };

    const classTitle = `${request.movement_experience} — ${request.duration_minutes} min ${request.level}`;

    return {
        class_title: classTitle,
        movement_experience: request.movement_experience,
        duration_minutes: request.duration_minutes,
        level: request.level,
        students: request.students || 1,
        equipment: request.equipment || [],
        movement_strategy: strategy,
        benchmark: benchmarkPlan,
        journey,
        safety_summary: {
            risk_policy: request.group_safety?.risk_policy || 'Balanced',
            applied_contraindications: contra,
            excluded_exercises: excludedExercises,
            modified_exercises: [],
            safety_notes: [`Group safety policy: ${request.group_safety?.risk_policy || 'Balanced'}`]
        },
        retest: {
            assessments: benchmarkPlan.assessments,
            expected_improvement: `After this ${request.movement_experience} class, expect improved movement quality in the target systems.`
        },
        expected_improvement: `Improved ${request.movement_experience} movement quality after class.`,
        warnings
    };
}

// ── Markdown Renderer ─────────────────────────────────────────

function renderMarkdown(plan) {
    let md = '';
    md += `# ${plan.class_title}\n`;
    md += `## Total Body Pilates Class\n\n`;
    md += `**Duration:** ${plan.duration_minutes} min  \n`;
    md += `**Level:** ${plan.level}  \n`;
    md += `**Students:** ${plan.students}  \n`;
    md += `**Equipment:** [${plan.equipment.join(', ')}]\n\n`;
    md += `---\n\n`;

    // Strategy
    md += `## Movement Strategy\n\n`;
    md += `**Primary Focus:** ${plan.movement_strategy.primary_focus}  \n`;
    md += `**Secondary Focus:** ${plan.movement_strategy.secondary_focus}\n\n`;
    md += `| System | Emphasis |\n|---|---:|\n`;
    for (const [sys, val] of Object.entries(plan.movement_strategy.emphasis)) {
        md += `| ${sys} | ${val}% |\n`;
    }
    md += `\n**Why this strategy:**  \n${plan.movement_strategy.explanation}\n\n`;
    md += `---\n\n`;

    // Benchmarks
    md += `## Before Class Benchmark\n\n`;
    for (const a of plan.benchmark.assessments) {
        md += `### ${a.name}\n\n${a.instruction}\n`;
        md += `**Watch for:**\n`;
        for (const w of a.what_to_watch) {
            md += `- ${w}\n`;
        }
        md += `\n`;
    }
    md += `---\n\n`;

    // Journey
    md += `## Class Journey\n\n`;
    for (const phase of plan.journey) {
        md += `### ${phase.phase} — ${phase.target_duration_minutes} min\n\n`;
        md += `${phase.purpose}\n\n`;
        for (const ex of phase.exercises) {
            md += `**${ex.name}** | ${ex.apparatus} | ${ex.duration_minutes} min | Role: ${ex.role}\n\n`;
            md += `- Objectives: ${ex.movement_objectives.join(', ')}\n`;
            md += `- Why selected: ${ex.why_selected}\n`;
            md += `- Cues:\n`;
            for (const cue of ex.teaching_cues) {
                md += `  - ${cue}\n`;
            }
            if (ex.regression) md += `- Regression: ${ex.regression}\n`;
            if (ex.progression) md += `- Progression: ${ex.progression}\n`;
            for (const note of ex.safety_notes) {
                md += `- ⚠️ ${note}\n`;
            }
            md += `\n`;
        }
        if (phase.exercises.length === 0) {
            md += `*No exercises selected for this phase*\n\n`;
        }
    }
    md += `---\n\n`;

    // Safety
    md += `## Safety Notes\n\n`;
    md += `**Risk Policy:** ${plan.safety_summary.risk_policy}\n\n`;
    if (plan.safety_summary.applied_contraindications.length > 0) {
        md += `**Applied group contraindications:**\n`;
        for (const c of plan.safety_summary.applied_contraindications) {
            md += `- ${c}\n`;
        }
    }
    if (plan.safety_summary.excluded_exercises.length > 0) {
        md += `\n**Excluded exercises:**\n`;
        for (const ex of plan.safety_summary.excluded_exercises) {
            md += `- ${ex.name}: ${ex.reason}\n`;
        }
    }
    md += `\n`;
    for (const note of plan.safety_summary.safety_notes) {
        md += `- ${note}\n`;
    }
    md += `\n---\n\n`;

    // Retest
    md += `## After Class Retest\n\n`;
    for (const a of plan.retest.assessments) {
        md += `### ${a.name}\n\n${a.instruction}\n\n`;
    }
    md += `**Expected improvement:**  \n${plan.retest.expected_improvement}\n`;

    return md;
}
