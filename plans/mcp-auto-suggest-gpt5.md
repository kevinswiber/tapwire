### MCP auto-suggestions: deterministic sequence analysis and product surfacing

Date: 2025-08-10
Owner: Tapwire (Shadowcat)
Scope: Analyze recorded MCP traffic to propose server/client design improvements and surface actionable suggestions in-product

### Feasibility: how deterministic can this be?
- **Deterministic on recorded data**: Given complete session tapes, we can deterministically compute frequent sequences, co-occurrence, and latency distributions. Algorithms like PrefixSpan/FP-Growth on method names are deterministic for a fixed dataset and parameters.
- **Non-determinism across environments**: Recommendations generalize statistically (support/confidence thresholds) rather than absolutely. Upstream server versions, client UX, and transport affect patterns.
- **Bounded ambiguity**: We can restrict analysis to exact method names and argument-shape hashes to avoid false positives. Where args vary, we bucket by selectors (e.g., `resources/list?glob=**/*.md`).
- **Actionability gates**: Only emit suggestions when (a) support ≥ X% of sessions and (b) expected benefit exceeds a threshold (latency or call-count reduction), with confidence intervals.

### What we can analyze deterministically from tapes
- **Frequent subsequences**: e.g., `initialize` → `tools/list` → `resources/list` occurs in 82% of sessions within 1s.
- **Pairwise bundles**: e.g., `resources/list` immediately followed by N× `resources/read` for the top-K paths.
- **Redundant polls**: Same `tools/list` or `resources/list` repeated with identical results within TTL.
- **Cold-start waterfalls**: Identify first-N calls driving p95 latency; model potential bundling.
- **Transport churn**: SSE reconnect loops, stdio restarts, backpressure drops.
- **Error hotspots**: Specific methods or arg shapes returning `InvalidParams`, `NotFound`, `RateLimited`.
- **Payload inefficiency**: Repeated large fields not needed by the client (detected via downstream discard/use heuristics or size vs. reference usage).
- **Idempotency and retries**: Same `tools/call` reissued due to ambiguous timeouts; lack of idempotency keys.

### Potential improvements for MCP servers
- **Batch/describe endpoints**
  - Combine early discovery calls: a single `describe` method returning `tools`, `resources`, `prompts`, `models`, with ETags and partial fields.
  - Add `resources/list_with_contents` with filters for top-K hot files to avoid N× reads after listing.
- **Delta sync + caching**
  - Support `since`/cursor tokens on list endpoints; `If-None-Match` with ETags; `Cache-Control` TTLs.
  - Server-sent `catalog/update` notifications over SSE to avoid client polling.
- **Streaming and pagination**
  - Stream large lists incrementally and allow `fields=...` selection to reduce payload size.
- **Idempotency + error taxonomy**
  - Accept `Idempotency-Key` on mutating tool calls; return deterministic error codes with retry-after hints.
- **Rate-limit + backpressure hints**
  - Return `x-mcp-ratelimit-remaining`, `x-mcp-ratelimit-reset`, and `x-mcp-backpressure` headers/fields.
- **Precomputed indices**
  - Maintain server-side indices for common globs/content queries detected from traffic.

### Potential improvements for MCP clients
- **Warm-cache on initialize**
  - If server advertises `describe`/ETags, fetch once and reuse across sessions.
- **Batch and prefetch**
  - Coalesce `resources/read` for the first screenful after `resources/list`; prefetch top-K hot paths seen historically.
- **Adaptive retry and jitter**
  - Respect retry-after; exponential backoff with jitter for SSE reconnects.
- **Concurrency control**
  - Limit parallel `tools/call` to server-advertised concurrency; use token buckets.
- **Delta-aware polls**
  - Prefer `since`/`If-None-Match`; avoid full list refresh unless invalidated.
- **Idempotency keys**
  - Send keys for any retried `tools/call` to avoid duplicates.

