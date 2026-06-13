/**
 * Kotisatama backend: anonymous reports + fallback search telemetry.
 *
 * POST /report
 *   { "kind": "site_broken" | "suggest_site", "domain": "...", ... }
 *
 * POST /fallback
 *   { "query": "kela eläke", "platform": "desktop" | "android" }
 *
 * Env:
 *   REPORTS — KV for user reports
 *   FALLBACK — KV for fallback search queries
 *   WEBHOOK_URL — optional forward for reports
 */

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type",
};

const MAX_QUERY_LEN = 200;

function jsonResponse(body, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      ...CORS_HEADERS,
      "Content-Type": "application/json",
    },
  });
}

function isValidKind(kind) {
  return kind === "site_broken" || kind === "suggest_site";
}

function sanitizeQuery(raw) {
  if (typeof raw !== "string") {
    return null;
  }
  const query = raw.trim();
  if (!query || query.length > MAX_QUERY_LEN) {
    return null;
  }
  const lower = query.toLowerCase();
  if (lower.startsWith("http://") || lower.startsWith("https://") || lower.startsWith("data:")) {
    return null;
  }
  if (query.includes("@")) {
    return null;
  }
  return query;
}

async function handleReport(request, env) {
  let body;
  try {
    body = await request.json();
  } catch {
    return jsonResponse({ error: "invalid JSON" }, 400);
  }

  const { kind, domain, message, context_url } = body ?? {};

  if (!isValidKind(kind)) {
    return jsonResponse({ error: "invalid kind" }, 400);
  }

  if (typeof domain !== "string" || domain.trim().length === 0) {
    return jsonResponse({ error: "domain required" }, 400);
  }

  if (message !== undefined && typeof message !== "string") {
    return jsonResponse({ error: "message must be a string" }, 400);
  }

  if (context_url !== undefined && typeof context_url !== "string") {
    return jsonResponse({ error: "context_url must be a string" }, 400);
  }

  const record = {
    id: crypto.randomUUID(),
    kind,
    domain: domain.trim(),
    message: message?.trim() || null,
    context_url: context_url || null,
    received_at: new Date().toISOString(),
  };

  if (!env.REPORTS) {
    return jsonResponse({ error: "REPORTS KV not configured" }, 500);
  }

  await env.REPORTS.put(record.id, JSON.stringify(record));

  if (env.WEBHOOK_URL) {
    try {
      await fetch(env.WEBHOOK_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(record),
      });
    } catch (error) {
      console.error("webhook forward failed:", error);
    }
  }

  return jsonResponse({ ok: true, id: record.id });
}

async function handleFallback(request, env) {
  let body;
  try {
    body = await request.json();
  } catch {
    return jsonResponse({ error: "invalid JSON" }, 400);
  }

  const query = sanitizeQuery(body?.query);
  if (!query) {
    return jsonResponse({ error: "invalid query" }, 400);
  }

  const platform =
    body?.platform === "android" || body?.platform === "desktop" ? body.platform : "unknown";

  const record = {
    id: crypto.randomUUID(),
    query,
    platform,
    received_at: new Date().toISOString(),
  };

  if (!env.FALLBACK) {
    return jsonResponse({ error: "FALLBACK KV not configured" }, 500);
  }

  await env.FALLBACK.put(record.id, JSON.stringify(record));

  return jsonResponse({ ok: true, id: record.id });
}

export default {
  async fetch(request, env) {
    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: CORS_HEADERS });
    }

    if (request.method !== "POST") {
      return jsonResponse({ error: "method not allowed" }, 405);
    }

    const path = new URL(request.url).pathname.replace(/\/+$/, "") || "/";

    if (path.endsWith("/fallback")) {
      return handleFallback(request, env);
    }

    return handleReport(request, env);
  },
};
