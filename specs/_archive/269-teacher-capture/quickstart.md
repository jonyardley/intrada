# Quickstart: Teacher Assignment Capture

## Prerequisites

- Rust stable (1.89.0+)
- `just` command runner
- Turso database with auth token
- Cloudflare R2 bucket (for photos)
- iOS: Xcode 16+, iOS 17+ simulator

## Verification Steps

### 1. Core compiles and tests pass

```bash
cargo test -p intrada-core
cargo clippy -p intrada-core -- -D warnings
```

Verify:
- [ ] Lesson events (create, update, delete, list, fetch) are handled
- [ ] Lesson model updates correctly on each event
- [ ] ViewModel includes lessons list and lesson detail
- [ ] Validation rejects future dates and oversized notes

### 2. API compiles and tests pass

```bash
cargo test -p intrada-api
```

Verify:
- [ ] Migrations run (lessons + lesson_photos tables created)
- [ ] POST /api/lessons creates a lesson and returns 201
- [ ] GET /api/lessons returns user-scoped lessons in reverse date order
- [ ] GET /api/lessons/:id returns a single lesson with photos
- [ ] PUT /api/lessons/:id updates lesson fields
- [ ] DELETE /api/lessons/:id removes lesson and cascades to photos
- [ ] POST /api/lessons/:id/photos accepts multipart upload
- [ ] DELETE /api/lessons/:id/photos/:photo_id removes photo
- [ ] All endpoints reject unauthenticated requests with 401
- [ ] All endpoints scope by user_id

### 3. Web shell renders lessons UI

```bash
trunk serve
```

Verify:
- [ ] "Log Lesson" entry point visible from Library screen
- [ ] Capture form shows date (today), notes field, photo upload
- [ ] Saving a lesson navigates to lesson list/detail
- [ ] Lesson list shows entries in reverse chronological order
- [ ] Lesson detail shows date, notes, photos
- [ ] Edit updates lesson, delete removes with confirmation
- [ ] Photo upload works via file picker
- [ ] Photo thumbnails display, tappable to full-size

### 4. iOS shell renders lessons UI

```bash
just typegen
just ios-swift-check
just ios-smoke-test
```

Verify:
- [ ] "Log Lesson" entry point visible from Library screen
- [ ] Capture form shows date (today), notes field, camera/photo picker
- [ ] Saving a lesson navigates to lesson list/detail
- [ ] Lesson list shows entries in reverse chronological order
- [ ] Lesson detail shows date, notes, photos
- [ ] Edit and delete work correctly
- [ ] Photo capture from camera works
- [ ] Photo selection from photo library works
- [ ] Photos display as thumbnails, tappable to full-size

### 5. E2E tests pass

```bash
cd e2e && npx playwright test
```

Verify:
- [ ] Create lesson with notes only
- [ ] Create lesson with photo only
- [ ] Edit lesson (change notes, change date)
- [ ] Delete lesson with confirmation
- [ ] Lesson list ordering is correct
- [ ] Lesson persists across page reload

### 6. Cross-platform parity

- [ ] Mobile and desktop layouts match Pencil designs
- [ ] iOS and web show the same data
- [ ] Design system tokens used throughout (no raw colours)
