import http from "node:http";
import { bearerFromHeaders, createOAuthVerifier, JSON_RPC, requiredScope, rpcError } from "./auth.mjs";
import { assertSafeIdentifier, createMpsClient, sanitize } from "./mps-client.mjs";

const STYLE = "mono-gesture-ink-pilates-v1"; const CHARACTER = "teacher-01"; const CHEEK = "#D98F9A";
const object = (properties, required = []) => ({ type: "object", properties, required, additionalProperties: false });
const str = { type: "string", minLength: 1 };
export const tools = [
  ["search_exercises", "read_catalog", object({ query: str, apparatus: str, level: str }, ["query"])], ["get_exercise_context", "read_catalog", object({ exercise_id: str }, ["exercise_id"])], ["get_visual_contract", "read_catalog", object({})], ["create_flashcard_draft", "write_draft", object({ exercise_id: str, teaching_copy_json: { type: "object" } }, ["exercise_id"])], ["build_visual_brief", "write_draft", object({ card_id: str, exercise_id: str, pose: { type: "object" }, style_profile: str, character_id: str, cheek_accent: str }, ["card_id", "exercise_id", "pose"])], ["generate_flashcard_image", "generate_asset", object({ card_id: str, brief_id: str }, ["card_id", "brief_id"])], ["review_flashcard_image", "submit_review", object({ card_id: str, asset_id: str }, ["card_id", "asset_id"])], ["save_flashcard_draft", "write_draft", object({ card_id: str, patch: object({ category: str, teaching_copy_json: { type: "object" } }) }, ["card_id", "patch"])], ["submit_for_review", "submit_review", object({ card_id: str }, ["card_id"])]
].map(([name, scope, inputSchema]) => ({ name, description: `MPS flashcard ${name.replaceAll("_", " ")}.`, inputSchema, scope }));
const scopeFor = (name) => tools.find((tool) => tool.name === name)?.scope;
const toolResult = (value, isError = false) => ({ content: [{ type: "text", text: JSON.stringify(value) }], isError, ...(isError ? {} : { structuredContent: value }) });
function input(value, field) { return assertSafeIdentifier(value, field); } function locked(value, expected, field) { if (value !== undefined && value !== expected) throw new Error(`${field} is locked to ${expected}`); }

export function createMcpHandler({ verifyBearerToken, client, clientFactory } = {}) {
  if (!verifyBearerToken) throw new Error("verifyBearerToken is required");
  return async (request, headers = {}) => {
    try {
      if (request?.method === "initialize") return { result: { protocolVersion: "2025-03-26", capabilities: { tools: {} }, serverInfo: { name: "mps-flashcard-mcp", version: "0.1.1" } } };
      if (request?.method === "tools/list") return { result: { tools } };
      if (request?.method !== "tools/call") throw rpcError(JSON_RPC.METHOD_NOT_FOUND, "method not found");
      const name = request.params?.name; const scope = scopeFor(name); if (!scope) throw rpcError(JSON_RPC.METHOD_NOT_FOUND, "tool not found");
      const token = bearerFromHeaders(headers); const identity = await verifyBearerToken(token); if (!identity) throw rpcError(JSON_RPC.UNAUTHORIZED, "invalid bearer token"); requiredScope(identity, scope);
      const api = client ?? clientFactory?.(identity); if (!api) throw new Error("MPS API client is required");
      try { return { result: toolResult(sanitize(await execute(name, request.params?.arguments ?? {}, api))) }; } catch (error) { return { result: toolResult({ error: error.message }, true) }; }
    } catch (error) { return { error: { code: Number.isInteger(error.code) ? error.code : JSON_RPC.INTERNAL_ERROR, message: error.message } }; }
  };
}

async function execute(name, args, api) { switch (name) {
  case "search_exercises": return api.searchExercises(String(args.query ?? "").trim(), args.apparatus, args.level);
  case "get_exercise_context": { const result = await api.getExerciseContext(input(args.exercise_id, "exercise_id")); if (!result) throw new Error("unknown exercise ID"); return result; }
  case "get_visual_contract": return api.getVisualContract();
  case "create_flashcard_draft": return api.createDraft({ source_exercise_id: input(args.exercise_id, "exercise_id"), ...(args.teaching_copy_json && { teaching_copy_json: args.teaching_copy_json }) });
  case "build_visual_brief": locked(args.style_profile, STYLE, "style_profile"); locked(args.character_id, CHARACTER, "character_id"); locked(args.cheek_accent, CHEEK, "cheek_accent"); return api.createVisualBrief(input(args.card_id, "card_id"), { exercise_id: input(args.exercise_id, "exercise_id"), style_profile: STYLE, character_id: CHARACTER, pose_json: args.pose ?? {}, palette_json: { cheekAccent: CHEEK }, must_not_show_json: ["arrows", "text", "logos", "watermark", "extra people"] });
  case "generate_flashcard_image": return api.createJob(input(args.card_id, "card_id"), { kind: "generate", brief_id: input(args.brief_id, "brief_id") });
  case "review_flashcard_image": return api.createJob(input(args.card_id, "card_id"), { kind: "review", revision_notes: `asset_id:${input(args.asset_id, "asset_id")}` });
  case "save_flashcard_draft": if (!args.patch || typeof args.patch !== "object" || (!args.patch.category && !args.patch.teaching_copy_json)) throw new Error("patch requires category or teaching_copy_json"); return api.updateDraft(input(args.card_id, "card_id"), Object.fromEntries(Object.entries(args.patch).filter(([key]) => key === "category" || key === "teaching_copy_json")));
  case "submit_for_review": return api.submitForReview(input(args.card_id, "card_id"));
} }

export function createHttpServer({ env = process.env, fetchImpl = fetch } = {}) {
  const verifier = createOAuthVerifier({ env, fetchImpl }); const handler = createMcpHandler({ verifyBearerToken: verifier, clientFactory: (identity) => createMpsClient({ baseUrl: env.MPS_API_BASE_URL, serviceToken: env.MPS_API_SERVICE_TOKEN, identity, fetchImpl }) });
  return http.createServer(async (req, res) => {
    if (req.method === "GET" && req.url === "/.well-known/oauth-protected-resource") return respond(res, 200, { resource: env.MPS_RESOURCE_URL, authorization_servers: [env.MPS_OAUTH_ISSUER] });
    if (req.method !== "POST" || req.url !== "/mcp") return respond(res, 404, { error: "not found" });
    const chunks = []; for await (const chunk of req) chunks.push(chunk); let rpc; try { rpc = JSON.parse(Buffer.concat(chunks).toString("utf8")); } catch { return respond(res, 400, { jsonrpc: "2.0", id: null, error: { code: JSON_RPC.INVALID_REQUEST, message: "invalid JSON" } }); }
    const response = await handler(rpc, req.headers); const unauthorized = response.error?.code === JSON_RPC.UNAUTHORIZED;
    return respond(res, unauthorized ? 401 : 200, { jsonrpc: "2.0", id: rpc.id ?? null, ...response }, unauthorized ? { "www-authenticate": `Bearer resource="${env.MPS_RESOURCE_URL}", resource_metadata="${new URL("/.well-known/oauth-protected-resource", env.MPS_RESOURCE_URL).toString()}"` } : {});
  });
}
function respond(res, status, body, headers = {}) { res.writeHead(status, { "content-type": "application/json", ...headers }); res.end(JSON.stringify(body)); }
if (import.meta.url === `file://${process.argv[1]}`) createHttpServer().listen(Number(process.env.PORT ?? 8787));
