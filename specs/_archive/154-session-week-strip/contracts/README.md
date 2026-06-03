# API Contracts: Session Week Strip Navigator

**Feature**: 154-session-week-strip

## No New API Contracts

This feature is a **pure frontend refactor** of the Sessions page. It filters and displays existing session data that is already fetched from the API.

### Existing Endpoints Used (unchanged)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/sessions` | GET | Fetch all sessions (already called on app init) |
| `/api/sessions/:id` | DELETE | Delete a session (existing functionality preserved) |

### Why No New Endpoints

- All session data is already available in `ViewModel.sessions` after the initial fetch
- Week-based filtering is a client-side operation on the in-memory dataset
- Adding server-side week filtering would add latency without benefit (dataset is small enough for client-side processing)
- The spec explicitly calls for client-side date grouping using `started_at` timestamps
