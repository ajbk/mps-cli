import http from "node:http";
import { bearerFromHeaders, createOAuthVerifier, JSON_RPC, requiredScope, rpcError } from "./auth.mjs";
import { assertSafeIdentifier, createMpsClient, sanitize } from "./mps-client.mjs";

const STYLE = "mono-gesture-ink-pilates-v1"; const CHARACTER = "teacher-01"; const CHEEK = "#D98F9A";
const object = (properties, required = []) => ({ type: "object", properties, required, additionalProperties: false });
const str = { type: "string", minLength: 1 };
export const tools = [
  ["search_exercises", "read_catalog", object({ query: str, apparatus: str, level: str }, ["query"])], ["get_exercise_context", "read_catalog", object({ exercise_id: str }, ["exercise_id"])], ["get_visual_contract", "read_catalog", object({})], ["create_flashcard_draft", "write_draft", object({ exercise_id: str, teaching_copy_json: { type: "object", additionalProperties: true } }, ["exercise_id"])], ["build_visual_brief", "write_draft", object({ card_id: str, exercise_id: str, pose: { type: "object", additionalProperties: true }, style_profile: str, character_id: str, cheek_accent: str }, ["card_id", "exercise_id", "pose"])], ["generate_flashcard_image", "generate_asset", object({ card_id: str, brief_id: str }, ["card_id", "brief_id"])], ["review_flashcard_image", "submit_review", object({ card_id: str, asset_id: str }, ["card_id", "asset_id"])], ["save_flashcard_draft", "write_draft", object({ card_id: str, patch: object({ category: str, teaching_copy_json: { type: "object", additionalProperties: true } }) }, ["card_id", "patch"])], ["submit_for_review", "submit_review", object({ card_id: str }, ["card_id"])]
].map(([name, scope, inputSchema]) => ({ name, description: `MPS flashcard ${name.replaceAll("_", " ")}.`, inputSchema, scope }));
const scopeFor = (name) => tools.find((tool) => tool.name === name)?.scope;
const toolFor = (name) => tools.find((tool) => tool.name === name);
const toolResult = (value, isError = false) => ({ content: [{ type: "text", text: JSON.stringify(value) }], isError, ...(isError ? {} : { structuredContent: value }) });
function input(value, field) { return assertSafeIdentifier(value, field); } function locked(value, expected, field) { if (value !== undefined && value !== expected) throw new Error(`${field} is locked to ${expected}`); }
function validate(schema, value, path = "arguments") {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`${path} must be an object`);
  if (schema.additionalProperties === true) return;
  for (const key of schema.required ?? []) if (!(key in value)) throw new Error(`${path}.${key} is required`);
  for (const [key, entry] of Object.entries(value)) {
    const property = schema.properties?.[key];
    if (!property) throw new Error(`${path}.${key} is not allowed`);
    if (property.type === "string" && (typeof entry !== "string" || (property.minLength && entry.length < property.minLength))) throw new Error(`${path}.${key} must be a non-empty string`);
    if (property.type === "object") validate(property, entry, `${path}.${key}`);
  }
}

export function createMcpHandler({ verifyBearerToken, client, clientFactory } = {}) {
  if (!verifyBearerToken) throw new Error("verifyBearerToken is required");
  return async (request, headers = {}) => {
    try {
      if (request?.method === "notifications/initialized") return { result: null };
      if (request?.method === "initialize") return { result: { protocolVersion: "2025-03-26", capabilities: { tools: {} }, serverInfo: { name: "mps-flashcard-mcp", version: "0.1.2" } } };
      if (request?.method === "tools/list") return { result: { tools } };
      if (request?.method !== "tools/call") throw rpcError(JSON_RPC.METHOD_NOT_FOUND, "method not found");
      const name = request.params?.name; const tool = toolFor(name); const scope = scopeFor(name); if (!scope) throw rpcError(JSON_RPC.METHOD_NOT_FOUND, "tool not found");
      const token = bearerFromHeaders(headers); const identity = await verifyBearerToken(token); if (!identity) throw rpcError(JSON_RPC.UNAUTHORIZED, "invalid bearer token"); requiredScope(identity, scope);
      const api = client ?? clientFactory?.(identity); if (!api) throw new Error("MPS API client is required");
      try { validate(tool.inputSchema, request.params?.arguments ?? {}); return { result: toolResult(sanitize(await execute(name, request.params.arguments, api))) }; } catch (error) { return { result: toolResult({ error: error.message }, true) }; }
    } catch (error) { return { error: { code: Number.isInteger(error.code) ? error.code : JSON_RPC.INTERNAL_ERROR, message: error.message, ...(error.requiredScope && { data: { requiredScope: error.requiredScope } }) } }; }
  };
}

