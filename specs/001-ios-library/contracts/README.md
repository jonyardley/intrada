# Contracts: iOS Library

**No new API contracts needed.**

The iOS library shell consumes the existing REST API via Crux HTTP effects.
All API contracts are defined in the Crux core (`intrada-core/src/http.rs`)
and the API server (`intrada-api/src/`).

## Existing API Endpoints Used

| Method | Endpoint | Core function | Purpose |
|--------|----------|--------------|---------|
| GET | `/api/items` | `fetch_items()` | Load all library items |
| POST | `/api/items` | `create_item()` | Create new item |
| PUT | `/api/items/{id}` | `update_item()` | Update existing item |
| DELETE | `/api/items/{id}` | `delete_item()` | Delete item |

## Shell Contract

The iOS shell's contract with the Crux core is the **Effect protocol**:

1. Shell receives `Effect.http(HttpRequest)` — contains URL, method, headers, body bytes
2. Shell executes the HTTP request, adding Clerk auth header
3. Shell returns `HttpResult.ok(HttpResponse)` or `HttpResult.err(HttpError)`
4. Shell receives `Effect.render` — reads updated ViewModel via BCS

The shell never needs to understand the JSON payload structure — that's
handled entirely in Rust by `serde_json` inside `crux_http`.

## Event Contract

| User Action | Event Dispatched |
|-------------|-----------------|
| App opens | `Event.startApp(apiBaseUrl)` |
| Tap filter tab | `Event.setQuery(ListQuery)` |
| Type in search | `Event.setQuery(ListQuery)` |
| Submit add form | `Event.item(.add(CreateItem))` |
| Submit edit form | `Event.item(.update(id, UpdateItem))` |
| Confirm delete | `Event.item(.delete(id))` |
