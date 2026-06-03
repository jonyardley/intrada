# API Contract Changes: User Authentication

## Authentication Requirement

All endpoints under `/api/items`, `/api/sessions`, `/api/routines` now require:

```
Authorization: Bearer <clerk_jwt_token>
```

The health endpoint (`GET /api/health`) remains public — no auth required.

## CORS Change

Add `Authorization` to allowed headers:

```
Before: allow_headers([Content-Type])
After:  allow_headers([Content-Type, Authorization])
```

## New Error Response: 401 Unauthorized

When the `Authorization` header is missing, malformed, or contains an invalid/expired JWT:

```
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{ "error": "Unauthorized" }
```

## Endpoint-by-Endpoint Contract

| Endpoint | Auth | user_id Scoping |
|----------|------|-----------------|
| `GET /api/health` | No | N/A |
| `GET /api/items` | Yes | `WHERE user_id = ?` |
| `POST /api/items` | Yes | Sets `user_id` on insert |
| `GET /api/items/{id}` | Yes | `WHERE id = ? AND user_id = ?` |
| `PUT /api/items/{id}` | Yes | `WHERE id = ? AND user_id = ?` |
| `DELETE /api/items/{id}` | Yes | `WHERE id = ? AND user_id = ?` |
| `GET /api/sessions` | Yes | `WHERE user_id = ?` |
| `POST /api/sessions` | Yes | Sets `user_id` on insert |
| `GET /api/sessions/{id}` | Yes | `WHERE id = ? AND user_id = ?` |
| `DELETE /api/sessions/{id}` | Yes | `WHERE id = ? AND user_id = ?` |
| `GET /api/routines` | Yes | `WHERE user_id = ?` |
| `POST /api/routines` | Yes | Sets `user_id` on insert |
| `GET /api/routines/{id}` | Yes | `WHERE id = ? AND user_id = ?` |
| `PUT /api/routines/{id}` | Yes | `WHERE id = ? AND user_id = ?` |
| `DELETE /api/routines/{id}` | Yes | `WHERE id = ? AND user_id = ?` |

## Request/Response Examples

### Authenticated Request

```
GET /api/items HTTP/1.1
Host: intrada-api.fly.dev
Authorization: Bearer eyJhbGciOiJSUzI1NiIs...
```

### Unauthenticated Request (rejected)

```
GET /api/items HTTP/1.1
Host: intrada-api.fly.dev

→ 401 Unauthorized
{ "error": "Unauthorized" }
```

### Health Check (always public)

```
GET /api/health HTTP/1.1
Host: intrada-api.fly.dev

→ 200 OK
```

## Auth-Optional Mode

When `CLERK_ISSUER_URL` is not set (dev/test), all requests pass through with `user_id = ""`. This matches the migration default and preserves existing test behavior.

## Handler Signature Change

```
Before: fn list_items(State(state): State<AppState>)
After:  fn list_items(State(state): State<AppState>, AuthUser(user_id): AuthUser)
```

## DB Function Signature Change

```
Before: fn list_items(conn: &Connection)
After:  fn list_items(conn: &Connection, user_id: &str)
```
