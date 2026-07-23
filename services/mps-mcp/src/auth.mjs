const REQUIRED_ENV = ["MPS_OAUTH_INTROSPECTION_URL", "MPS_OAUTH_CLIENT_ID", "MPS_OAUTH_CLIENT_SECRET", "MPS_OAUTH_ISSUER", "MPS_RESOURCE_URL"];
export const SCOPES = new Set(["read_catalog", "write_draft", "generate_asset", "submit_review"]);

export const JSON_RPC = { INVALID_REQUEST: -32600, METHOD_NOT_FOUND: -32601, INVALID_PARAMS: -32602, INTERNAL_ERROR: -32603, UNAUTHORIZED: -32001, FORBIDDEN: -32003 };

export function rpcError(code, message) { return Object.assign(new Error(message), { code }); }
export function requiredScope(context, scope) { if (!context?.scopes?.includes(scope)) throw rpcError(JSON_RPC.FORBIDDEN, `missing required scope: ${scope}`); }
export function bearerFromHeaders(headers = {}) { const value = headers.authorization ?? headers.Authorization; if (!value?.startsWith("Bearer ")) throw rpcError(JSON_RPC.UNAUTHORIZED, "bearer token is required"); return value.slice(7); }

function audienceContains(audience, resource) { return (Array.isArray(audience) ? audience : [audience]).includes(resource); }

export function createOAuthVerifier({ fetchImpl = fetch, env = process.env, now = () => Date.now() } = {}) {
  for (const name of REQUIRED_ENV) if (!env[name]) throw new Error(`${name} must be configured`);
  return async (token) => {
    const response = await fetchImpl(env.MPS_OAUTH_INTROSPECTION_URL, { method: "POST", headers: { "content-type": "application/x-www-form-urlencoded" }, body: new URLSearchParams({ token, client_id: env.MPS_OAUTH_CLIENT_ID, client_secret: env.MPS_OAUTH_CLIENT_SECRET }) });
    if (!response.ok) return null;
    const claims = await response.json();
    const scopes = String(claims.scope ?? "").split(/\s+/).filter((scope) => SCOPES.has(scope));
    if (!claims.active || claims.iss !== env.MPS_OAUTH_ISSUER || !audienceContains(claims.aud, env.MPS_RESOURCE_URL) || !Number.isFinite(claims.exp) || claims.exp <= now() / 1000 || !claims.studio_id || !claims.teacher_id || scopes.length === 0) return null;
    return { studioId: claims.studio_id, teacherId: claims.teacher_id, scopes };
  };
}
