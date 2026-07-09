CREATE TABLE movement_experiences (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE equipment (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL CHECK(category IN ('apparatus', 'movement_context'))
);

CREATE TABLE movement_systems (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE base_strategies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    movement_experience TEXT NOT NULL REFERENCES movement_experiences(id),
    primary_focus TEXT NOT NULL REFERENCES movement_systems(id),
    secondary_focus TEXT NOT NULL REFERENCES movement_systems(id),
    explanation_template TEXT NOT NULL
);

CREATE TABLE base_strategy_emphasis (
    strategy_id INTEGER NOT NULL REFERENCES base_strategies(id),
    system_name TEXT NOT NULL,
    emphasis_value INTEGER NOT NULL,
    PRIMARY KEY (strategy_id, system_name)
);

CREATE TABLE base_strategy_objectives (
    strategy_id INTEGER NOT NULL REFERENCES base_strategies(id),
    objective TEXT NOT NULL,
    PRIMARY KEY (strategy_id, objective)
);

CREATE TABLE observation_modifiers (
    id TEXT PRIMARY KEY,
    explanation_fragment TEXT NOT NULL
);

CREATE TABLE observation_modifier_keywords (
    modifier_id TEXT NOT NULL REFERENCES observation_modifiers(id),
    keyword TEXT NOT NULL,
    PRIMARY KEY (modifier_id, keyword)
);

CREATE TABLE observation_emphasis_adjustments (
    modifier_id TEXT NOT NULL REFERENCES observation_modifiers(id),
    system_name TEXT NOT NULL,
    adjustment INTEGER NOT NULL,
    PRIMARY KEY (modifier_id, system_name)
);

CREATE TABLE observation_added_objectives (
    modifier_id TEXT NOT NULL REFERENCES observation_modifiers(id),
    objective TEXT NOT NULL,
    PRIMARY KEY (modifier_id, objective)
);

CREATE TABLE observation_preferred_objectives (
    modifier_id TEXT NOT NULL REFERENCES observation_modifiers(id),
    objective TEXT NOT NULL,
    PRIMARY KEY (modifier_id, objective)
);

CREATE TABLE exercises (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    equipment TEXT NOT NULL REFERENCES equipment(id),
    min_level TEXT NOT NULL,
    max_level TEXT NOT NULL,
    difficulty INTEGER NOT NULL,
    default_duration_minutes INTEGER NOT NULL,
    regression TEXT,
    progression TEXT
);

CREATE TABLE exercise_roles (
    exercise_id TEXT NOT NULL REFERENCES exercises(id),
    role TEXT NOT NULL,
    PRIMARY KEY (exercise_id, role)
);

CREATE TABLE exercise_objectives (
    exercise_id TEXT NOT NULL REFERENCES exercises(id),
    objective TEXT NOT NULL,
    PRIMARY KEY (exercise_id, objective)
);

CREATE TABLE exercise_teaching_cues (
    exercise_id TEXT NOT NULL REFERENCES exercises(id),
    cue TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (exercise_id, cue)
);

CREATE TABLE exercise_contraindications (
    exercise_id TEXT NOT NULL REFERENCES exercises(id),
    tag TEXT NOT NULL,
    severity TEXT NOT NULL CHECK(severity IN ('HardExclude', 'Caution', 'RequireRegression')),
    note TEXT NOT NULL,
    PRIMARY KEY (exercise_id, tag)
);

CREATE TABLE benchmarks (
    id TEXT PRIMARY KEY,
    movement_experience TEXT NOT NULL REFERENCES movement_experiences(id),
    name TEXT NOT NULL,
    instruction TEXT NOT NULL
);

CREATE TABLE benchmark_watch_points (
    benchmark_id TEXT NOT NULL REFERENCES benchmarks(id),
    watch_point TEXT NOT NULL,
    PRIMARY KEY (benchmark_id, watch_point)
);
