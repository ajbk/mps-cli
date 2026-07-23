# MPS ChatGPT App / MCP setup

`services/mps-mcp` is a small MCP JSON-RPC HTTP resource server. It exposes a
draft-only MPS authoring surface; the MPS API and teacher workflow remain the
authority for review, approval, and publication.

## OAuth boundary

Production uses OAuth 2.1. ChatGPT is the OAuth client and receives an access
token from the studio authorization server using authorization-code flow with
PKCE. ChatGPT owns the authorization redirect and code exchange. The MCP
service is a resource server: it publishes protected-resource metadata at
`/.well-known/oauth-protected-resource`, returns an HTTP 401 Bearer challenge
with resource metadata for absent or invalid tokens, and validates the bearer
token on every `/mcp` request through the configured authorization-server
introspection endpoint.

Configure these deployment environment variables (see
`services/mps-mcp/.env.example`):

- `MPS_API_BASE_URL`: the MPS HTTP API origin.
- `MPS_RESOURCE_URL`: public MCP URL ending in `/mcp`.
- `MPS_OAUTH_ISSUER` and `MPS_OAUTH_INTROSPECTION_URL`:
  the studio authorization server endpoints.
- `MPS_OAUTH_CLIENT_ID` and `MPS_OAUTH_CLIENT_SECRET`: deployment secrets used
  only for token introspection.
- `MPS_API_SERVICE_TOKEN`: server-only credential for the MPS API.
- `MPS_MCP_MAX_BODY_BYTES`: maximum MCP POST body size (defaults to 65,536).

The adapter does not implement an OAuth callback, code exchange, or token
storage. Local development has no OAuth fallback or embedded test identity:
configure a real authorization server, or inject a verifier only in automated
tests.

In the ChatGPT app connection, use the public MCP endpoint
`https://<MCP-host>/mcp`, grant only the requested scopes, and register the
authorization server with the MCP resource URL as its audience/resource.
The app should discover the resource metadata from the MCP host and initiate
authorization with PKCE.

## Scope map

| Scope | MCP tools |
| --- | --- |
| `read_catalog` | `search_exercises`, `get_exercise_context`, `get_visual_contract` |
| `write_draft` | `create_flashcard_draft`, `build_visual_brief`, `save_flashcard_draft` |
| `generate_asset` | `generate_flashcard_image` |
| `submit_review` | `review_flashcard_image`, `submit_for_review` |

Token claims must contain active status, `studio_id`, `teacher_id`, and at least
one supported scope, exact issuer, matching resource audience, and an unexpired
`exp`. The adapter does not accept studio or teacher IDs from a tool call.
An insufficient scope receives HTTP 401 with `error="insufficient_scope"`, the
required `scope`, and an MCP reauthorization challenge in response metadata so
ChatGPT can request the added scope.

MCP notifications, including `notifications/initialized`, return HTTP 202 with
no body. JSON-RPC batches are supported and return only responses for entries
that include an ID; an empty batch is invalid.

## Safety contract

The adapter rejects unknown exercise IDs, absolute paths, private identifiers,
and any style profile other than `mono-gesture-ink-pilates-v1`. Visual briefs
are locked to `teacher-01`, the locked outfit, and subtle dusty-rose cheek
accent `#D98F9A`. API results are recursively stripped of source-photo,
database-path, and private-reference fields before being returned to ChatGPT.
Runtime validation matches each published tool schema and rejects unknown,
identity, publication-status, and unlocked-outfit fields instead of dropping
them silently.

There is intentionally no publish or approval tool. ChatGPT cannot approve or
publish a flashcard. Teacher review and the server/PWA publication workflow are
the only authority for that transition.

## Operational limitations

This repository provides the MCP adapter boundary, not a hosted authorization
server or an image provider. Deploy an OAuth 2.1 authorization server and MPS
HTTP API before connecting it to ChatGPT. The API token validation policy must
be configured to return the authenticated studio, teacher, and allowed scopes;
the adapter does not manufacture or persist them.

The adapter never forwards the end-user OAuth bearer to MPS. Instead it calls
MPS with `MPS_API_SERVICE_TOKEN` and verified studio/teacher identity headers.
MPS accepts those headers only when its `MPS_MCP_SERVICE_TOKEN` matches that
service token and requires actor kind `chatgpt`; human static bearer auth is
unchanged. These adapter-originated mutations are audited as `chatgpt` with the
verified teacher identity. Keep both service-token variables server-side and
identical; never expose either to ChatGPT or a browser.
