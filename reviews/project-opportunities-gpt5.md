### Shadowcat competitive landscape and ranked opportunities

Date: 2025-08-10
Owner: Tapwire (Shadowcat)
Scope: Forward/reverse MCP proxy, inspection, and traffic recording/replay

### Data sources used
- GitHub discovery (keywords: "mcp proxy", "model context protocol proxy/inspector/record"; organization: `modelcontextprotocol`)
- Repository metadata (stars, language, updatedAt)

### Competitive landscape (selected)

| Project | Stars | Lang | Focus | Notes |
|---|---:|---|---|---|
| `modelcontextprotocol/inspector` | 5,578 | TS | Visual inspector | Official GUI for testing MCP servers; not a network proxy or recorder
| `sparfenyuk/mcp-proxy` | 1,477 | Py | stdio↔︎SSE bridge | Popular simple bridge; no recording/observability
| `TBXark/mcp-proxy` | 421 | Go | Aggregator proxy (HTTP) | Multi-server hub via single HTTP endpoint
| `punkpeye/mcp-proxy` | 174 | TS | SSE proxy for stdio servers | Lightweight transport adapter
| `adamwattis/mcp-proxy-server` | 151 | TS | Aggregator/hub | Multi-server interface; basic
| `tidewave-ai/mcp_proxy_elixir` | 102 | Elixir | stdio↔︎SSE bridge | Focus on BEAM runtime; adapter only
| `tidewave-ai/mcp_proxy_rust` | 84 | Rust | stdio↔︎SSE bridge | Performance-centric bridge
| `openserv-labs/mcp-proxy` | 14 | TS | Reverse proxy + admin UI | Early admin UI; limited features
| `pangeacyber/pangea-mcp-proxy` | 5 | JS | Security filtering | Security guard; early stage
| `igrigorik/MCProxy` | 7 | Rust | Aggregation, filtering | Early; security-oriented framing
| `jaxxstorm/tailscale-mcp-proxy` | 6 | Go | Tailscale-aware proxy | Network deployment focus
| `gujord/OpenAPI-MCP` | 55 | Py | OpenAPI→MCP tools | Not a traffic proxy; useful as upstream

Related ecosystem signals
- `modelcontextprotocol/registry` (2,048★): discovery for servers; useful to integrate
- `modelcontextprotocol/use-mcp` (901★): client ergonomics; demand for easier wiring
- Proxies are fragmented across languages; most focus on transport bridging and simple aggregation rather than recording, replay, interception, or enterprise controls.

### Developer reception takeaways
- Visual inspection (official `inspector`) is widely adopted; developers value easy test surfaces.
- Simple stdio↔︎SSE bridges with minimal configuration see meaningful adoption (hundreds to 1.4k★) when they are easy to run and stable.
- Few projects attempt comprehensive recording/replay, policy enforcement, or observability; those are clear gaps.
- Early interest in security gateways (e.g., Pangea, Tramlines); opportunity to lead with robust, audited controls.

### Shadowcat positioning
- Strengths: Rust performance; forward/reverse proxy; session-centric design; recording/tape engine; rule-based interception; planned OAuth 2.1 gateway; observability and metrics; conformance and perf targets.
- Differentiators vs. current proxies: deep recording/replay, policy engine, audit/metrics, enterprise security, multi-transport abstraction beyond simple bridges.

### Ranked opportunities (impact-first, with rationale and quick win notes)

1) Recording + deterministic replay CLI and API
- Why: Almost no proxies provide robust session capture, redaction, export, and deterministic replay. High developer value for debugging and CI.
- What: `shadowcat record`/`replay` UX, tape schema, filters (PII redaction), timeline UI hooks.
- Signals to watch: CI usage, tape size, replay determinism rate.

2) First-class Aggregator Hub with Registry integration
- Why: Multi-server routing is common, but discovery and zero-config wiring are weak.
- What: Integrate `modelcontextprotocol/registry`; hot-reload server catalogs; per-route policies.
- Signals: Time-to-first-request, number of mounted servers, route error rate.

3) Security Gateway (OAuth 2.1, token isolation, RBAC, redaction)
- Why: Security proxies are nascent; enterprises need auditable controls and zero token leakage.
- What: OAuth 2.1, JWT audience checks, PKCE, resource-scoped tokens, redaction rules, deny/allow lists.
- Signals: Policy hits, blocked events, audit exports.

4) Observability and SLOs out of the box
- Why: Competing bridges lack metrics and tracing; performance is critical at scale.
- What: `tracing` spans, OpenTelemetry export, Prometheus metrics, p95 targets, flamegraphs.
- Signals: p95 latency overhead (<5%), error budgets, adoption of metrics endpoints.

5) Interception rules and programmable policies
- Why: Developers want to mutate/validate traffic; few tools offer robust policy engines.
- What: Declarative rules (YAML) + WASM/Lua hooks; dry-run mode; per-session overrides.
- Signals: Rule count per deployment, dry-run to enforce ratio, rollback frequency.

6) Admin UI for sessions, policies, and tapes
- Why: Admin UX differentiates beyond CLI; competitors have little or basic UI.
- What: Web UI to view live sessions, inspect frames, manage tapes/rules, diff replays.
- Signals: Weekly active admins, mean time to diagnose (MTTD), replay success.

7) Reliability features for SSE/stdio bridging at scale
- Why: Many bridges exist; reliability and backpressure handling are pain points.
- What: Auto-reconnect with jitter, circuit breakers, health checks, bounded buffers.
- Signals: Reconnect success rate, dropped frame rate, backpressure incidents.

8) Compatibility and conformance test suite
- Why: Trust requires conformance; easy vendor testing improves ecosystem fit.
- What: Publish tests; badges; nightly against popular servers; example fixes upstream.
- Signals: Passed checks, external projects adopting tests.

9) Deployment artifacts and templates
- Why: Adoption correlates with ease of run; competitors win on simplicity.
- What: Docker images, Helm chart, `docker-compose`, Terraform module, Heroku/Vercel examples.
- Signals: Pulls of images, chart installs, quickstart time.

10) Privacy-first recording (selective capture + redaction presets)
- Why: Recording is sensitive; built-in guardrails ease enterprise adoption.
- What: Field/path redaction, regex/tokenization, on-disk encryption, TTL retention.
- Signals: % tapes with redaction enabled, security reviews passed.

11) Ecosystem integrations (Inspector, Claude Desktop, IDEs, Tailscale)
- Why: Meet developers where they work; reduce friction.
- What: One-click connect to Inspector; desktop/IDE proxies; Tailscale ACL examples.
- Signals: Integration installs, success rate of plugin flows.

12) Differential replay and regression detection
- Why: Catch upstream server changes; unique beyond bridging.
- What: Replay same tape across versions/environments; diff responses; alerts.
- Signals: Diffs caught per week, false positive rate.

### Quick comparison summary
- Where others focus on transport bridging and simple hubs, Shadowcat can lead with: recording/replay, security/policy, and production-grade observability/reliability. Pair that with great DX (CLI + UI + deploy templates) to compete with simpler but popular bridges.
