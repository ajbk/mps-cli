const REQUIRED_ENV = ["MPS_OAUTH_INTROSPECTION_URL", "MPS_OAUTH_CLIENT_ID", "MPS_OAUTH_CLIENT_SECRET"];
export const SCOPES = new Set(["read_catalog", "write_draft", "generate_asset", "submit_review"]);

export function requiredScope(context, scope) {
  if (!context?.scopes?.includes(scope)) throw rpcError("FORBIDDEN", `missing required scope: ${scope}`);
}

export function rpcError(code, message) {
  return Object.assign(new Error(message), { code });
}

export function bearerFromHeaders(headers = {}) {
  const value = headers.authorization ?? headers.Authorization;
  if (!value?.startsWith("Bearer ")) throw rpcError("UNAUTHORIZED", "bearer token is required");
  return value.slice("Bearer ".length);
}

export function createOAuthVerifier({ fetchImpl = fetch, env = process.env } = {}) {
  for (const name of REQUIRED_ENV) if (!env[name]) throw new Error(`${name} must be configured`);
  return async (token) => {
    const response = await fetchImpl(env.MPS_OAUTH_INTROSPECTION_URL, {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({ token, client_id: env.MPS_OAUTH_CLIENT_ID, client_secret: env.MPS_OAUTH_CLIENT_SECRET })
    });
    if (!response.ok) return null;
    const claims = await response.json();
    const scopes = String(claims.scope ?? "").split(/\s+/).filter((scope) => SCOPES.has(scope));
    if (!claims.active || !claims.studio_id || !claims.teacher_id || scopes.length === 0) return null;
    return { studioId: claims.studio_id, teacherId: claims.teacher_id, scopes };
  };
}

export async function exchangeAuthorizationCode({ code, codeVerifier, redirectUri, fetchImpl = fetch, env = process.env }) {
  if (!code || !codeVerifier || !redirectUri) throw rpcError("INVALID_INPUT", "code, code_verifier, and redirect_uri are required");
  if (!env.MPS_OAUTH_TOKEN_URL || !env.MPS_OAUTH_CLIENT_ID || !env.MPS_OAUTH_CLIENT_SECRET) {
    throw rpcError("OAUTH_NOT_CONFIGURED", "OAuth authorization-server exchange is not configured");
  }
  const response = await fetchImpl(env.MPS_OAUTH_TOKEN_URL, {
    method: "POST",
    headers: { "content-type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({ grant_type: "authorization_code", code, code_verifier: codeVerifier, redirect_uri: redirectUri, client_id: env.MPS_OAUTH_CLIENT_ID, client_secret: env.MPS_OAUTH_CLIENT_SECRET })
  });
  if (!response.ok) throw rpcError("UNAUTHORIZED", "authorization code exchange failed");
  return response.json();
}
