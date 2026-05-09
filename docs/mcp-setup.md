# Connect intrada to your AI client

intrada exposes your library, sets, and practice sessions as a tool an
AI client can use on your behalf. Once connected, you can ask Claude
"what should I work on this week?" and have it read your recent
sessions, or "add Bach BWV 1007 movements I–VI to my library" and have
it create the items for you.

This page covers the three clients we've tested. The same setup works
with any MCP-compatible client.

- [Claude.ai (web)](#claudeai-web)
- [Claude Desktop](#claude-desktop)
- [Cursor](#cursor)
- [Other MCP clients](#other-mcp-clients)
- [Privacy and revoking access](#privacy-and-revoking-access)
- [Troubleshooting](#troubleshooting)

## Before you start

You need an intrada account. Sign in at <https://myintrada.com> and
make sure you can see your library — the AI client will only see what
you can see.

Two ways to authenticate:

- **OAuth** (claude.ai web). The client opens an intrada page, you
  click "Allow", and the client gets access. No copy-paste.
- **Personal Access Token** (Claude Desktop, Cursor, others). You
  generate a token in **Settings → MCP tokens**, copy it once, and
  paste it into the client's config. Treat it like a password.

Both produce the same level of access. Use whichever the client
supports.

---

## Claude.ai (web)

Claude.ai's "Custom connectors" feature uses OAuth, so you don't need
to copy a token.

1. In claude.ai, open **Settings → Connectors → Add custom connector**.
2. Set the URL to `https://intrada-api.fly.dev/api/mcp`.
3. Click **Add**. Claude.ai will redirect you to intrada to sign in
   (if you aren't already) and ask if you want to grant access.
4. Click **Allow**. You'll land back on claude.ai with the connector
   active.
5. Start a chat and try: *"List the pieces in my intrada library."*

If the dialog says intrada doesn't support OAuth, double-check the URL
has no trailing slash and points at `/api/mcp`, not `/mcp` or
`/oauth/authorize`.

---

## Claude Desktop

Claude Desktop reads a JSON config file at startup. It doesn't speak
OAuth, so you'll need a Personal Access Token.

1. In intrada, open **Settings → MCP tokens** and click **Generate
   token**. Give it a memorable name like "Claude Desktop on MacBook"
   so you can revoke just this one later.
2. Copy the token. **You won't see it again** after closing the
   confirmation — if you lose it, generate a new one.
3. Open Claude Desktop's config:
   - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
4. Add an `mcpServers` entry:

   ```jsonc
   {
     "mcpServers": {
       "intrada": {
         "url": "https://intrada-api.fly.dev/api/mcp",
         "headers": {
           "Authorization": "Bearer intrada_pat_..."
         }
       }
     }
   }
   ```

   Replace `intrada_pat_...` with the token you copied.
5. Quit and reopen Claude Desktop (the menu bar item, not just the
   window — fully quit so it re-reads the config).
6. Open a new chat. You should see a small tool indicator. Try:
   *"What did I practise last week?"*

If Claude Desktop doesn't see the server, check the config file is
valid JSON (a stray comma will silently disable the whole file).

---

## Cursor

Cursor's MCP support is similar to Claude Desktop — it reads a JSON
config at startup.

1. Generate a token in **Settings → MCP tokens** (as in the Claude
   Desktop section above).
2. In Cursor, open **Settings → MCP** and add a new server:

   ```jsonc
   {
     "intrada": {
       "url": "https://intrada-api.fly.dev/api/mcp",
       "headers": {
         "Authorization": "Bearer intrada_pat_..."
       }
     }
   }
   ```

3. Restart Cursor.

---

## Other MCP clients

If your client supports the standard MCP HTTP transport, point it at:

- **URL**: `https://intrada-api.fly.dev/api/mcp`
- **Auth**: HTTP `Authorization: Bearer <your token>` header.
- **Transport**: streamable HTTP, JSON-RPC 2.0.

For OAuth-aware clients, the discovery doc lives at
`https://intrada-api.fly.dev/.well-known/oauth-authorization-server`.

---

## What the AI client can see and do

Once connected, the client can:

- Read your library (pieces, exercises, sets).
- Read your practice sessions and weekly summaries.
- Create, edit, and delete items in your library.
- Bulk-import items (with a preview step before anything is written).

It cannot:

- See your account email, payment info, or other users' data.
- Sign you in or out.
- Change your password or revoke other tokens.

Every action runs on your account, attributed to the token (or OAuth
grant) the client used. You can review the audit log at any time —
**Settings → MCP tokens** lists last-used time per token.

## Privacy and revoking access

- **Tokens never leave your control.** intrada stores a hash, not the
  token itself — even our database can't replay it.
- **Revoke at any time.** **Settings → MCP tokens** has a Revoke
  button on every active token. The next request from that client
  fails immediately.
- **OAuth grants** appear in the same list, named after the client
  (e.g. "OAuth: Claude"). Revoke them the same way.
- **Rate limits**: each token can make 60 requests per minute. If a
  client gets noisy, you'll see it slow down rather than burn through
  your data.

## Troubleshooting

**The client connects but no tools show up.**
Restart the client fully. Most MCP clients only re-read tools at
startup.

**"Authentication failed" or 401 errors.**
Either the token is mistyped (check for stray spaces or a missing
prefix — it should start with `intrada_pat_`) or it's been revoked.
Generate a fresh one.

**"Rate limit exceeded" / 429 errors.**
The client is calling intrada more than 60 times a minute. Usually
it'll back off automatically. If it keeps happening, the client may
be misbehaving — revoke its token, file an issue with the client's
maintainer, and generate a new token when ready.

**OAuth flow loops or returns to a broken page.**
Clear cookies for `myintrada.com` and try again. If that doesn't
help, fall back to the Personal Access Token flow — most OAuth
clients also accept bearer tokens.

**Something else.**
Open an issue at <https://github.com/jonyardley/intrada/issues> with
the client name, the request that failed, and the error message.
