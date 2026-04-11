# Research: Teacher Assignment Capture

## Decision 1: Photo Storage Strategy

**Decision**: Use Cloudflare R2 for photo storage, accessed via the API server on Fly.io.

**Rationale**: 
- Turso (libsql/SQLite) is not suitable for binary blob storage — row size limits and query performance degrade with large payloads.
- Cloudflare R2 is already in the ecosystem (web shell deployed via Cloudflare Workers) and has an S3-compatible API.
- R2 has no egress fees, which is ideal for photo retrieval.
- Photos are stored as objects keyed by `{user_id}/{lesson_id}/{photo_id}.jpg`. The DB stores only the object key, not the binary data.

**Alternatives considered**:
- *Base64 in DB*: Simple but hits Turso row size limits (~1MB), inflates payload by 33%, and degrades query performance on large datasets. Rejected.
- *Local device storage only*: No cross-device access, no backup. Rejected.
- *Direct shell-to-R2 upload via presigned URL*: More complex (requires presigned URL generation endpoint), but better for very large files. Deferred — not needed for compressed lesson photos.

## Decision 2: Photo Upload Flow Through Crux Architecture

**Decision**: Photo upload is handled by the shell directly, outside the Crux effect system. Core manages photo metadata only.

**Rationale**:
- Crux's HTTP capability is JSON-only (`body_json()`). There is no multipart/binary support.
- The current request body limit is 1MB, insufficient for raw photos.
- The shell (iOS: native camera/photo picker, web: file input) captures the photo, compresses it, and uploads via a dedicated multipart API endpoint (`POST /api/lessons/{id}/photos`).
- After upload, the shell dispatches an event to core to refresh lesson data, which includes photo metadata (URLs/keys) from the normal JSON API.
- Core never handles binary photo data — it only sees photo metadata in the lesson model.

**Alternatives considered**:
- *Base64 in JSON via Crux HTTP*: Would require increasing body limit, encoding/decoding overhead, and makes core handle binary concerns. Rejected.
- *Custom Crux capability for file uploads*: Over-engineered for this feature. Rejected.

## Decision 3: Photo Compression

**Decision**: Shell compresses photos to max 2048px longest edge, JPEG quality 80%, before upload.

**Rationale**:
- Lesson photos are reference material (handwritten notes, sheet music annotations), not high-art photography. 2048px is more than sufficient for readability.
- At JPEG 80%, a 2048px photo is typically 200-500KB — well within reasonable upload limits.
- Compression happens on-device before upload, reducing bandwidth and storage costs.

**Alternatives considered**:
- *Server-side compression*: Wastes bandwidth uploading full-resolution images. Rejected.
- *No compression*: 10MB+ photos from modern phone cameras would be wasteful. Rejected.

## Decision 4: Lesson Navigation Placement

**Decision**: "Log Lesson" accessible via a prominent action button on the Library screen (both platforms), not a new tab.

**Rationale**:
- Adding a new tab to the bottom navigation changes the app's information architecture significantly — too much for an M-sized feature.
- Lessons are closely related to the Library (they produce items that live there eventually). A button on the Library screen keeps them discoverable without restructuring navigation.
- The lessons list is accessible from the same entry point (tap "Log Lesson" or "View Lessons" from Library).
- This can be promoted to a tab or dedicated section later if lessons prove to be heavily used.

**Alternatives considered**:
- *New bottom tab*: Changes IA, affects all screens, requires design system updates across both platforms. Deferred.
- *Floating action button*: iOS doesn't have a strong FAB convention. Rejected.
- *Inside settings/profile*: Too hidden — contradicts FR-001 ("not buried inside Library"). Rejected.

## Decision 5: Lesson Entity Scope

**Decision**: Lesson is a standalone entity with no item relationships in this iteration.

**Rationale**:
- Per spec, item linking is explicitly deferred to a follow-up feature.
- This keeps the data model simple: `lessons` table + `lesson_photos` table, no junction tables.
- The lesson entity can be extended with relationships later without breaking changes.

**Alternatives considered**:
- *Pre-build the junction table*: Adds unused schema. Rejected per YAGNI.
