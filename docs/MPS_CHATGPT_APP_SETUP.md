# MPS ChatGPT App / MCP setup

`services/mps-mcp` is a small MCP JSON-RPC HTTP resource server. It exposes a
draft-only MPS authoring surface; the MPS API and teacher workflow remain the
authority for review, approval, and publication.

## OAuth boundary

Production uses OAuth 2.1. ChatGPT is the OAuth client and receives an access
token from the studio authorization server using authorization-code flow with
PKCE. The MCP service is a resource server: it publishes protected-resource
metadata at `/.well-known/oauth-protected-resource` and validates the bearer
token on every `/mcp` request through the configured authorization-server
introspection endpoint. It forwards that bearer token only from server to the
MPS HTTP API.

Configure these deployment environment variables (see
`services/mps-mcp/.env.example`):

- `MPS_API_BASE_URL`: the MPS HTTP API origin.
- `MPS_RESOURCE_URL`: public MCP URL ending in `/mcp`.
- `MPS_OAUTH_ISSUER`, `MPS_OAUTH_INTROSPECTION_URL`, and `MPS_OAUTH_TOKEN_URL`:
  the studio authorization server endpoints.
- `MPS_OAUTH_CLIENT_ID`, `MPS_OAUTH_CLIENT_SECRET`, and
  `MPS_TOKEN_ENCRYPTION_KEY`: deployment secrets only.
- `MPS_OAUTH_REDIRECT_URI`: `https://<MCP-host>/oauth/callback`, registered with
  the authorization server.

The callback accepts an authorization code plus its PKCE verifier and exchanges
it at the authorization server. The adapter does not store access or refresh
tokens. Local development has no OAuth fallback or embedded test identity:
configure a real authorization server, or inject a verifier only in automated
tests.

In the ChatGPT app connection, use the public MCP endpoint
`https://<MCP-host>/mcp`, grant only the requested scopes, and register the
same redirect URI with the authorization server. The app should discover the
resource metadata from the MCP host and initiate authorization with PKCE.

## Scope map

| Scope | MCP tools |
| --- | --- |
| `read_catalog` | `search_exercises`, `get_exercise_context`, `get_visual_contract` |
| `write_draft` | `create_flashcard_draft`, `build_visual_brief`, `save_flashcard_draft` |
| `generate_asset` | `generate_flashcard_image` |
| `submit_review` | `review_flashcard_image`, `submit_for_review` |

Token claims must contain active status, `studio_id`, `teacher_id`, and at least
one supported scope. The adapter does not accept studio or teacher IDs from a
tool call; the authenticated MPS API context supplies those values to its
existing audited draft and review routes.

## Safety contract

The adapter rejects unknown exercise IDs, absolute paths, private identifiers,
and any style profile other than `mono-gesture-ink-pilates-v1`. Visual briefs
are locked to `teacher-01`, the locked outfit, and subtle dusty-rose cheek
accent `#D98F9A`. API results are recursively stripped of source-photo,
database-path, and private-reference fields before being returned to ChatGPT.

There is intentionally no publish or approval tool. ChatGPT cannot approve or
publish a flashcard. Teacher review and the server/PWA publication workflow are
the only authority for that transition.

## Operational limitations

This repository provides the MCP adapter boundary, not a hosted authorization
server or an image provider. Deploy an OAuth 2.1 authorization server and MPS
HTTP API before connecting it to ChatGPT. The API token validation policy must
be configured to return the authenticated studio, teacher, and allowed scopes;
the adapter does not manufacture or persist them.

Task 6 currently records those authenticated draft/review mutations as a
`teacher` actor in the Rust repository. Changing the persisted audit actor to
`chatgpt` requires an MPS API/database change and is deliberately outside this
adapter-only task; the adapter does not spoof that value in tool input or
headers.
