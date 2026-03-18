# Contracts: iOS Session Builder

No new API contracts are needed for this feature.

The iOS session builder is a pure shell — it dispatches Crux events and renders the ViewModel. All HTTP communication with the API is handled by the Crux core's `crux_http` capability, which builds and executes requests internally. The iOS shell only processes generic `HttpRequest` effects (URL, method, headers, body bytes) and returns `HttpResponse` (status, body bytes).

## Event Contract (iOS → Crux Core)

See [data-model.md](../data-model.md) for the full list of events dispatched and ViewModel fields consumed.

## Existing API Endpoints Used (via Crux core)

- `POST /api/sessions` — create a new session (dispatched by core on `StartSession`)
- `GET /api/sessions` — fetch session history (dispatched by core on app start)
- `GET /api/items` — fetch library items (dispatched by core on app start)
- `GET /api/routines` — fetch routines (dispatched by core on app start)
- `POST /api/routines` — save routine (dispatched by core on save routine event)
