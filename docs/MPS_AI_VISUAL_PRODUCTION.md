# MPS AI Visual Production

This guide is for teachers who create Pilates flashcards with the MPS PWA and
the MPS Custom App in ChatGPT.

## What owns each decision

- `MPS_Database_v1_Core.xlsx` and its derived catalog own exercise identity,
  apparatus, level, and workbook taxonomy.
- `mono-gesture-ink-pilates-v1` owns the drawing language.
- The approved/reviewed `teacher-01` character contract owns the hair, glasses,
  body proportions, outfit, and subtle dusty-rose cheek accent.
- ChatGPT may read the catalog, prepare a visual brief, request generation, and
  request review work through MCP.
- The server owns card status, job state, review state, approval guards, and
  publication. The teacher is the person who approves and publishes.

The browser never calls an image provider and never contains an image-provider
key. In API mode, the browser sends normal MPS API requests and treats the API
response as authoritative. Without `MPS_API_BASE`, the PWA stays usable as a
static demo with local drafts; that mode cannot publish a real card.

Set `window.MPS_API_BASE` before the application scripts load (the default in
`web/index.html` is an empty string). A same-origin deployment can leave it
empty only when the API is mounted at the PWA origin; a separate API origin
must be written as its HTTPS base URL and configured for browser credentials.

## Connect the Custom App in ChatGPT

1. Deploy the MPS MCP service and MPS API with the server-side environment
   variables in `services/mps-mcp/.env.example`.
2. In ChatGPT, create or open a custom app/connector and enter the public MCP
   endpoint ending in `/mcp`.
3. Configure the OAuth 2.1 authorization server for the MCP resource URL. The
   ChatGPT client performs the authorization-code + PKCE flow; the MCP service
   publishes protected-resource metadata and does not receive a browser-side
   client secret.
4. Grant only the scopes needed for the current task. Drafting normally needs
   `read_catalog` and `write_draft`; generation needs `generate_asset`; review
   submission needs `submit_review`.

For the current ChatGPT Apps SDK authentication model, see the official
[Apps SDK authentication guide](https://developers.openai.com/apps-sdk/build/auth)
and [Apps SDK quickstart](https://developers.openai.com/apps-sdk/quickstart/).

## Teacher workflow

### 1. Select a workbook exercise

Open **Flashcards**, search by the canonical exercise name or ID, and choose
**Create card**. In API mode this creates the draft through `POST
/api/flashcards`; in static mode it creates a local demo draft.

Do not replace the source exercise with a similar-looking name. The source ID
is the identity used by the visual brief, worker, review, and publication
guards.

### 2. Edit and save teaching copy

Edit only the teacher-facing question, cue, regression, and progression. Select
**Save teacher changes**. The API stores these fields as teaching-copy JSON;
locked workbook and visual fields are not accepted from the browser.

### 3. Ask ChatGPT for the visual brief

Select **Ask ChatGPT** in the card editor. Connect the MPS Custom App if it is
not already connected, then ask ChatGPT to:

> Prepare a visual brief for this MPS flashcard using the canonical exercise
> context. Use `mono-gesture-ink-pilates-v1`, `teacher-01`, the locked outfit,
> the real teacher proportions, and the subtle dusty-rose cheek accent. Do not
> add arrows, labels, logos, watermarks, extra people, or style-reference
> identity. Save the brief and return its brief ID.

Paste the returned brief ID into **Visual brief ID from ChatGPT**. The browser
stores only that non-secret ID locally. It does not create an unreviewed image
brief or send provider credentials.

### 4. Generate and poll the job

Select **Generate**. The PWA calls the MPS job endpoint with the saved brief ID,
then polls the job endpoint until the server reports `succeeded` or `failed`.
Generation happens in the server-side visual worker. A failed job is not an
approval failure; read the server error, correct the brief or ask ChatGPT for a
revision, and run a new generation job.

### 5. Inspect the visual and findings

Check the pose and contact points first, then check:

- the teacher remains recognizable: shoulder-length layered bob and round dark
  glasses;
- the proportions look like the real teacher, without slimming or lengthening;
- the outfit is the off-white thin-strap cropped Pilates camisole and dark
  charcoal high-waisted mid-thigh biker shorts;
- the mono gesture-ink linework remains spare, organic, and mostly monochrome;
- the cheek tint is subtle dusty rose, not heavy makeup or a full-color render;
- the apparatus is readable and the body is anatomically plausible;
- there are no arrows, instructional labels, UI chrome, logos, or watermarks.

Use **Show findings** when the API or completed job returns validation findings.
Findings are displayed as text; they are never interpreted as HTML.

### 6. Request a revision

In static demo mode, **Request revision** records the local state transition.
The current MPS API does not expose a dedicated browser revision route. In API
mode, ask ChatGPT/MCP to revise or regenerate the brief, then refresh the card
and use the returned brief ID for a new generation. The PWA does not fake a
revision status by editing local browser state around the server.

### 7. Submit review, approve, and publish

Select **Submit review** after the generated job is complete. The API records
the `needs-review` transition. The server checks the latest automated review,
current asset, locked visual contract, and canonical source before accepting
**Approve**.

**Publish** stays disabled until the API reports `approved`. The server performs
the final checks again when the teacher publishes, so a stale browser tab cannot
publish an outdated asset. ChatGPT/MCP has no approve or publish tool.

## Troubleshooting

| Symptom | What to do |
| --- | --- |
| The library is empty in API mode | Confirm the API session is authenticated and that the teacher has catalog permission. |
| Generate is disabled | Paste the visual brief ID returned by ChatGPT, then refresh the card if the API has not returned the brief yet. |
| The job remains running | Leave the card open until polling ends, then refresh. Do not start a second job for the same version. |
| Findings reject the image | Ask ChatGPT to revise the brief against the exact failed checks; do not alter the locked style or character fields. |
| Publish is disabled | Confirm the API status is `approved` and that the latest asset has passed automated and teacher review. |
| Static mode cannot publish | This is intentional. Configure `MPS_API_BASE` and use the authenticated MPS API for real publication. |
