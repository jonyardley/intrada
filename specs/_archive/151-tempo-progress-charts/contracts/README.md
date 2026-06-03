# Contracts: Tempo Progress Charts

No new API contracts. This feature is a client-side-only visualisation that consumes data already provided by existing endpoints:

- **GET /api/sessions** — returns sessions with entries including `achieved_tempo`
- **GET /api/items/:id** — returns item with `tempo` field (target BPM)

The chart component receives pre-processed data (`Vec<TempoHistoryEntry>` and `Option<u16>` target) as props from the detail view. No new endpoints, no new request/response shapes.
