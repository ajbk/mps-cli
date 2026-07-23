const PRIVATE_KEY = /(?:private|source.*(?:photo|image|path)|(?:file|repo)_path)/i;
const ABSOLUTE_PATH = /^(?:[A-Za-z]:[\\/]|\/|~\/)/;

export function assertSafeIdentifier(value, label) { if (typeof value !== "string" || !value.trim() || ABSOLUTE_PATH.test(value) || value.includes("private://")) throw new Error(`${label} must be a non-private identifier`); return value; }
export function sanitize(value, key = "") { if (PRIVATE_KEY.test(key)) return undefined; if (typeof value === "string") return value.startsWith("private://") || ABSOLUTE_PATH.test(value) ? undefined : value; if (Array.isArray(value)) return value.map((item) => sanitize(item)).filter((item) => item !== undefined); if (value && typeof value === "object") return Object.fromEntries(Object.entries(value).flatMap(([childKey, child]) => { const clean = sanitize(child, childKey); return clean === undefined ? [] : [[childKey, clean]]; })); return value; }

export function createMpsClient({ baseUrl, serviceToken, identity, fetchImpl = fetch }) {
  if (!baseUrl || !serviceToken || !identity?.studioId || !identity?.teacherId) throw new Error("MPS API service configuration is incomplete");
  async function request(path, options = {}) {
    const response = await fetchImpl(new URL(path, baseUrl), { ...options, headers: { authorization: `Bearer ${serviceToken}`, "content-type": "application/json", "x-mps-studio-id": identity.studioId, "x-mps-teacher-id": identity.teacherId, "x-mps-actor-kind": "chatgpt", ...options.headers } });
    if (response.status === 404) return null;
    const body = await response.json();
    if (!response.ok) throw new Error(body?.error?.message ?? `MPS API request failed (${response.status})`);
    return sanitize(body);
  }
  return {
    searchExercises: (query, apparatus, level, bodyRegion) => request(`/api/catalog/exercises?${new URLSearchParams({ query, ...(apparatus && { apparatus }), ...(level && { level }), ...(bodyRegion && { body_region: bodyRegion }) })}`),
    getExerciseContext: (exerciseId) => request(`/api/catalog/exercises/${encodeURIComponent(assertSafeIdentifier(exerciseId, "exercise_id"))}/context`), getVisualContract: () => request("/api/visual-contract"),
    createDraft: (body) => request("/api/flashcards", { method: "POST", body: JSON.stringify(body) }), createVisualBrief: (cardId, body) => request(`/api/flashcards/${encodeURIComponent(assertSafeIdentifier(cardId, "card_id"))}/visual-briefs`, { method: "POST", body: JSON.stringify(body) }), createJob: (cardId, body) => request(`/api/flashcards/${encodeURIComponent(assertSafeIdentifier(cardId, "card_id"))}/jobs`, { method: "POST", body: JSON.stringify(body) }), updateDraft: (cardId, body) => request(`/api/flashcards/${encodeURIComponent(assertSafeIdentifier(cardId, "card_id"))}`, { method: "PATCH", body: JSON.stringify(body) }), submitForReview: (cardId) => request(`/api/flashcards/${encodeURIComponent(assertSafeIdentifier(cardId, "card_id"))}/submit-review`, { method: "POST" })
  };
}
