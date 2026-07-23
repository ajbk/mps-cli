import http from "node:http";
import { bearerFromHeaders, createOAuthVerifier, exchangeAuthorizationCode, requiredScope, rpcError } from "./auth.mjs";
import { assertSafeIdentifier, createMpsClient, sanitize } from "./mps-client.mjs";

const STYLE = "mono-gesture-ink-pilates-v1";
const CHARACTER = "teacher-01";
const CHEEK = "#D98F9A";
const TOOL_SCOPES = { search_exercises: "read_catalog", get_exercise_context: "read_catalog", get_visual_contract: "read_catalog", create_flashcard_draft: "write_draft", build_visual_brief: "write_draft", generate_flashcard_image: "generate_asset", review_flashcard_image: "submit_review", save_flashcard_draft: "write_draft", submit_for_review: "submit_review" };

export const tools = Object.keys(TOOL_SCOPES).map((name) => ({ name, scope: TOOL_SCOPES[name] }));

function input(value, field) { return assertSafeIdentifier(value, field); }
function locked(value, expected, field) { if (value !== undefined && value !== expected) throw rpcError("INVALID_INPUT", `${field} is locked to ${expected}`); }
function bad(error) { return { error: { code: error.code ?? "INVALID_INPUT", message: error.message } }; }

export function createMcpHandler({ verifyBearerToken, client, clientFactory } = {}) {
  if (!verifyBearerToken) throw new Error("verifyBearerToken is required");
  return async function handle(request, headers = {}) {
    try {
      if (request?.method === "initialize") return { result: { protocolVersion: "2025-03-26", capabilities: { tools: {} }, serverInfo: { name: "mps-flashcard-mcp", version: "0.1.0" } } };
      if (request?.method === "tools/list") return { result: tools };
      if (request?.method !== "tools/call") throw rpcError("METHOD_NOT_FOUND", "method not found");
      const name = request.params?.name;
      if (!TOOL_SCOPES[name]) throw rpcError("METHOD_NOT_FOUND", "tool not found");
      const token = bearerFromHeaders(headers);
      const context = await verifyBearerToken(token);
      if (!context) throw rpcError("UNAUTHORIZED", "invalid bearer token");
      requiredScope(context, TOOL_SCOPES[name]);
      const api = client ?? clientFactory?.(token, context);
      if (!api) throw new Error("MPS API client is required");
      const result = await execute(name, request.params?.arguments ?? {}, api);
      return { result: sanitize(result) };
    } catch (error) { return bad(error); }
  };
}

async function execute(name, args, api) {
  switch (name) {
    case "search_exercises": return api.searchExercises(String(args.query ?? "").trim(), args.apparatus, args.level);
    case "get_exercise_context": { const result = await api.getExerciseContext(input(args.exercise_id, "exercise_id")); if (!result) throw rpcError("INVALID_INPUT", "unknown exercise ID"); return result; }
    case "get_visual_contract": return api.getVisualContract();
    case "create_flashcard_draft": {
      const context = await api.getExerciseContext(input(args.exercise_id, "exercise_id"));
      if (!context) throw rpcError("INVALID_INPUT", "unknown exercise ID");
      return api.createDraft({ source_exercise_id: args.exercise_id, ...(args.teaching_copy_json && { teaching_copy_json: args.teaching_copy_json }) });
    }
    case "build_visual_brief": {
      input(args.card_id, "card_id"); input(args.exercise_id, "exercise_id");
      locked(args.style_profile, STYLE, "style_profile"); locked(args.character_id, CHARACTER, "character_id"); locked(args.cheek_accent, CHEEK, "cheek_accent");
      const context = await api.getExerciseContext(args.exercise_id); if (!context) throw rpcError("INVALID_INPUT", "unknown exercise ID");
      return api.createVisualBrief(args.card_id, { exercise_id: args.exercise_id, style_profile: STYLE, character_id: CHARACTER, pose_json: args.pose ?? {}, apparatus: context.exercise.apparatus, palette_json: { cheekAccent: CHEEK }, must_not_show_json: ["arrows", "text", "logos", "watermark", "extra people"] });
    }
    case "generate_flashcard_image": return api.createJob(input(args.card_id, "card_id"), { kind: "generate", brief_id: input(args.brief_id, "brief_id") });
    case "review_flashcard_image": return api.createJob(input(args.card_id, "card_id"), { kind: "review", revision_notes: `asset_id:${input(args.asset_id, "asset_id")}` });
    case "save_flashcard_draft": {
      input(args.card_id, "card_id");
      if (!args.patch || typeof args.patch !== "object" || (!args.patch.category && !args.patch.teaching_copy_json)) throw rpcError("INVALID_INPUT", "patch requires category or teaching_copy_json");
      return api.updateDraft(args.card_id, Object.fromEntries(Object.entries(args.patch).filter(([key]) => key === "category" || key === "teaching_copy_json")));
    }
    case "submit_for_review": return api.submitForReview(input(args.card_id, "card_id"));
  }
}

export function createHttpServer({ env = process.env, fetchImpl = fetch } = {}) {
  const verifier = createOAuthVerifier({ env, fetchImpl });
  const handler = createMcpHandler({ verifyBearerToken: verifier, clientFactory: (token) => createMpsClient({ baseUrl: env.MPS_API_BASE_URL, bearerToken: token, fetchImpl }) });
  return http.createServer(async (req, res) => {
    if (req.method === "GET" && req.url === "/.well-known/oauth-protected-resource") return respond(res, 200, { resource: env.MPS_RESOURCE_URL, authorization_servers: [env.MPS_OAUTH_ISSUER] });
    if (req.method === "POST" && new URL(req.url, "http://localhost").pathname === "/oauth/callback") {
      try { return respond(res, 200, { token: await exchangeAuthorizationCode({ code: new URL(req.url, "http://localhost").searchParams.get("code"), codeVerifier: req.headers["x-pkce-verifier"], redirectUri: env.MPS_OAUTH_REDIRECT_URI, env, fetchImpl }) }); } catch (error) { return respond(res, 400, bad(error)); }
    }
    if (req.method !== "POST" || req.url !== "/mcp") return respond(res, 404, { error: { code: "NOT_FOUND", message: "not found" } });
    const chunks = []; for await (const chunk of req) chunks.push(chunk);
    const rpc = JSON.parse(Buffer.concat(chunks).toString("utf8"));
    const response = await handler(rpc, req.headers);
    return respond(res, 200, { jsonrpc: "2.0", id: rpc.id ?? null, ...response });
  });
}

function respond(res, status, body) { res.writeHead(status, { "content-type": "application/json" }); res.end(JSON.stringify(body)); }

if (import.meta.url === `file://${process.argv[1]}`) createHttpServer().listen(Number(process.env.PORT ?? 8787));
