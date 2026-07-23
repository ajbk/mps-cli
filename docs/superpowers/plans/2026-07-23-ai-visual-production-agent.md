# MPS AI Visual Production Agent Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an iOS-first PWA and MPS backend workflow where a teacher selects a canonical Pilates exercise, ChatGPT reads the exercise and visual rules through MCP, generates a versioned mono-gesture-ink image draft, validates it, and leaves publication to the teacher.

**Architecture:** Add a focused flashcard domain contract and persistence layer to the Rust workspace, then expose it through a small HTTP API. Keep the PWA as the teacher's library/editor/review workspace. Add a separate MCP adapter that authenticates ChatGPT with OAuth and calls the MPS API; image generation and visual QA run as backend jobs and never receive secrets in browser code.

**Tech Stack:** Rust 2021, serde, sqlx SQLite, Axum for the HTTP server, vanilla HTML/CSS/JavaScript PWA, Node.js MCP adapter with the official MCP SDK, OpenAI/image-provider calls only from server-side workers.

## Global Constraints

- The canonical exercise catalog is `data/reference/MPS_Database_v1_Core.xlsx`; `data/reference/mps_database_v1_core.json` is derived.
- The only active visual profile is `mono-gesture-ink-pilates-v1`.
- Character references and private source photos stay outside git; committed manifests use logical `private://` IDs and repo-relative public asset paths.
- The teacher's real proportions, shoulder-length layered bob, round dark glasses, exact outfit, and subtle dusty-rose cheek accent are locked.
- AI may draft, generate, review, and revise; AI may not mutate workbook identity, approve the character sheet, or publish a card.
- Existing deterministic class-plan generation remains unchanged and must not call an LLM.
- OpenAI/provider secrets never appear in `web/`, browser storage, MCP tool output, or committed fixtures.
- Every card and asset keeps an explicit version and audit trail.
- Every task ends with a focused test command and a small commit.
- Do not edit generated directories or unrelated student-studio behavior while implementing flashcards.

---

## File Map

| File or directory | Responsibility |
| --- | --- |
| `crates/mps-flashcards/` | Typed flashcard, visual brief, asset, review, and job domain contracts |
| `crates/mps-flashcards/src/catalog.rs` | Canonical exercise catalog loader over the derived workbook export |
| `crates/mps-db/migrations/0002_flashcards.sql` | SQLite tables and constraints for cards, briefs, assets, jobs, reviews, and audit events |
| `crates/mps-db/src/flashcard_repository.rs` | SQLx repository operations for the flashcard domain |
| `crates/mps-server/` | HTTP API and job orchestration; reuses `mps-core`, `mps-db`, and `mps-flashcards` |
| `services/mps-mcp/` | Custom ChatGPT App/MCP adapter, OAuth boundary, typed tools, and REST client |
| `services/visual-worker/` | Server-side image generation and deterministic/vision validation job adapter |
| `web/flashcard-store.js` | PWA repository adapter for static demo and HTTP-backed mode |
| `web/flashcard-model.js` | Pure filtering, status, and editor state functions used by the PWA |
| `web/index.html`, `web/app.js`, `web/styles.css` | Library-first, single-scroll editor, review queue, and mobile layout |
| `data/reference/pilates_visual_manifest.json` | Public visual-production manifest and card asset status |
| `docs/MPS_AI_VISUAL_PRODUCTION.md` | Operator guide for teacher review, OAuth connection, and failed jobs |

---

### Task 1: Add the shared flashcard domain contract

**Files:**
- Create: `crates/mps-flashcards/Cargo.toml`
- Create: `crates/mps-flashcards/src/lib.rs`
- Create: `crates/mps-flashcards/src/catalog.rs`
- Modify: `Cargo.toml`
- Test: `crates/mps-flashcards/src/lib.rs` unit tests

**Interfaces:**
- Consumes: canonical exercise IDs and visual manifest identifiers as strings.
- Produces: serializable domain types and `CanonicalCatalog::load_json` / `CanonicalCatalog::find` used by the database, HTTP API, PWA fixtures, and MCP adapter.

