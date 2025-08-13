# Tapwire — PRD for MCP Developer Proxies (Forward & Reverse), Recorder/Interceptor & Observability

**Owner:** You (Founder/Architect)  
**Doc status:** Draft v0.1 (August 3, 2025)  
**Product codename:** **Tapwire**  
**Audience:** Engineering, Product, Security, DevRel

---

## 1) Summary & Vision

**Problem.** MCP is quickly becoming the way IDEs/agents talk to tools (servers), but developers lack a *neutral*, protocol-aware proxy to **inspect, record/replay, intercept, and measure** MCP traffic across both transports (stdio and Streamable HTTP), with strong security guardrails.  [oai_citation:0‡Visual Studio Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)

**Vision.** Tapwire is the “**DevTools for MCP**” platform:
- **Forward proxy** for MCP clients (e.g., VS Code, Cursor, Windsurf) to **observe/shape** requests flowing to servers.  
- **Reverse proxy** that fronts remote MCP servers to **standardize auth**, **enforce security**, and **add telemetry** without modifying server code.  
- **Recorder/Replay** to capture **sessions** (with MCP-aware semantics) and **deterministically replay** conversations for debugging, tests, and demos.  
- **Interceptor** to override tool/resource/prompt outputs in-flight (mocking, fault injection), gated by role-based controls.

**Why now.** MCP standardized messaging (JSON-RPC), transports (stdio + Streamable HTTP), protocol versioning, and sessions, so proxies can be first-class: we can **segment by session**, infer **request/response types**, and **apply auth** consistently.  [oai_citation:1‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)

---

## 2) Background & Market Context

