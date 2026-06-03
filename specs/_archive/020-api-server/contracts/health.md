# API Contract: Health Check

**Base Path**: `/api/health`

---

## Health Check

**Endpoint**: `GET /api/health`

Verifies that the server is running and can reach the database.

**Response** `200 OK` (healthy):
```json
{
  "status": "ok",
  "database": "ok"
}
```

**Response** `503 Service Unavailable` (database unreachable):
```json
{
  "status": "degraded",
  "database": "error"
}
```

---

## Notes

- The health check executes a simple database query (`SELECT 1`) to verify connectivity
- Fly.io's health check configuration (`fly.toml`) is pointed at this endpoint
- The endpoint does not require authentication
- Response status code (200 vs 503) is what Fly.io uses to determine instance health — the body is informational