- [ ] **Step 1: Register the crate and write failing validation tests**

Add the crate to the workspace and define tests for:
- a valid draft with an existing exercise ID;
- rejection of an empty exercise ID;
- rejection of an invalid status transition from `published` back to `draft`;
- JSON round-trip for a visual brief containing the mono style profile and cheek accent.
- loading `data/reference/mps_database_v1_core.json` and finding a known source exercise by ID.

Run:

~~~bash
cargo test -p mps-flashcards
~~~

Expected: FAIL because the crate and types do not exist.

- [ ] **Step 2: Implement the minimal typed contract**

Use these exact conceptual types and add a catalog loader whose public interface is:

~~~rust
pub struct CanonicalCatalog {
    pub exercises: Vec<CatalogExercise>,
}
impl CanonicalCatalog {
    pub fn load_json(json: &str) -> Result<Self, CatalogError>;
    pub fn find(&self, exercise_id: &str) -> Option<&CatalogExercise>;
}
~~~

Use these exact conceptual card types:

~~~rust
pub enum FlashcardStatus {
    Draft,
    Generating,
    NeedsReview,
    RevisionRequested,
    Approved,
    Published,
}

pub struct FlashcardCard {
    pub id: String,
    pub source_exercise_id: String,
    pub category: String,
    pub teaching_copy_json: serde_json::Value,
    pub style_profile: String,
    pub character_id: String,
    pub current_asset_id: Option<String>,
    pub version: i64,
    pub status: FlashcardStatus,
}

pub struct VisualBrief {
    pub id: String,
    pub card_id: String,
    pub exercise_id: String,
    pub style_profile: String,
    pub character_id: String,
    pub outfit: String,
    pub pose_json: serde_json::Value,
    pub apparatus: String,
    pub palette_json: serde_json::Value,
    pub must_show_json: serde_json::Value,
    pub must_not_show_json: serde_json::Value,
    pub version: i64,
}

pub struct AssetReview {
    pub asset_id: String,
    pub passed: bool,
    pub findings_json: serde_json::Value,
    pub validator_version: String,
}
~~~

Implement `validate_new_draft`, `can_transition`, and serde derives. Reject any style profile other than `mono-gesture-ink-pilates-v1` in this MVP.

- [ ] **Step 3: Run the focused tests**

Run:

~~~bash
cargo test -p mps-flashcards
cargo fmt --all --check
~~~

Expected: all domain tests pass and formatting is clean.

- [ ] **Step 4: Commit**

~~~bash
git add Cargo.toml crates/mps-flashcards
git commit -m "feat: add flashcard domain contracts"
~~~

---

### Task 2: Persist cards, visual briefs, assets, jobs, reviews, and audit events

**Files:**
- Create: `crates/mps-db/migrations/0002_flashcards.sql`
- Create: `crates/mps-db/src/flashcard_repository.rs`
- Modify: `crates/mps-db/src/lib.rs`
- Modify: `crates/mps-db/Cargo.toml`
- Modify: `Cargo.lock`
- Test: `crates/mps-db/src/flashcard_repository.rs` tests

**Interfaces:**
- Consumes: `mps_flashcards::FlashcardCard`, `VisualBrief`, and status transitions.
- Produces: create/get/list/update methods used by the HTTP server.

- [ ] **Step 1: Write the migration and repository tests first**

Cover:
- creating a card from a valid source exercise ID;
- listing cards by status/category;
- storing a versioned brief and asset;
- rejecting a publish transition unless the latest review passed and the card is approved;
- recording an audit event for every write.

Run:

~~~bash
cargo test -p mps-db flashcard_repository
~~~

Expected: FAIL because the migration and repository methods do not exist.

- [ ] **Step 2: Add the migration**

Create these tables with foreign keys, timestamps, and checks:

~~~sql
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
~~~

- [ ] **Step 3: Implement repository methods**

