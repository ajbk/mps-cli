CREATE TABLE flashcard_cards (
    id TEXT PRIMARY KEY,
    source_exercise_id TEXT NOT NULL,
    category TEXT NOT NULL,
    teaching_copy_json TEXT NOT NULL,
    style_profile TEXT NOT NULL CHECK(style_profile = 'mono-gesture-ink-pilates-v1'),
    character_id TEXT NOT NULL,
    current_asset_id TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL CHECK(status IN ('draft','generating','needs-review','revision-requested','approved','published')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE visual_briefs (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES flashcard_cards(id),
    exercise_id TEXT NOT NULL,
    brief_json TEXT NOT NULL,
    version INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE flashcard_assets (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES flashcard_cards(id),
    brief_id TEXT NOT NULL REFERENCES visual_briefs(id),
    repo_path TEXT NOT NULL,
    provider_job_id TEXT,
    version INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('generating','needs-review','rejected','approved')),
    created_at TEXT NOT NULL
);

CREATE TABLE flashcard_jobs (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES flashcard_cards(id),
    kind TEXT NOT NULL CHECK(kind IN ('generate','review','regenerate')),
    status TEXT NOT NULL CHECK(status IN ('queued','running','succeeded','failed')),
    input_json TEXT NOT NULL,
    output_json TEXT,
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE flashcard_reviews (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES flashcard_cards(id),
    asset_id TEXT NOT NULL REFERENCES flashcard_assets(id),
    passed INTEGER NOT NULL CHECK(passed IN (0,1)),
    findings_json TEXT NOT NULL,
    reviewer_kind TEXT NOT NULL CHECK(reviewer_kind IN ('automated','teacher')),
    reviewer_id TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE flashcard_audit_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id TEXT NOT NULL REFERENCES flashcard_cards(id),
    actor_kind TEXT NOT NULL CHECK(actor_kind IN ('teacher','chatgpt','system')),
    actor_id TEXT,
    action TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);