### How this surfaces in Shadowcat (product)
- **Auto-Suggest Panel (UI + CLI)**
  - Shows ranked recommendations with:
    - **Insight**: e.g., "82% of sessions call `tools/list` + `resources/list` within 1s."
    - **Proposed change**: e.g., "Add `mcp.describe` returning tools/resources with ETags."
    - **Expected impact**: e.g., "-2 calls (-40%), -280ms p95 cold-start latency."
    - **Confidence**: support/confidence with sample size and lookback window.
  - CLI: `shadowcat analyze /path/to/tapes --suggest` outputs JSON and markdown.

- **What‑If Simulator**
  - Simulate bundling: recompute timeline if pairs are combined; estimate p50/p95 gains and bandwidth savings.

- **Anti‑Pattern Alerts**
  - Rules like: "Redundant `tools/list` with identical ETag within 60s" or ">3 SSE reconnects in 30s" trigger alerts with fixes.

- **Spec‑Aware Hints**
  - Link to MCP docs relevant to caching/delta sync/pagination. Flag where server advertises capabilities (via `initialize.capabilities`) but client does not use them.

- **Pull Request Drafts**
  - Generate templated PR/MR text for servers or clients:
    - Summary, motivation, suggested API additions, example code (pseudo), and telemetry from tapes.

- **Capability Matrix**
  - Per server: support for `describe`, `since` tokens, ETags, pagination, streaming, idempotency. Auto-detected from traffic.

### Candidate suggestions library (examples)
- **Bundle discovery**: Suggest `mcp.describe` if `initialize` → `tools/list` (+ optionally `resources/list`, `prompts/list`, `models/list`) occurs in ≥60% sessions.
- **List+N reads**: Suggest `resources/list_with_contents` if `resources/list` → ≥N `resources/read` on same glob occurs in ≥40% sessions within 2s.
- **Redundant list**: Suggest ETag/`If-None-Match` if identical list results observed ≥2× within 60s.
- **Payload trimming**: Suggest `fields` selection if average response size > P95 of peers and client uses only subset of fields.
- **SSE churn**: Suggest longer heartbeats/backoff if reconnect rate > threshold; propose jitter and `Retry-After` handling.
- **Idempotency**: Suggest keys if duplicate tool effects observed following timeouts.
- **Delta sync**: Suggest `since` cursors if list payloads are large and change rate is low.

### Metrics and thresholds (defaults)
- Lookback window: 7–30 days; minimum 200 sessions or 5k frames
- Support threshold: 40–60% depending on suggestion type
- Expected impact gate: ≥15% reduction in calls or ≥150ms p95 latency reduction for cold-start waterfalls
- Confidence: Wilson score interval lower bound > 0.6 for YES suggestions

### Implementation plan
- **Pipeline**
  - Offline analyzer: consume tapes → build per-session call sequences → mine frequent subsequences → compute opportunities.
  - Real-time preview: approximate metrics per active session for UI hints.
- **Data model**
  - Session graph: nodes=methods, edges=observed transitions with counts/latencies.
  - Catalog: normalized arg-shape hashes for grouping similar calls.
- **Surfacing**
  - API: `POST /analyze/suggest` returns structured suggestions with evidence and impact estimates.
  - UI: Auto-Suggest Panel; drill-down to sequences and raw frames.
  - CLI: `shadowcat analyze --suggest --format md > suggestions.md`.
- **Privacy & safety**
  - PII redaction before analysis; aggregate metrics only; opt-in sharing of anonymized aggregates; no secrets in suggestions.

### Risks and mitigations
- False positives → use strict bucketing and higher thresholds; show evidence and allow dismissal/feedback.
- Spec evolution → version‑gate suggestions; advertise server/client capabilities before recommending.
- Performance overhead → run analysis offline/async; bounded sampling in real-time.

### Success criteria
- ≥30% reduction in cold‑start p95 in pilot servers adopting suggestions
- ≥25% reduction in average calls per session for targeted sequences
- ≥70% acceptance rate of top 3 suggestions in pilot projects
- NPS uplift from developers on diagnosability and performance