Add explicit methods:
- `create_flashcard`
- `get_flashcard`
- `list_flashcards`
- `save_visual_brief`
- `create_job`
- `record_asset`
- `record_review`
- `transition_status`
- `record_audit_event`

All status transitions must call `mps_flashcards::can_transition` before SQL mutation.

- [ ] **Step 4: Run database tests**

Run:

~~~bash
cargo test -p mps-db
cargo test -p mps-core
~~~

Expected: existing generator tests remain green and new repository tests pass.

- [ ] **Step 5: Commit**

~~~bash
git add crates/mps-db crates/mps-flashcards Cargo.lock
git commit -m "feat: persist flashcard production state"
~~~

---

### Task 3: Extract a pure PWA flashcard model and repository adapter

**Files:**
- Create: `web/flashcard-model.js`
- Create: `web/flashcard-store.js`
- Create: `web/flashcard-model.test.mjs`
- Modify: `web/index.html`
- Modify: `web/app.js`

**Interfaces:**
- Consumes: `window.MPS_FLASHCARDS` and the REST API shape from Task 6.
- Produces: `filterCards(cards, filters)`, `statusLabel(status)`, `draftKey(cardId)`, `createLocalDraft(card)`, and store methods `listCards`, `getCard`, `saveDraft`.

- [ ] **Step 1: Write pure model tests**

Test:
- category and search filtering;
- level filtering;
- stable ID sort;
- draft status labels;
- local draft creation preserves source exercise fields and sets status to `draft`.

Run:

~~~bash
node --test web/flashcard-model.test.mjs
~~~

Expected: FAIL because the module does not exist.

- [ ] **Step 2: Implement the model and store**

Use a repository boundary:

