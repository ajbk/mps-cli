-- Draft schema for MPS Client Studio.
-- SQLite-compatible; keep column names close to the web MVP data model.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS studios (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    studio_id TEXT NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'owner',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (studio_id, email)
);

CREATE TABLE IF NOT EXISTS students (
    id TEXT PRIMARY KEY,
    studio_id TEXT NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    age INTEGER,
    occupation TEXT,
    start_date TEXT,
    class_type TEXT NOT NULL DEFAULT 'Private',
    frequency TEXT,
    goal TEXT,
    lifestyle TEXT,
    pain_area TEXT,
    pain_level INTEGER CHECK (pain_level IS NULL OR (pain_level >= 0 AND pain_level <= 10)),
    status TEXT NOT NULL DEFAULT 'Active',
    caution TEXT,
    pattern TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    archived_at TEXT
);

CREATE TABLE IF NOT EXISTS student_archetypes (
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    archetype TEXT NOT NULL,
    PRIMARY KEY (student_id, archetype)
);

CREATE TABLE IF NOT EXISTS student_red_flags (
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    red_flag TEXT NOT NULL,
    PRIMARY KEY (student_id, red_flag)
);

CREATE TABLE IF NOT EXISTS assessments (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    assessment_date TEXT NOT NULL,
    focus TEXT,
    posture TEXT,
    tests TEXT,
    control_score INTEGER CHECK (control_score IS NULL OR (control_score >= 1 AND control_score <= 5)),
    mobility_score INTEGER CHECK (mobility_score IS NULL OR (mobility_score >= 1 AND mobility_score <= 5)),
    stability_score INTEGER CHECK (stability_score IS NULL OR (stability_score >= 1 AND stability_score <= 5)),
    breath_score INTEGER CHECK (breath_score IS NULL OR (breath_score >= 1 AND breath_score <= 5)),
    awareness_score INTEGER CHECK (awareness_score IS NULL OR (awareness_score >= 1 AND awareness_score <= 5)),
    endurance_score INTEGER CHECK (endurance_score IS NULL OR (endurance_score >= 1 AND endurance_score <= 5)),
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS session_logs (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    session_date TEXT NOT NULL,
    session_no INTEGER,
    duration_minutes INTEGER,
    energy_score INTEGER CHECK (energy_score IS NULL OR (energy_score >= 1 AND energy_score <= 5)),
    pain_before INTEGER CHECK (pain_before IS NULL OR (pain_before >= 0 AND pain_before <= 10)),
    pain_after INTEGER CHECK (pain_after IS NULL OR (pain_after >= 0 AND pain_after <= 10)),
    level TEXT,
    movement_experience TEXT,
    theme TEXT,
    playbook_name TEXT,
    experience_note TEXT,
    before_test TEXT,
    exercises_text TEXT,
    easy TEXT,
    hard TEXT,
    compensation TEXT,
    cue TEXT,
    after_test TEXT,
    result_percent INTEGER CHECK (result_percent IS NULL OR (result_percent >= 0 AND result_percent <= 100)),
    feeling TEXT,
    next_plan TEXT,
    generated_plan_json TEXT,
    generated_markdown TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS session_equipment (
    session_id TEXT NOT NULL REFERENCES session_logs(id) ON DELETE CASCADE,
    equipment TEXT NOT NULL,
    PRIMARY KEY (session_id, equipment)
);

CREATE TABLE IF NOT EXISTS playbooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    movement_experience TEXT NOT NULL,
    tests TEXT NOT NULL,
    exercises TEXT NOT NULL,
    cautions TEXT NOT NULL,
    experience TEXT NOT NULL,
    observations_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS playbook_archetypes (
    playbook_id TEXT NOT NULL REFERENCES playbooks(id) ON DELETE CASCADE,
    archetype TEXT NOT NULL,
    PRIMARY KEY (playbook_id, archetype)
);

CREATE TABLE IF NOT EXISTS sync_outbox (
    id TEXT PRIMARY KEY,
    studio_id TEXT NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    sent_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_students_studio_status ON students(studio_id, status);
CREATE INDEX IF NOT EXISTS idx_assessments_student_date ON assessments(student_id, assessment_date DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_student_date ON session_logs(student_id, session_date DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_theme ON session_logs(theme);
CREATE INDEX IF NOT EXISTS idx_sync_outbox_unsent ON sync_outbox(studio_id, sent_at);
