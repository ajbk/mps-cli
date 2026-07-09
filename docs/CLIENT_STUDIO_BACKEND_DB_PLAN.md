# MPS Client Studio Backend and DB Plan

This document defines the backend shape for the Client Studio web app. The current MVP stores data in `localStorage`; the backend should preserve the same domain objects so migration is direct.

## Product Modules

1. Dashboard
   - Reads student count, caution count, session count, average class result, latest session, and next plan.
2. Students
   - Stores profile, goals, lifestyle, class type, archetypes, pain/caution, movement pattern, and red flags.
3. Assessment
   - Stores dated posture notes, movement tests, quality scores, and teacher notes.
4. Session Log
   - Stores each private class record and the generated MPS plan snapshot.
5. Progress
   - Reads session timeline, result trend, repeated themes, and latest next plan.
6. Playbook Link
   - Maps student archetypes to recommended playbooks.
7. Student Report
   - Builds a report from student profile, assessments, sessions, and generated plan history.

## Suggested Backend Shape

Keep the existing Rust workspace and add a server crate later:

- `crates/mps-server`
  - HTTP framework: Axum or Actix Web.
  - Storage: SQLite for local studio installs; Postgres-compatible schema when hosted.
  - Reuse `mps-core` for deterministic plan generation.
  - Reuse `mps-db` for repository patterns and migrations.

Suggested layers:

- `api`: request/response DTOs and route handlers.
- `service`: orchestration, validation, report building, generated-plan snapshots.
- `repository`: SQL queries and transactions.
- `auth`: studio/user identity, API tokens, and role checks.

## API Draft

All mutating endpoints should be scoped by `studio_id` from auth context, not supplied by the client.

### Students

- `GET /api/students`
- `POST /api/students`
- `GET /api/students/{student_id}`
- `PATCH /api/students/{student_id}`
- `DELETE /api/students/{student_id}`

### Assessments

- `GET /api/students/{student_id}/assessments`
- `POST /api/students/{student_id}/assessments`
- `PATCH /api/assessments/{assessment_id}`
- `DELETE /api/assessments/{assessment_id}`

### Sessions

- `GET /api/students/{student_id}/sessions`
- `POST /api/students/{student_id}/sessions`
- `PATCH /api/sessions/{session_id}`
- `DELETE /api/sessions/{session_id}`

### Plans

- `POST /api/plans/generate`
  - Input: current class request or `{ student_id, duration_minutes, level, equipment, movement_experience }`
  - Output: deterministic plan JSON and markdown.
- `POST /api/sessions/{session_id}/plans`
  - Regenerates and stores a snapshot on a session.

### Playbooks and Reports

- `GET /api/playbooks`
- `GET /api/students/{student_id}/recommendation`
- `GET /api/students/{student_id}/report`

### Import/Export

- `GET /api/export`
- `POST /api/import`

## DB Entities

Core tables:

- `studios`
- `users`
- `students`
- `student_archetypes`
- `student_red_flags`
- `assessments`
- `session_logs`
- `session_equipment`
- `playbooks`
- `playbook_archetypes`
- `sync_outbox`

Important design choices:

- Store generated plans as immutable JSON snapshots on `session_logs.generated_plan_json`.
- Store generated markdown on `session_logs.generated_markdown` for fast copy/export.
- Keep archetypes and red flags as join tables instead of comma strings.
- Use soft archive for students later; the MVP delete can become `archived_at`.
- Add `updated_at` to every user-owned table for sync and conflict resolution.

## LocalStorage Migration

The current web MVP stores:

```json
{
  "students": [],
  "assessments": [],
  "sessions": [],
  "selectedStudentId": "..."
}
```

Migration path:

1. Export JSON from the web app.
2. `POST /api/import` validates arrays and IDs.
3. Insert students first.
4. Insert `student_archetypes` and `student_red_flags`.
5. Insert assessments.
6. Insert session logs and `session_equipment`.
7. Preserve generated plan snapshots when present.

## Security and Privacy

Student movement notes can contain health-related information. Treat them as sensitive studio records.

- Require authentication before any API access.
- Scope every row by `studio_id`.
- Log exports and imports.
- Avoid sending reports to third-party services without explicit user action.
- Encrypt backups at rest when hosted.

## First Backend Milestone

1. Add `crates/mps-server`.
2. Implement SQLite migrations from `docs/client_studio_schema.sql`.
3. Add CRUD for students, assessments, sessions.
4. Add `/api/plans/generate` using `mps-core`.
5. Add import/export matching the current localStorage JSON.
6. Switch the static web app from localStorage repository to HTTP repository behind a small adapter.