- **MCP transports & headers.** MCP defines **stdio** and **Streamable HTTP**, including **session management via `Mcp-Session-Id`** and a mandatory **`MCP-Protocol-Version`** header on HTTP. Streamable HTTP supersedes the older HTTP+SSE (with backward-compat guidance).  [oai_citation:2‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)
- **Security posture.** Spec calls out Origin validation, localhost binding for dev, auth on HTTP, and anti-DNS-rebind protections; it **forbids token passthrough** and mandates **OAuth2.1-style** audience validation.  [oai_citation:3‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports) [oai_citation:4‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)
- **Ecosystem demand.**  
  - **VS Code**: GA support (1.102+) for MCP servers and the full spec surface (tools/resources/prompts/sampling) across **stdio**, **SSE**, and **http** transports.  [oai_citation:5‡Visual Studio Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)  
  - **Cursor**: Documents MCP usage and encourages servers in any language.  [oai_citation:6‡Cursor](https://docs.cursor.com/context/model-context-protocol?utm_source=chatgpt.com)  
  - **Windsurf**: Supports **stdio** and **/sse**, notes SHTTP and MCP Auth support.  [oai_citation:7‡docs.windsurf.com](https://docs.windsurf.com/windsurf/cascade/mcp?utm_source=chatgpt.com)  
- **Existing tools** (MCP Inspector) are **test UIs**, not traffic-intercepting proxies—leaving a gap for Tapwire’s proxy/observability role.  [oai_citation:8‡GitHub](https://github.com/modelcontextprotocol/inspector)

---

## 3) Goals / Non-Goals

**Goals**
1. First-class **MCP-aware proxying** (forward & reverse) for stdio and Streamable HTTP.  
2. **Session-centric** traffic capture with secure **record/replay**.  
3. **Interactive interception** (rewrite/mocking/faults) with audit trails.  
4. **Metrics & tracing** (latency, tool call profiles, sampling usage).  
5. **Auth hardening** for remote servers: OAuth2.1 resource-server validation, **no token passthrough**.  [oai_citation:9‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)

**Non-Goals**
- Competing with IDEs/agents; Tapwire stays protocol-level.  
- Building an MCP server catalog.  
- Replacing MCP Inspector’s dev-UI; Tapwire integrates with it instead.  [oai_citation:10‡GitHub](https://github.com/modelcontextprotocol/inspector)

---

## 4) Personas & JTBD

- **Server author**: “When my server misbehaves with VS Code/Cursor, I need to **see exact JSON-RPC** and **replay** the same session to reproduce and fix.”  
- **Client integrator** (plugin/agent dev): “I want a **safe sandbox** to test auth flows, timeouts, cancellation, and streaming.”  
- **Security/Platform**: “I need a **reverse proxy** in front of remote MCP servers to **enforce OAuth**, **ban token passthrough**, and **log** usage.”

---

## 5) Use Cases (Top)

- **Record & deterministic replay** of a failing interaction across reconnects and sampling steps.  
- **Intercept/override** one tool output to simulate a backend outage.  
- **Auth gateway**: accept end-user auth, **mint resource-audience tokens**, block passthrough.  [oai_citation:11‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)  
- **Switch transports** (stdio ↔ http) while keeping the same session view.  [oai_citation:12‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)

---

## 6) Product Requirements

### 6.1 Protocol & Transport Compliance (Must)
- **Support transports:** **stdio** and **Streamable HTTP** (client POST+GET; optional SSE for streaming).  [oai_citation:13‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Session management:** Honor `Mcp-Session-Id` issuance/propagation; enforce 400 if required header missing; 404 on expired sessions; DELETE to terminate sessions.  [oai_citation:14‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Protocol versioning:** Forward/propagate `MCP-Protocol-Version: <date>`; generate helpful errors on invalid versions.  [oai_citation:15‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Resumability:** Respect SSE `id` and `Last-Event-ID` for redelivery during stream resume.  [oai_citation:16‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)

### 6.2 Security (Must)
- **Origin validation/localhost binding** defaults for dev; warnings when binding 0.0.0.0; DNS rebinding protection.  [oai_citation:17‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Auth hardening (reverse proxy):**  
  - Treat remote servers as **OAuth 2.1 resource servers**; validate audience; return 401 with `WWW-Authenticate` and discovery locations (RFC 9728).  
  - **Absolutely no token passthrough**; require token exchange with correct **`resource`** parameter.  [oai_citation:18‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)

### 6.3 Forward Proxy (Dev loop) (Must)
- **Live traffic viewer**: structured JSON-RPC, correlated **request/response/notifications**; streaming previews.  
- **Session timeline**: init → operations → shutdown; filters by tool/resource/prompt.  
- **Interception**: step-through (pause/modify/resume) for requests & responses; **rules** (match-and-rewrite) per server/tool/resource.  
  - Prior art: dev proxies support intercept/rewrite/map-local and repeat/replay.  [oai_citation:19‡docs.mitmproxy.org](https://docs.mitmproxy.org/stable/mitmproxytutorial-modifyrequests/?utm_source=chatgpt.com) [oai_citation:20‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/rewrite/?utm_source=chatgpt.com) [oai_citation:21‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/repeat/?utm_source=chatgpt.com)

### 6.4 Reverse Proxy (Gateway) (Must)
- **HTTP endpoint** that speaks SHTTP to clients; fans out to real server target (SHTTP or SSE legacy) with **backcompat** support.  [oai_citation:22‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **OAuth2.1 enforcement** (audience validation, AS discovery via RFC 9728), dynamic client registration, PKCE checks.  [oai_citation:23‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)  
- **Policy engine** (allow/deny tool calls; rate limits; redact logs).

### 6.5 Recorder & Replay (Must)
- **Non-repudiable “tapes”**: immutable capture of **ordered** client↔server frames + timing + transport edges + sampling checkpoints.  
- **Selective masking** (secrets) and **deterministic reply** mode that re-emits recorded server frames for client tests.  
- Motivation: industry proxies let you save conversations and replay requests.  [oai_citation:24‡Kali Linux](https://www.kali.org/tools/mitmproxy/?utm_source=chatgpt.com)

### 6.6 Intercept & Mock (Should)
- **Map-local** (serve local files as resource/tool outputs).  [oai_citation:25‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/map-local/?utm_source=chatgpt.com)  
- **Fault injection** (delay, drop, throttle); **Canned responses**; **Sticky overrides** per matcher.  
- **Audit**: every override is logged as **synthetic** with actor & reason.

### 6.7 Developer Experience (Must)
- **One-line setup** (env var or CLI) for stdio servers; **proxy URL** for HTTP servers.  
- **VS Code**: works with `.vscode/mcp.json`, gallery installs, discovery; supports **stdio/sse/http**, features (**tools/prompts/resources/sampling**).  [oai_citation:26‡Visual Studio Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)  
- **Cursor/Windsurf**: quickstarts and examples.  [oai_citation:27‡Cursor](https://docs.cursor.com/context/model-context-protocol?utm_source=chatgpt.com) [oai_citation:28‡docs.windsurf.com](https://docs.windsurf.com/windsurf/cascade/mcp?utm_source=chatgpt.com)

### 6.8 Telemetry & Tracing (Should)
- **Metrics**: P50/P95 latency per tool, error rates, retries, cancellations.  
- **Tracing**: spans for init, tool call, sampling step; export OTLP.

### 6.9 Compatibility (Must)
- **Backwards-compat** with legacy HTTP+SSE servers; smart probing/fallback per spec.  [oai_citation:29‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)

---

## 7) User Stories & Acceptance Criteria (selected)

1. **As a server author**, I can launch my server behind Tapwire (stdio) and see **Initialize → ServerCapabilities → Tools/Resources/Prompts**, then a tool call that streams partials—all shown as a **session**.  
   - **AC:** Session timeline shows **`Mcp-Session-Id`** when server issues it; subsequent requests contain it; removal causes 400; expired causes 404; deletion via HTTP `DELETE` terminates session.  [oai_citation:30‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)

2. **As a client integrator**, I can **pause** a server response, edit the JSON result, and **resume**—or set a rule so future matching calls are automatically overridden.  
   - **AC:** Intercept UI supports *pause/edit/resume* and **rule** creation. (Comparable features exist in established dev proxies.)  [oai_citation:31‡docs.mitmproxy.org](https://docs.mitmproxy.org/stable/mitmproxytutorial-modifyrequests/?utm_source=chatgpt.com) [oai_citation:32‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/rewrite/?utm_source=chatgpt.com)

3. **As a platform engineer**, I can configure the reverse proxy with **OAuth**: on a request without/invalid token, Tapwire returns 401 with `WWW-Authenticate` pointing to resource metadata; when token audience≠server, Tapwire rejects; Tapwire never forwards client tokens upstream.  [oai_citation:33‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)

4. **As a QA**, I can **record** a failing flow and **replay** it deterministically without the real server.  
   - **AC:** Replay emits the same frames & SSE `id` sequence, honoring `Last-Event-ID` resume semantics.  [oai_citation:34‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)

---

## 8) UX & IA (high level)

- **Left:** Session list / filters. **Center:** Timeline + message viewer (raw/pretty). **Right:** Context (server info, auth, rules).  
- **Modes:** *Live*, *Record*, *Replay*, *Intercept*, *Metrics*.

---

## 9) Data Model (selected)

- **Session**: ids (server-issued and Tapwire), start/end, transport, protocol version, auth state.  [oai_citation:35‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Frame**: direction, method (e.g., `tools/call`, `resources/read`, `prompts/get`, `sampling/createMessage`), id, headers, body, t-stamps.  [oai_citation:36‡Model Context Protocol](https://modelcontextprotocol.io/specification/2025-06-18/server/tools?utm_source=chatgpt.com) [oai_citation:37‡Model Context Protocol](https://modelcontextprotocol.io/specification/2025-03-26/server/resources?utm_source=chatgpt.com) [oai_citation:38‡Model Context Protocol](https://modelcontextprotocol.io/specification/2025-06-18/server/prompts?utm_source=chatgpt.com) [oai_citation:39‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/sampling?utm_source=chatgpt.com)  
- **Rule**: matcher (server/tool/resource/prompt + JSON-path), action (rewrite/mock/fault), scope.

---

## 10) Architecture

- **Forward proxy (stdio)**: Tapwire process spawns target server as child; stdin/stdout shim logs and optionally intercepts frames; stderr is logs.  [oai_citation:40‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Forward proxy (HTTP/SHTTP)**: Tapwire endpoint terminates client requests, maintains session map, handles POST/GET, SSE streaming, `Last-Event-ID` resume.  [oai_citation:41‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Reverse proxy**: Validates OAuth (audience, metadata discovery, PKCE), enforces no-passthrough, exchanges tokens for upstream as needed.  [oai_citation:42‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)

---

## 11) Non-Functional Requirements

- **Security defaults:** Origin checks, localhost bind, CORS expose of session header (`Mcp-Session-Id`) for web UIs.  [oai_citation:43‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports) [oai_citation:44‡GitHub](https://github.com/modelcontextprotocol/typescript-sdk)  
- **Reliability:** Replayable tapes; resumable streams; graceful backoff on disconnects.  [oai_citation:45‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Performance:** Proxy adds <5% p95 latency overhead for typical `tools/call` flows (target).

---

## 12) MVP Scope

**Must:**  
- stdio & SHTTP proxying; sessionization; live viewer; manual intercept; tape record/replay; OAuth validation (aud/audiences, discovery, no passthrough); metrics v1.  [oai_citation:46‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports) [oai_citation:47‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)  

**Should:**  
- Rules engine; map-local; fault injection; VS Code helper & examples.  [oai_citation:48‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/map-local/?utm_source=chatgpt.com)

**Won’t (v1):**  
- Cloud multi-tenant UI; team RBAC; advanced tracing UI.

---

## 13) Success Metrics

- **Time-to-fix** for a repro (first reproduce to validated fix) ↓ 50%.  
- **Auth misconfig incidents** at the edge ↓ 80%.  
- **Adoption**: 100+ weekly active developers within 60 days.

---

## 14) Packaging & Pricing

| Tier | Who | Included |
|---|---|---|
| **Free** | Solo devs | Local stdio/SHTTP proxy; live viewer; **manual** intercept; basic tapes (local); 7-day recent session list; local metrics. *(Analogous dev proxies expose intercept/rewrite basics for free.)*  [oai_citation:49‡docs.mitmproxy.org](https://docs.mitmproxy.org/stable/mitmproxytutorial-modifyrequests/?utm_source=chatgpt.com) [oai_citation:50‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/rewrite/?utm_source=chatgpt.com) |
| **Pro** | Teams | All Free + **rules engine**, **map-local**, **fault injection**, OAuth enforcement (single AS), project workspaces, tape sharing, CI replay, Grafana/OTLP export.  [oai_citation:51‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/map-local/?utm_source=chatgpt.com) |
| **Enterprise** | Platform/Sec | All Pro + **policy** (ABAC), multiple AS/tenants, SSO, audit export, long-term retention, HA gateway.

---

## 15) Cost–Value Matrix (qualitative)

| Feature | Dev Value | Build Cost |
|---|---|---|
| Live viewer + sessionization | Very High | Medium |
| Record & deterministic replay | Very High | Medium–High |
| Manual intercept (pause/edit/resume) | High | Medium |
| Rules engine (rewrite/mock/fault) | High | High |
| Reverse proxy OAuth gateway | High | High |
| Metrics (p95, errors) | Medium | Low–Medium |
| OTLP export | Medium | Low |
| VS Code helpers (mcp.json snippets) | Medium | Low |

*(Plot: prioritize top-right: viewer, record/replay, intercept; invest early in OAuth gateway as a differentiator.)*

---

## 16) Competitive & Prior Art

- **MCP Inspector**: excellent **visual tester/bridge**; explicitly **not** a network traffic-intercepting proxy—Tapwire complements it and can even open Inspector against recorded servers.  [oai_citation:52‡GitHub](https://github.com/modelcontextprotocol/inspector)  
- **General web proxies** (Charles, mitmproxy, Burp) show dev appetite for **interception, rewrite/map-local, and replay**—we adapt those affordances to MCP semantics.  [oai_citation:53‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/rewrite/?utm_source=chatgpt.com) [oai_citation:54‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/map-local/?utm_source=chatgpt.com) [oai_citation:55‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/repeat/?utm_source=chatgpt.com)

---

## 17) Risks & Mitigations

- **Spec churn** (e.g., SHTTP replacing HTTP+SSE): track protocol revs and support backcompat detection paths.  [oai_citation:56‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Security pitfalls**: enforce **no token passthrough**, audience validation, and AS discovery per spec; ship secure defaults.  [oai_citation:57‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)  
- **Transport edge cases**: honor SSE resume (`Last-Event-ID`) and 1-stream semantics.  [oai_citation:58‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)

---

## 18) Open Questions

- Should Tapwire ship a **lightweight recorder-only** CLI for CI (no proxy)?  
- Which sampling controls (user approval gates) should Tapwire surface for clients that expose sampling?  [oai_citation:59‡Visual Studio Code](https://code.visualstudio.com/api/extension-guides/ai/mcp) [oai_citation:60‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/sampling?utm_source=chatgpt.com)

---

## 19) Rollout Plan

1. **Alpha (4–6 weeks)**: stdio proxy + viewer + basic record/replay + manual intercept; VS Code sample workspace.  
2. **Beta**: SHTTP support (streaming, resume), reverse proxy with OAuth validation, rules engine v1.  
3. **GA**: policy packs, dashboards, enterprise auth integrations.

---

## 20) Test Plan (high level)

- **Conformance**: run against official **TypeScript SDK** sample servers and verify headers (`Mcp-Session-Id`, `MCP-Protocol-Version`), SHTTP/SSE behavior.  [oai_citation:61‡GitHub](https://github.com/modelcontextprotocol/typescript-sdk)  
- **Auth**: present invalid/missing tokens → 401 with `WWW-Authenticate`; audience mismatch → 403/401 per spec; ensure **no upstream passthrough**.  [oai_citation:62‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)  
- **Replay**: verify byte-wise equality for replayed responses; resume from `Last-Event-ID`.  [oai_citation:63‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)

---

## 21) Documentation & DevRel

- **Guides**: “Proxying an MCP server (stdio/SHTTP)”, “Recording & Replaying”, “Intercept Rules Cookbook”.  
- **IDE how-tos**: VS Code `.vscode/mcp.json`, installed servers, transports & features.  [oai_citation:64‡Visual Studio Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)

---

## 22) References (select)

- **Transports / Sessions / Version header / Backcompat** — Model Context Protocol.  [oai_citation:65‡Model Context Protocol](https://modelcontextprotocol.io/docs/concepts/transports)  
- **Authorization & Token-passthrough prohibition** — MCP Authorization spec & Best Practices.  [oai_citation:66‡Model Context Protocol](https://modelcontextprotocol.io/specification/draft/basic/authorization)  
- **VS Code MCP support (GA in 1.102), transports & features**.  [oai_citation:67‡Visual Studio Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)  
- **Cursor & Windsurf docs** (client adoption).  [oai_citation:68‡Cursor](https://docs.cursor.com/context/model-context-protocol?utm_source=chatgpt.com) [oai_citation:69‡docs.windsurf.com](https://docs.windsurf.com/windsurf/cascade/mcp?utm_source=chatgpt.com)  
- **Inspector is not an intercepting proxy** (complementary tool).  [oai_citation:70‡GitHub](https://github.com/modelcontextprotocol/inspector)  
- **Prior-art features** (intercept/rewrite/map-local/replay).  [oai_citation:71‡docs.mitmproxy.org](https://docs.mitmproxy.org/stable/mitmproxytutorial-modifyrequests/?utm_source=chatgpt.com) [oai_citation:72‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/rewrite/?utm_source=chatgpt.com) [oai_citation:73‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/repeat/?utm_source=chatgpt.com) [oai_citation:74‡charlesproxy.com](https://www.charlesproxy.com/documentation/tools/map-local/?utm_source=chatgpt.com)