~~~js
export function createFlashcardStore({
  apiBase = '',
  staticCards = [],
  storage = window.localStorage,
  fetchImpl = window.fetch.bind(window)
} = {}) {
  const request = async (path, options = {}) => {
    const response = await fetchImpl(`${apiBase}${path}`, options);
    if (!response.ok) throw new Error(`MPS API request failed: ${response.status}`);
    return response.json();
  };
  return {
    async listCards(filters = {}) {
      if (!apiBase) return filterCards(staticCards, filters);
      const query = new URLSearchParams(filters).toString();
      return request(`/api/flashcards?${query}`);
    },
    async getCard(id) {
      if (!apiBase) return JSON.parse(storage.getItem(`mps.flashcard.${id}`) || 'null');
      return request(`/api/flashcards/${encodeURIComponent(id)}`);
    },
    async saveDraft(card) {
      if (!apiBase) {
        storage.setItem(`mps.flashcard.${card.id}`, JSON.stringify(card));
        return card;
      }
      return request(`/api/flashcards/${encodeURIComponent(card.id)}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(card)
      });
    }
  };
}
~~~

The fallback must keep the current static demo usable when no backend is configured.

- [ ] **Step 3: Load the modules without changing class-plan behavior**

Add the model/store scripts after `flashcards-data.js`, then replace the existing inline filtering calls in `app.js` with the pure model functions. Do not change `mps-engine.js`.

- [ ] **Step 4: Run browser checks**

Run:

~~~bash
node --test web/flashcard-model.test.mjs
node --check web/flashcard-model.js
node --check web/flashcard-store.js
node --check web/app.js
~~~

Expected: all tests pass.

- [ ] **Step 5: Commit**

~~~bash
git add web/flashcard-model.js web/flashcard-store.js web/flashcard-model.test.mjs web/index.html web/app.js
git commit -m "refactor: add flashcard repository boundary"
~~~

---

### Task 4: Build the Library-first PWA screen

**Files:**
- Modify: `web/index.html:383-409`
- Modify: `web/app.js:97-115,235-242,794-880,1071-1086`
- Modify: `web/styles.css:703-803`
- Test: `web/flashcard-model.test.mjs`

**Interfaces:**
- Consumes: the store/model from Task 3.
- Produces: a responsive library with search, category tabs, level filters, card counts, and a “Create card” action.

- [ ] **Step 1: Add failing model coverage for the Library contract**

Add tests that the library returns:
- 160 static cards in demo mode;
- only matching category after selecting Mat;
- M02 appears for the search term `pelvic clock`;
- image URLs remain relative and do not contain machine-local paths.

- [ ] **Step 2: Replace the current deck-only markup**

The Flashcards view must contain:
- search field;
- category filters for Reformer, Mat, Stand, Chair;
- level filter;
- result count;
- card rows/cards with exercise name, source category, level, status, and thumbnail;
- a Create button that creates a local draft from the selected catalog record.

Keep the current print action available as a secondary action.

- [ ] **Step 3: Implement mobile layout**

Add a breakpoint that collapses the desktop sidebar, makes the library controls full width, and keeps each card row tappable at a minimum 44px target height. Preserve the current desktop layout for the client studio.

- [ ] **Step 4: Run UI checks**

Run:

~~~bash
node --test web/flashcard-model.test.mjs
node --check web/app.js
git diff --check
~~~

Expected: tests pass and no whitespace errors.

- [ ] **Step 5: Commit**

~~~bash
git add web/index.html web/app.js web/styles.css web/flashcard-model.test.mjs
git commit -m "feat: add library-first flashcard browser"
~~~

---

### Task 5: Build the single-scroll editor and teacher review queue

**Files:**
- Modify: `web/index.html`
- Modify: `web/app.js`
- Modify: `web/styles.css`
- Create: `web/flashcard-review.js`
- Test: `web/flashcard-review.test.mjs`

**Interfaces:**
- Consumes: a local or API-backed `FlashcardCard`.
- Produces: editor fields, AI-job status display, visual brief preview, validation findings, revision request, teacher approval, and publish guard.

- [ ] **Step 1: Write review-state tests**

Cover:
- a draft can move to generating;
- a generating card cannot be approved;
- a needs-review card with a failed automated review cannot be approved;
- a passed needs-review card can become approved;
- only an approved card can become published.

Run:

~~~bash
node --test web/flashcard-review.test.mjs
~~~

Expected: FAIL before the review state module exists.

- [ ] **Step 2: Implement the editor**

The single-scroll editor must show:
- workbook-locked source exercise, apparatus, level, and objective;
- editable front question, cue, regression, and progression;
- current style profile and character reference;
- visual brief JSON summary;
- image preview and asset version;
- Generate / Regenerate / Request revision / Submit review actions;
- a clear draft status badge.

AI-generated values must appear as proposed changes until the teacher saves them.

- [ ] **Step 3: Implement review and publish guards**

Use `flashcard-review.js` as a pure state transition module. The publish button must be disabled unless:
- status is `approved`;
- latest automated review passed;
- latest teacher review exists;
- source exercise and style profile are unchanged.

- [ ] **Step 4: Run browser checks**

Run:

~~~bash
node --test web/flashcard-review.test.mjs
node --check web/flashcard-review.js
node --check web/app.js
~~~

Expected: all state tests pass.

- [ ] **Step 5: Commit**

~~~bash
git add web/index.html web/app.js web/styles.css web/flashcard-review.js web/flashcard-review.test.mjs
git commit -m "feat: add flashcard editor and review queue"
~~~

---

### Task 6: Add the MPS flashcard HTTP API

**Files:**
- Create: `crates/mps-server/Cargo.toml`
- Create: `crates/mps-server/src/lib.rs`
- Create: `crates/mps-server/src/main.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/mps-db/src/flashcard_repository.rs`
- Test: `crates/mps-server/src/lib.rs` route tests

**Interfaces:**
- Consumes: authenticated teacher requests and repository methods from Task 2.
- Produces: JSON routes consumed by `web/flashcard-store.js` and `services/mps-mcp`.

Implement these routes:
- `GET /api/flashcards?query=&category=&level=&status=`
- `GET /api/catalog/exercises/:exercise_id/context`
- `GET /api/visual-contract`
- `GET /api/character-reference`
- `GET /api/flashcards/:id`
- `POST /api/flashcards`
- `PATCH /api/flashcards/:id`
- `POST /api/flashcards/:id/visual-briefs`
- `POST /api/flashcards/:id/jobs`
- `GET /api/flashcards/:id/jobs/:job_id`
- `POST /api/flashcards/:id/reviews`
- `POST /api/flashcards/:id/submit-review`
- `POST /api/flashcards/:id/approve`
- `POST /api/flashcards/:id/publish`

- [ ] **Step 1: Add route tests before handlers**

Use an in-memory SQLite repository and assert:
- list returns only cards scoped to the authenticated studio;
- POST rejects an unknown source exercise ID;
- PATCH cannot alter source exercise ID or style profile;
- publish returns 409 unless approval prerequisites pass;
- every mutation creates an audit row.

Run:

~~~bash
cargo test -p mps-server routes
~~~

Expected: FAIL until the server crate and handlers exist.

- [ ] **Step 2: Add Axum server, catalog loading, and DTOs**

Load `CanonicalCatalog` once at server startup from the configured `data/reference/mps_database_v1_core.json` path. The catalog context route must return only the selected source exercise and its related apparatus, family, category, body-region, movement-taxonomy, and progression records. Use request/response DTOs separate from database rows. Extract `studio_id`, `teacher_id`, and permission scopes from an auth context rather than accepting them in JSON bodies. Reuse `mps-core` only for existing deterministic plan behavior; do not add AI calls to `mps-core`.

- [ ] **Step 3: Add static-file compatibility**

Update `vercel.json` only after the API deployment shape is selected. Keep static `web/` hosting working locally, and make API URLs configurable through a single `MPS_API_BASE` value in the PWA.

- [ ] **Step 4: Run server checks**

Run:

~~~bash
cargo test -p mps-server
cargo test -p mps-db
cargo test -p mps-core
cargo fmt --all --check
~~~

Expected: all server, repository, and deterministic generator tests pass.

- [ ] **Step 5: Commit**

~~~bash
git add Cargo.toml Cargo.lock crates/mps-server crates/mps-db
git commit -m "feat: add flashcard HTTP API"
~~~

---

### Task 7: Add the Custom ChatGPT App/MCP adapter

**Files:**
- Create: `services/mps-mcp/package.json`
- Create: `services/mps-mcp/src/server.mjs`
- Create: `services/mps-mcp/src/mps-client.mjs`
- Create: `services/mps-mcp/src/auth.mjs`
- Create: `services/mps-mcp/test/tools.test.mjs`
- Create: `services/mps-mcp/.env.example`
- Create: `docs/MPS_CHATGPT_APP_SETUP.md`

**Interfaces:**
- Consumes: MPS HTTP API and OAuth bearer context.
- Produces: typed MCP tools that ChatGPT can call without receiving raw database files or private photos.

- [ ] **Step 1: Define MCP tool schemas and failing tests**

Test tool input validation for:
- `search_exercises`;
- `get_exercise_context`;
- `build_visual_brief`;
- `generate_flashcard_image`;
- `review_flashcard_image`;
- `save_flashcard_draft`.

Reject unknown exercise IDs, unsupported style profiles, absolute file paths, and publish requests.

Run:

~~~bash
cd services/mps-mcp
npm install
npm test
~~~

Expected: FAIL before the adapter exists.

- [ ] **Step 2: Implement the REST client and read-only tools**

The client must call the server routes from Task 6, return compact JSON, and strip private source data. `get_visual_contract` returns the active style ID, locked outfit, cheek accent, negative constraints, and review checklist.

- [ ] **Step 3: Implement draft-only write tools**

`save_flashcard_draft` and `submit_for_review` must require an authenticated teacher scope and write audit events as actor kind `chatgpt`. Do not expose a publish tool.

- [ ] **Step 4: Add OAuth boundary**

Implement the Custom App OAuth callback, token validation, and scope mapping:
- `read_catalog`;
- `write_draft`;
- `generate_asset`;
- `submit_review`.

Keep client secret and token encryption keys in environment variables. Document the callback URL, redirect URL, scopes, and ChatGPT app connection steps in `docs/MPS_CHATGPT_APP_SETUP.md`.

- [ ] **Step 5: Run adapter checks**

Run:

~~~bash
cd services/mps-mcp
npm test
node --check src/server.mjs
node --check src/mps-client.mjs
node --check src/auth.mjs
~~~

Expected: all tool and scope tests pass.

- [ ] **Step 6: Commit**

~~~bash
git add services/mps-mcp docs/MPS_CHATGPT_APP_SETUP.md
git commit -m "feat: add MPS ChatGPT MCP adapter"
~~~

---

### Task 8: Add server-side image generation and visual QA jobs

**Files:**
- Create: `services/visual-worker/package.json`
- Create: `services/visual-worker/src/worker.mjs`
- Create: `services/visual-worker/src/visual-contract.mjs`
- Create: `services/visual-worker/src/validator.mjs`
- Create: `services/visual-worker/test/validator.test.mjs`
- Modify: `crates/mps-server/src/lib.rs`
- Modify: `data/reference/pilates_visual_manifest.json`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: a validated `VisualBrief`, canonical public references, and provider credentials available only to the worker.
- Produces: versioned draft assets, automated findings, and job status updates.

- [ ] **Step 1: Write validator tests**

Cover:
- brief must use `mono-gesture-ink-pilates-v1`;
- brief must reference `teacher-01`;
- outfit must match the locked outfit;
- negative constraints include arrows, text, logos, and watermark;
- asset path must be repo-relative or an approved object-store key;
- failed visual checks produce `revision-requested`.

Run:

~~~bash
cd services/visual-worker
npm install
npm test
~~~

Expected: FAIL before validator modules exist.

- [ ] **Step 2: Implement brief compiler**

Build the final generation payload from:
- exercise context returned by MPS;
- visual style profile;
- canonical character sheet;
- required role references;
- pose landmarks and contact points;
- cheek accent palette;
- negative constraints.

Never compile a prompt directly from arbitrary ChatGPT text without validating the structured brief first.

- [ ] **Step 3: Implement the provider adapter**

Use an interface:

~~~js
export function createImageGenerator({ generate }) {
  return {
    async generateDraft({ brief, referenceAssets }) {
      return generate({ brief, referenceAssets });
    }
  };
}
~~~

Load credentials from the worker environment only. Store provider job IDs and outputs in the MPS job tables.

- [ ] **Step 4: Implement automated visual review**

Run metadata checks first, then image review checks for pose, anatomy, identity, glasses/hair, outfit, apparatus, linework, cheek accent, and forbidden overlays. Save findings as JSON and keep rejected assets out of the published deck.

- [ ] **Step 5: Connect jobs to the server**

Implement:
- `POST /api/flashcards/:id/jobs` queueing;
- `GET /api/flashcards/:id/jobs/:job_id` status;
- worker callbacks that update asset/review/status rows;
- retryable failure status with a human-readable error.

- [ ] **Step 6: Run worker and manifest checks**

Run:

~~~bash
cd services/visual-worker
npm test
node --check src/worker.mjs
node --check src/visual-contract.mjs
node --check src/validator.mjs
cd ../..
node .agents/skills/mps-pilates-visual-production/scripts/validate_visual_manifest.mjs data/reference/pilates_visual_manifest.json
node --test .agents/skills/mps-pilates-visual-production/scripts/validate_visual_manifest.test.mjs
~~~

Expected: worker tests and visual manifest tests pass.

- [ ] **Step 7: Commit**

~~~bash
git add services/visual-worker crates/mps-server data/reference/pilates_visual_manifest.json .gitignore
git commit -m "feat: add flashcard image generation and visual QA jobs"
~~~

---

### Task 9: Wire the PWA to the API and document teacher operation

**Files:**
- Modify: `web/flashcard-store.js`
- Modify: `web/app.js`
- Modify: `web/index.html`
- Modify: `web/styles.css`
- Create: `docs/MPS_AI_VISUAL_PRODUCTION.md`
- Test: `web/flashcard-model.test.mjs`, `web/flashcard-review.test.mjs`

**Interfaces:**
- Consumes: API responses, job status, and review findings.
- Produces: an end-to-end teacher workflow that still works in static demo mode.

- [ ] **Step 1: Add API-mode tests**

Test that:
- the store uses `GET /api/flashcards` when `MPS_API_BASE` is configured;
- local demo mode remains available when it is absent;
- a generate action creates a job and polls status;
- review findings render without unsafe HTML interpolation;
- publish is unavailable until the API returns approved state.

- [ ] **Step 2: Wire the editor actions**

Connect buttons to:
- create draft;
- save teaching copy;
- ask ChatGPT / show the MCP connection instruction;
- generate image;
- poll job;
- show validation findings;
- request revision;
- submit review;
- approve;
- publish.

The PWA must never call the image provider directly.

- [ ] **Step 3: Add teacher documentation**

Document:
- how to connect the MPS Custom App in ChatGPT;
- how to select a workbook exercise;
- how to ask ChatGPT for a visual brief;
- how to confirm generation;
- how to review pose, identity, outfit, mono style, cheek accent, and forbidden overlays;
- how to request a revision;
- why publish remains a teacher action.

- [ ] **Step 4: Run end-to-end static checks**

Run:

~~~bash
node --test web/flashcard-model.test.mjs
node --test web/flashcard-review.test.mjs
node --check web/flashcard-store.js
node --check web/app.js
git diff --check
~~~

Expected: all browser-side tests pass and static mode remains usable.

- [ ] **Step 5: Commit**

~~~bash
git add web docs/MPS_AI_VISUAL_PRODUCTION.md
git commit -m "feat: wire teacher flashcard production workflow"
~~~

---

### Task 10: Full verification and handoff

**Files:**
- Modify only if verification finds a concrete issue: the files from Tasks 1-9.
- Test: workspace Rust tests, JavaScript checks, MCP tests, worker tests, visual manifest tests.

- [ ] **Step 1: Run deterministic Rust regression**

~~~bash
cargo test -q
cargo fmt --all --check
~~~

Expected: all existing MPS generator tests and new flashcard tests pass.

- [ ] **Step 2: Run browser and service checks**

~~~bash
node --check web/seed-data.js
node --check web/mps-engine.js
node --check web/app.js
node --check web/flashcards-data.js
node --test web/flashcard-model.test.mjs
node --test web/flashcard-review.test.mjs
cd services/mps-mcp && npm test
cd ../visual-worker && npm test
~~~

Expected: all checks pass.

- [ ] **Step 3: Run visual process checks**

~~~bash
node .agents/skills/mps-pilates-visual-production/scripts/validate_visual_manifest.mjs data/reference/pilates_visual_manifest.json
node --test .agents/skills/mps-pilates-visual-production/scripts/validate_visual_manifest.test.mjs
git diff --check
~~~

Expected: manifest is valid, no machine-local paths are committed, and no whitespace errors exist.

- [ ] **Step 4: Perform the teacher acceptance path**

Using iPhone Safari:
1. open Library;
2. search Mat / Pelvic Clock;
3. create a draft;
4. open ChatGPT and connect MPS;
5. request a visual brief;
6. confirm generation;
7. inspect the generated draft;
8. request one revision;
9. approve only after pose, identity, outfit, mono style, cheek accent, and no-overlay checks pass;
10. publish from PWA.

- [ ] **Step 5: Handoff**

Record:
- deployed PWA URL;
- MCP app callback and connection instructions;
- worker environment variables by name only;
- database migration version;
- current character/style approval status;
- known provider limitations.

~~~

