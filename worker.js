// Cloudflare Worker entry. Default behaviour: serve the Trunk-built
// dist/ via the [assets] binding (configured in wrangler.toml). Two
// concerns are handled before that fallback:
//
// 1. `/sentry-tunnel` — forwards Sentry envelopes through our origin.
// 2. Prerendered marketing routes — `/`, `/login` are served from
//    `dist/prerendered/` so crawlers (and all users) get real HTML on
//    first fetch. WASM loads on top and takes over interactively.
//
// ## Why /sentry-tunnel exists
//
// Sentry's browser SDK posts events directly to `*.ingest.sentry.io`.
// Common content blockers (uBlock Origin, Brave Shields, AdGuard,
// Pi-hole) drop those requests, which silently disables our entire
// observability pipeline for any user with a blocker installed —
// `window.Sentry` would still be undefined too, but we already
// self-host the SDK from this same origin to dodge that.
//
// Tunnelling routes the envelope through our origin (`myintrada.com`)
// instead, so blockers don't see the Sentry destination. The SDK is
// configured with `tunnel: '/sentry-tunnel'` in `index.html`.
//
// ## Security: this MUST validate the envelope DSN
//
// Without validation we'd be an open relay — any origin could POST
// arbitrary envelopes here and we'd forward them to Sentry under our
// project's quota. The first newline-terminated line of every Sentry
// envelope is a JSON header that may include `dsn`; when present we
// parse it and refuse any envelope whose project_id doesn't match our
// own. When absent (Sentry's tunnel-mode SDK is allowed to omit it),
// we let it through — the worst case is the same open-relay risk that
// the body-size cap and (eventually) Cloudflare rate-limit address.
//
// See https://docs.sentry.io/platforms/javascript/troubleshooting/#dealing-with-ad-blockers

// Hard-coded project id — same one in the DSN baked into index.html.
// The DSN itself is public (Sentry's security model expects it embedded
// in client code), but pinning the project id here prevents an attacker
// from swapping in a different project's DSN to drain its quota or
// forge events.
const SENTRY_PROJECT_ID = "4511313298980944";
const SENTRY_INGEST_HOST = "o4511313186979840.ingest.de.sentry.io";
const SENTRY_INGEST_URL = `https://${SENTRY_INGEST_HOST}/api/${SENTRY_PROJECT_ID}/envelope/`;

// Hard cap on accepted body size. Real Sentry envelopes are typically
// well under 100 KB — events with stack traces, breadcrumbs, and
// contexts. We don't enable session-replay (which would push past
// 1 MB), so 1 MB is comfortable headroom and a 100x ceiling on what
// we expect. Anything larger is either misconfiguration or abuse;
// reject before paying for the buffer + Worker CPU.
const MAX_ENVELOPE_BYTES = 1_000_000;

// Sentry's ingest API only accepts this content type. Hardcode it
// rather than echoing the client's header — removes one layer of
// trust on attacker-controlled input.
const SENTRY_CONTENT_TYPE = "application/x-sentry-envelope";

// Marketing routes prerendered at build time. Map from public path to
// the asset path inside dist/prerendered/. Adding a new marketing page
// is a one-line change here + a matching entry in scripts/prerender.mjs.
const PRERENDERED = {
  "/": "/prerendered/index.html",
  "/login": "/prerendered/login.html",
};

// Trunk-hashed outputs only. copy-file/copy-dir paths (fonts/, static/sentry/,
// favicon.svg, etc.) aren't content-hashed and must not be marked immutable.
const HASHED_ASSET = /^\/(intrada-web|tailwind-output)-[0-9a-f]{8,}.*\.(js|wasm|css)$/;
const IMMUTABLE_CACHE = "public, max-age=31536000, immutable";
const NO_CACHE = "no-cache";

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.pathname === "/sentry-tunnel") {
      return tunnelToSentry(request);
    }

    // Serve prerendered HTML for marketing routes so crawlers see real
    // content without waiting for WASM. The SPA shell (index.html) is
    // still the fallback for all app routes via wrangler.toml's
    // not_found_handling = "single-page-application".
    const normalizedPath =
      url.pathname === "/" ? "/" : url.pathname.replace(/\/+$/, "");
    const prerendered = PRERENDERED[normalizedPath];
    if (prerendered) {
      const prerenderUrl = new URL(request.url);
      prerenderUrl.pathname = prerendered;
      const response = await env.ASSETS.fetch(new Request(prerenderUrl, request));
      return withCacheControl(response, NO_CACHE);
    }

    const response = await env.ASSETS.fetch(request);
    if (HASHED_ASSET.test(url.pathname)) {
      return withCacheControl(response, IMMUTABLE_CACHE);
    }
    if (url.pathname === "/" || url.pathname.endsWith(".html")) {
      return withCacheControl(response, NO_CACHE);
    }
    return response;
  },
};

