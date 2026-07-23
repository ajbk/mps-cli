import assert from "node:assert/strict";
import test from "node:test";

import { createMcpHandler } from "../src/server.mjs";

const exercise = {
  exercise: { id: "source_mat_pelvic_clock_row_1", name: "Pelvic Clock", apparatus: "Mat", level: "Beginner" },
  apparatus: { name: "Mat", equipment_key: "mat" },
  families: [], categories: [], body_regions: [], movement_taxonomy: [], progressions: []
};

function handler(scopes = ["read_catalog", "write_draft", "generate_asset", "submit_review"]) {
  const calls = [];
  return {
    calls,
    handle: createMcpHandler({
      verifyBearerToken: async (token) => token === "valid" ? { studioId: "studio-01", teacherId: "teacher-01", scopes } : null,
      client: {
        searchExercises: async (query) => [{ id: "card-1", name: "Pelvic Clock", query }],
        getExerciseContext: async (id) => id === exercise.exercise.id ? exercise : null,
        getVisualContract: async () => ({ styleProfile: "mono-gesture-ink-pilates-v1", characterId: "teacher-01", outfit: "locked", cheekAccent: "#D98F9A", negativeConstraints: ["text"], reviewChecklist: ["identity"] }),
        createDraft: async (body) => { calls.push(["createDraft", body]); return { id: "card-1", ...body, status: "draft" }; },
        createVisualBrief: async (cardId, body) => { calls.push(["createVisualBrief", cardId, body]); return { id: "brief-1", cardId, ...body }; },
        createJob: async (cardId, body) => { calls.push(["createJob", cardId, body]); return { id: "job-1", cardId, ...body, status: "queued" }; },
        updateDraft: async (cardId, body) => { calls.push(["updateDraft", cardId, body]); return { id: cardId, ...body, status: "draft" }; },
        submitForReview: async (cardId) => { calls.push(["submitForReview", cardId]); return { id: cardId, status: "needs-review" }; }
      }
    })
  };
}

async function call(adapter, name, arguments_ = {}, token = "valid") {
  return adapter.handle({ method: "tools/call", params: { name, arguments: arguments_ } }, { authorization: `Bearer ${token}` });
}

test("search_exercises requires catalog scope and returns compact catalog data", async () => {
  const adapter = handler(["read_catalog"]);
  const result = await call(adapter, "search_exercises", { query: "pelvic" });
  assert.deepEqual(result.result, [{ id: "card-1", name: "Pelvic Clock", query: "pelvic" }]);
  const denied = await call(adapter, "search_exercises", { query: "pelvic" }, "missing");
  assert.equal(denied.error.code, "UNAUTHORIZED");
});

test("get_exercise_context rejects an unknown exercise ID", async () => {
  const adapter = handler();
  const result = await call(adapter, "get_exercise_context", { exercise_id: "unknown" });
  assert.equal(result.error.code, "INVALID_INPUT");
});

test("build_visual_brief locks the approved visual contract", async () => {
  const adapter = handler();
  const result = await call(adapter, "build_visual_brief", {
    card_id: "card-1", exercise_id: exercise.exercise.id, pose: { view: "side" }, style_profile: "other"
  });
  assert.equal(result.error.code, "INVALID_INPUT");
});

test("generate_flashcard_image rejects absolute paths and queues only generation jobs", async () => {
  const adapter = handler();
  const bad = await call(adapter, "generate_flashcard_image", { card_id: "card-1", brief_id: "/tmp/brief.json" });
  assert.equal(bad.error.code, "INVALID_INPUT");
  const good = await call(adapter, "generate_flashcard_image", { card_id: "card-1", brief_id: "brief-1" });
  assert.equal(good.result.kind, "generate");
});

test("review_flashcard_image submits a review job without approval or publication", async () => {
  const adapter = handler();
  const result = await call(adapter, "review_flashcard_image", { card_id: "card-1", asset_id: "asset-1" });
  assert.equal(result.result.kind, "review");
  assert.deepEqual(adapter.calls.at(-1), ["createJob", "card-1", { kind: "review", revision_notes: "asset_id:asset-1" }]);
});

test("save_flashcard_draft uses authenticated context and ignores caller studio or teacher IDs", async () => {
  const adapter = handler(["write_draft"]);
  const result = await call(adapter, "save_flashcard_draft", { card_id: "card-1", patch: { teaching_copy_json: { cue: "breathe" } }, studio_id: "other", teacher_id: "other" });
  assert.equal(result.result.status, "draft");
  assert.deepEqual(adapter.calls.at(-1), ["updateDraft", "card-1", { teaching_copy_json: { cue: "breathe" } }]);
});

test("submit_for_review requires its scope and publish requests are unavailable", async () => {
  const adapter = handler(["write_draft"]);
  const denied = await call(adapter, "submit_for_review", { card_id: "card-1" });
  assert.equal(denied.error.code, "FORBIDDEN");
  const publish = await call(adapter, "publish_flashcard", { card_id: "card-1" });
  assert.equal(publish.error.code, "METHOD_NOT_FOUND");
});
