import assert from "node:assert/strict";
import http from "node:http";
import test from "node:test";

import { createOAuthVerifier } from "../src/auth.mjs";
import { createMpsClient } from "../src/mps-client.mjs";
import { createHttpServer, createMcpHandler } from "../src/server.mjs";

const exercise = { exercise: { id: "source_mat_pelvic_clock_row_1", name: "Pelvic Clock", apparatus: "Mat", level: "Beginner" }, apparatus: { name: "Mat", equipment_key: "mat" }, families: [], categories: [], body_regions: [], movement_taxonomy: [], progressions: [] };

function adapter(scopes = ["read_catalog", "write_draft", "generate_asset", "submit_review"]) {
  const calls = [];
  return { calls, handle: createMcpHandler({
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
  }) };
}

async function call(instance, name, arguments_ = {}, token = "valid") {
  return instance.handle({ method: "tools/call", params: { name, arguments: arguments_ } }, { authorization: `Bearer ${token}` });
}

test("tools/list returns MCP tool definitions with object input schemas", async () => {
  const result = await adapter().handle({ method: "tools/list" });
  assert.ok(Array.isArray(result.result.tools));
  for (const tool of result.result.tools) assert.deepEqual(tool.inputSchema.type, "object");
});

test("tools/call returns a CallToolResult with structured content", async () => {
  const result = await call(adapter(["read_catalog"]), "search_exercises", { query: "pelvic" });
  assert.equal(result.result.isError, false);
  assert.deepEqual(result.result.structuredContent, [{ id: "card-1", name: "Pelvic Clock", query: "pelvic" }]);
  assert.equal(result.result.content[0].type, "text");
});

test("tool validation returns an MCP error result while unknown methods use JSON-RPC codes", async () => {
  const instance = adapter();
  const invalid = await call(instance, "get_exercise_context", { exercise_id: "unknown" });
  assert.equal(invalid.result.isError, true);
  assert.match(invalid.result.content[0].text, /unknown exercise ID/);
  const missing = await call(instance, "publish_flashcard", { card_id: "card-1" });
  assert.equal(missing.error.code, -32601);
});

test("write tools do not require read_catalog and preserve locked visual contract", async () => {
  const instance = adapter(["write_draft"]);
  const draft = await call(instance, "save_flashcard_draft", { card_id: "card-1", patch: { teaching_copy_json: { cue: "breathe" } }, studio_id: "other" });
  assert.equal(draft.result.isError, false);
  assert.deepEqual(instance.calls.at(-1), ["updateDraft", "card-1", { teaching_copy_json: { cue: "breathe" } }]);
  const locked = await call(instance, "build_visual_brief", { card_id: "card-1", exercise_id: exercise.exercise.id, pose: {}, style_profile: "other" });
  assert.equal(locked.result.isError, true);
});

test("generation rejects absolute paths and review cannot publish", async () => {
  const instance = adapter(["generate_asset", "submit_review"]);
  const invalid = await call(instance, "generate_flashcard_image", { card_id: "card-1", brief_id: "/tmp/brief.json" });
  assert.equal(invalid.result.isError, true);
  const review = await call(instance, "review_flashcard_image", { card_id: "card-1", asset_id: "asset-1" });
  assert.equal(review.result.structuredContent.kind, "review");
});

test("introspection requires active issuer audience expiry identity and supported scopes", async () => {
  const env = { MPS_OAUTH_INTROSPECTION_URL: "https://issuer.test/introspect", MPS_OAUTH_CLIENT_ID: "id", MPS_OAUTH_CLIENT_SECRET: "secret", MPS_OAUTH_ISSUER: "https://issuer.test", MPS_RESOURCE_URL: "https://mcp.test/mcp" };
  const verifier = createOAuthVerifier({ env, fetchImpl: async () => new Response(JSON.stringify({ active: true, iss: env.MPS_OAUTH_ISSUER, aud: [env.MPS_RESOURCE_URL], exp: Math.floor(Date.now() / 1000) + 60, studio_id: "studio-01", teacher_id: "teacher-01", scope: "write_draft" })) });
  assert.deepEqual(await verifier("user-token"), { studioId: "studio-01", teacherId: "teacher-01", scopes: ["write_draft"] });
  const expired = createOAuthVerifier({ env, fetchImpl: async () => new Response(JSON.stringify({ active: true, iss: env.MPS_OAUTH_ISSUER, aud: env.MPS_RESOURCE_URL, exp: 1, studio_id: "studio-01", teacher_id: "teacher-01", scope: "write_draft" })) });
  assert.equal(await expired("user-token"), null);
});

test("MPS client uses a server-only service token and verified identity headers", async () => {
  let request;
  const client = createMpsClient({ baseUrl: "https://api.test", serviceToken: "service-token", identity: { studioId: "studio-01", teacherId: "teacher-01" }, fetchImpl: async (_url, options) => { request = options; return new Response("{}", { status: 200 }); } });
  await client.submitForReview("card-1");
  assert.equal(request.headers.authorization, "Bearer service-token");
  assert.equal(request.headers["x-mps-studio-id"], "studio-01");
  assert.equal(request.headers["x-mps-teacher-id"], "teacher-01");
  assert.equal(request.headers["x-mps-actor-kind"], "chatgpt");
});

test("missing bearer receives a protected-resource Bearer challenge", async () => {
  const env = { MPS_API_BASE_URL: "https://api.test", MPS_API_SERVICE_TOKEN: "service-token", MPS_OAUTH_INTROSPECTION_URL: "https://issuer.test/introspect", MPS_OAUTH_CLIENT_ID: "id", MPS_OAUTH_CLIENT_SECRET: "secret", MPS_OAUTH_ISSUER: "https://issuer.test", MPS_RESOURCE_URL: "http://127.0.0.1/mcp" };
  const server = createHttpServer({ env, fetchImpl: async () => new Response("{}", { status: 401 }) });
  await new Promise((resolve) => server.listen(0, resolve));
  const { port } = server.address();
  const response = await new Promise((resolve, reject) => http.request({ host: "127.0.0.1", port, method: "POST", path: "/mcp" }, (res) => resolve(res)).on("error", reject).end(JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: "search_exercises", arguments: { query: "pelvic" } } })));
  assert.equal(response.statusCode, 401);
  assert.match(response.headers["www-authenticate"], /resource_metadata=/);
  await new Promise((resolve) => server.close(resolve));
});