function withCacheControl(response, value) {
  const headers = new Headers(response.headers);
  headers.set("Cache-Control", value);
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

async function tunnelToSentry(request) {
  // Same-origin POSTs from `myintrada.com` don't preflight, but a
  // misconfigured client (or the iOS WebView, if we ever extend the
  // tunnel there) might. Cheap to handle correctly upfront.
  if (request.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "POST, OPTIONS",
        "Access-Control-Allow-Headers": "content-type",
        "Access-Control-Max-Age": "86400",
      },
    });
  }
  if (request.method !== "POST") {
    // Sentry envelopes only ever POST. Reject everything else
    // explicitly so misconfigured clients fail loudly instead of
    // silently 404-ing into the SPA fallback.
    return new Response("Method Not Allowed", { status: 405 });
  }

  // Cheap reject before paying for `arrayBuffer()`. Content-Length is
  // advisory (clients can lie or omit), but it gates the obvious
  // amplification cases. The downstream `arrayBuffer()` is bounded by
  // Cloudflare's own request-size limits as a backstop.
  const declaredLength = Number(request.headers.get("Content-Length") || 0);
  if (declaredLength > MAX_ENVELOPE_BYTES) {
    return new Response("Payload Too Large", { status: 413 });
  }

  const body = await request.arrayBuffer();
  if (body.byteLength > MAX_ENVELOPE_BYTES) {
    return new Response("Payload Too Large", { status: 413 });
  }

  // Decode ONLY the first ~8 KB to find the header line. Decoding the
  // whole body would corrupt binary attachments (minidumps, profile
  // chunks, replay payloads) — none of which we currently emit, but
  // the bug is silent and would surface the day we flip a feature flag.
  const HEADER_PEEK = 8192;
  const headerSlice = body.slice(0, Math.min(body.byteLength, HEADER_PEEK));
  const headerText = new TextDecoder("utf-8", { fatal: false }).decode(
    headerSlice,
  );

  const newlineIndex = headerText.indexOf("\n");
  if (newlineIndex === -1) {
    return new Response("Bad envelope (no header line)", { status: 400 });
  }

  let header;
  try {
    header = JSON.parse(headerText.slice(0, newlineIndex));
  } catch {
    return new Response("Bad envelope (invalid JSON header)", { status: 400 });
  }

  // The DSN is recommended but not required in envelope headers;
  // tunnel-mode SDKs may omit it. When present, validate the project
  // id matches ours. When absent, allow through (we already pin the
  // ingest URL, so an attacker can't redirect anywhere; the body-size
  // cap above and a future Cloudflare rate-limit handle abuse).
  if (typeof header.dsn === "string") {
    let dsnProjectId;
    try {
      const dsnUrl = new URL(header.dsn);
      dsnProjectId = dsnUrl.pathname.replace(/^\/+/, "");
    } catch {
      return new Response("Bad envelope (invalid dsn)", { status: 400 });
    }
    if (dsnProjectId !== SENTRY_PROJECT_ID) {
      // Don't echo the supplied id back — keep error minimal so we
      // don't accidentally help an attacker probe for accepted ids.
      return new Response("Forbidden", { status: 403 });
    }
  }

  // Forward the original `body` ArrayBuffer untouched (binary-safe).
  // Build a fresh Headers object so client-supplied Cookie / Auth /
  // Origin / etc. don't leak to Sentry. Hardcode Content-Type — the
  // Sentry ingest only accepts this one value.
  return fetch(SENTRY_INGEST_URL, {
    method: "POST",
    headers: { "Content-Type": SENTRY_CONTENT_TYPE },
    body,
  });
}