async function execute(name, args, api) { switch (name) {
  case "search_exercises": return api.searchExercises(args.query, args.apparatus, args.level);
  case "get_exercise_context": { const result = await api.getExerciseContext(input(args.exercise_id, "exercise_id")); if (!result) throw new Error("unknown exercise ID"); return result; }
  case "get_visual_contract": return api.getVisualContract();
  case "create_flashcard_draft": return api.createDraft({ source_exercise_id: input(args.exercise_id, "exercise_id"), ...(args.teaching_copy_json && { teaching_copy_json: args.teaching_copy_json }) });
  case "build_visual_brief": locked(args.style_profile, STYLE, "style_profile"); locked(args.character_id, CHARACTER, "character_id"); locked(args.cheek_accent, CHEEK, "cheek_accent"); return api.createVisualBrief(input(args.card_id, "card_id"), { exercise_id: input(args.exercise_id, "exercise_id"), style_profile: STYLE, character_id: CHARACTER, pose_json: args.pose, palette_json: { cheekAccent: CHEEK }, must_not_show_json: ["arrows", "text", "logos", "watermark", "extra people"] });
  case "generate_flashcard_image": return api.createJob(input(args.card_id, "card_id"), { kind: "generate", brief_id: input(args.brief_id, "brief_id") });
  case "review_flashcard_image": return api.createJob(input(args.card_id, "card_id"), { kind: "review", revision_notes: `asset_id:${input(args.asset_id, "asset_id")}` });
  case "save_flashcard_draft": return api.updateDraft(input(args.card_id, "card_id"), args.patch);
  case "submit_for_review": return api.submitForReview(input(args.card_id, "card_id"));
} }

function isNotification(request) { return !Object.prototype.hasOwnProperty.call(request, "id"); }
function rpcResponse(request, response) { return { jsonrpc: "2.0", id: request.id ?? null, ...response }; }
function maxBodyBytes(env) { const value = Number(env.MPS_MCP_MAX_BODY_BYTES ?? 65_536); return Number.isSafeInteger(value) && value > 0 ? value : 65_536; }
async function readBody(request, maxBytes) { if (Number(request.headers["content-length"] ?? 0) > maxBytes) return null; const chunks = []; let size = 0; for await (const chunk of request) { size += chunk.length; if (size > maxBytes) return null; chunks.push(chunk); } return Buffer.concat(chunks); }
function challenge(env, error, requiredScope) { const parts = [`Bearer resource="${env.MPS_RESOURCE_URL}"`, `resource_metadata="${new URL("/.well-known/oauth-protected-resource", env.MPS_RESOURCE_URL).toString()}"`]; if (error) parts.push(`error="${error}"`); if (requiredScope) parts.push(`scope="${requiredScope}"`); return parts.join(", "); }

export function createHttpServer({ env = process.env, fetchImpl = fetch } = {}) {
  const verifier = createOAuthVerifier({ env, fetchImpl }); const handler = createMcpHandler({ verifyBearerToken: verifier, clientFactory: (identity) => createMpsClient({ baseUrl: env.MPS_API_BASE_URL, serviceToken: env.MPS_API_SERVICE_TOKEN, identity, fetchImpl }) });
  return http.createServer(async (req, res) => {
    if (req.method === "GET" && req.url === "/.well-known/oauth-protected-resource") return respond(res, 200, { resource: env.MPS_RESOURCE_URL, authorization_servers: [env.MPS_OAUTH_ISSUER] });
    if (req.method !== "POST" || req.url !== "/mcp") return respond(res, 404, { error: "not found" });
    const body = await readBody(req, maxBodyBytes(env)); if (!body) return respond(res, 413, { error: "request body too large" });
    let rpc; try { rpc = JSON.parse(body.toString("utf8")); } catch { return respond(res, 400, { jsonrpc: "2.0", id: null, error: { code: JSON_RPC.INVALID_REQUEST, message: "invalid JSON" } }); }
    if (Array.isArray(rpc) && rpc.length === 0) return respond(res, 400, { jsonrpc: "2.0", id: null, error: { code: JSON_RPC.INVALID_REQUEST, message: "empty batch" } });
    const requests = Array.isArray(rpc) ? rpc : [rpc]; const responses = [];
    for (const request of requests) { const response = await handler(request, req.headers); if (!isNotification(request)) responses.push(rpcResponse(request, response)); }
    if (responses.length === 0) return res.writeHead(202).end();
    const insufficient = responses.find((response) => response.error?.code === JSON_RPC.FORBIDDEN && response.error.data?.requiredScope); const unauthorized = responses.find((response) => response.error?.code === JSON_RPC.UNAUTHORIZED);
    const authError = insufficient ?? unauthorized; const requiredScope = insufficient?.error.data.requiredScope; const header = authError && challenge(env, insufficient ? "insufficient_scope" : "invalid_token", requiredScope);
    if (authError) for (const response of responses) if (response.error === authError.error) response.error.data = { ...response.error.data, _meta: { "mcp/www_authenticate": [header] } };
    return respond(res, authError ? 401 : 200, Array.isArray(rpc) ? responses : responses[0], header ? { "www-authenticate": header } : {});
  });
}
function respond(res, status, body, headers = {}) { res.writeHead(status, { "content-type": "application/json", ...headers }); res.end(JSON.stringify(body)); }
if (import.meta.url === `file://${process.argv[1]}`) createHttpServer().listen(Number(process.env.PORT ?? 8787));
