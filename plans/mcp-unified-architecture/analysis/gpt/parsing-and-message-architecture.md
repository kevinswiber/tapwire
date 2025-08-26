Love where you’re headed. Three quick clarifications up front:
	•	Result<<dyn Diagnostics<T>>, DiagnosticsError> won’t compile: trait objects can’t be generic like that, and you generally don’t want a success type that’s just “diagnostics”. Use a concrete wrapper that carries both the value (if any) and diagnostics.
	•	If you want zero-copy, avoid storing borrowed typed values in Box<dyn Any> — Any is only implemented for 'static types (i.e., owned data).  ￼
	•	Hyper/Tokio already give you streaming + backpressure; when you add your own queues, keep them bounded.  ￼

Below is a compact, drop-in planning guide plus a visitor-style registry sketch that borrows from input, produces diagnostics, and can pass mutations back using Bytes.

⸻

MCP Proxy & mcpspec: Diagnostics-First, Registry-Driven Plan

1) Wire model (fast + flexible by default)
	•	Parse the JSON-RPC envelope into a small typed “frame”. Keep params/result unparsed as &serde_json::value::RawValue to defer work and preserve exact bytes for pass-through/replay.  ￼
	•	When you do decode, use borrowed deserialization (#[serde(borrow)]) to avoid allocations.  ￼

Why: RawValue is purpose-built to “defer or avoid parsing” and retain original formatting; perfect for a proxy/interceptor.  ￼

2) Registry-driven decoding (A-base) with a visitor pattern
	•	Keep a Registry keyed by method → handler. Each handler decides whether to:
	1.	Leave it raw and pass through,
	2.	Borrow-deserialize into a typed view, and/or
	3.	Mutate the payload (producing new Bytes).
	•	Prefer the visitor style (handlers do work immediately) instead of returning typed values you’d have to store as Any — that preserves zero-copy options and avoids 'static constraints.  ￼

3) Diagnostics API (always available)
	•	You’ll want diagnostics even on success. Use a success wrapper rather than making success a trait object:

pub struct Diag {
  pub code: &'static str,        // e.g., "schema.pattern"
  pub severity: Severity,        // info | warning | error
  pub path: String,              // JSON Pointer to offending spot
  pub message: std::borrow::Cow<'static, str>,
  pub data: Option<serde_json::Value>,
}
pub enum Severity { Info, Warning, Error }

pub struct WithDiags<T> {
  pub value: T,                  // often (), or a mutation artifact
  pub diags: Vec<Diag>,
}

// “passes with warnings” vs “fails with errors”
pub type DiagResult<T> = Result<WithDiags<T>, DiagError>;

	•	Use JSON Pointer for path so tooling can jump to exact fields.  ￼
	•	For ergonomic severities, the LSP severities (Error/Warning/Information/Hint) are a good model.  ￼
	•	When deserializing typed params, wrap the deserializer with serde_path_to_error so you can populate path precisely.  ￼

4) Strict vs lenient (runtime policy)
	•	Lenient (proxy default): unknown methods → pass through; type errors → accept but attach diagnostics.
	•	Strict (mcpspec default): unknown/invalid → JSON-RPC errors (-32700, -32600, -32601, -32602, -32603). MCP itself says “messages MUST follow JSON-RPC 2.0,” so strict mode is your conformance oracle.  ￼

5) JSON Schema in mcpspec (optional layer)
	•	Precompile schemas with the jsonschema crate and collect all violations using its error iterator; then reformat into your Diag shape with JSON Pointers.  ￼ ￼
	•	Schema output can be terse; your formatter can translate keyword-level errors into human-readable messages but still keep the pointer for machine tooling.

6) Buffering & backpressure
	•	Hyper/Tokio bodies are streaming and exert backpressure by only reading when polled — don’t buffer more than necessary.  ￼
	•	Wherever you queue internally, use bounded mpsc::channel — unbounded queues can explode memory; Tokio’s docs explicitly recommend bounded for backpressure.  ￼ ￼
	•	For bytes you do hold, use bytes::Bytes / BytesMut to share and mutate without extra copies.  ￼

⸻

Compact Visitor-Style Registry (borrow, diagnose, mutate with Bytes)

The sketch below shows:
(1) envelope with RawValue,
(2) registry + visitor handler,
(3) borrowed deserialization (#[serde(borrow)]),
(4) diagnostics from serde_path_to_error,
(5) optional mutation returned as Bytes.

use bytes::{Bytes, BytesMut};
use serde::Deserialize;
use serde_json::value::RawValue;
use std::{collections::HashMap, sync::Arc};

// ---------- Diagnostics ----------
#[derive(Debug)]
pub enum Severity { Info, Warning, Error }

#[derive(Debug)]
pub struct Diag {
    pub code: &'static str,
    pub severity: Severity,
    pub path: String,        // JSON Pointer
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct WithDiags<T> { pub value: T, pub diags: Vec<Diag> }

pub type DiagError = Vec<Diag>;                    // fatal problems
pub type DiagResult<T> = Result<WithDiags<T>, DiagError>;

// ---------- Wire envelope (JSON-RPC subset) ----------
#[derive(Deserialize)]
pub struct Frame<'de> {
    pub jsonrpc: &'de str,
    pub id: Option<serde_json::Value>,
    pub method: Option<&'de str>,
    #[serde(borrow)]
    pub params: Option<&'de RawValue>,             // borrow, no parse
    #[serde(borrow)]
    pub result: Option<&'de RawValue>,
    pub error: Option<serde_json::Value>,
}

// ---------- Interceptor context ----------
pub struct Ctx {
    /// Original message bytes; safe to clone (ref-counted)
    pub msg_bytes: Bytes,
    /// If a handler wants to replace the message body, set this.
    pub out: Option<Bytes>,
    /// Collected non-fatal diagnostics
    pub diags: Vec<Diag>,
    /// Strict vs lenient
    pub strict: bool,
}

impl Ctx {
    pub fn replace_body(&mut self, new_body: Bytes) { self.out = Some(new_body); }
    pub fn warn(&mut self, path: impl Into<String>, code: &'static str, msg: impl Into<String>) {
        self.diags.push(Diag { code, severity: Severity::Warning,
                               path: path.into(), message: msg.into(), data: None });
    }
}

// ---------- Handler trait (visitor) ----------
pub trait MethodHandler: Send + Sync {
    fn handle<'de>(&self, frame: &Frame<'de>, ctx: &mut Ctx) -> DiagResult<()>;
}

pub struct Registry {
    map: HashMap<&'static str, Arc<dyn MethodHandler>>,
}

impl Registry {
    pub fn new() -> Self { Self { map: HashMap::new() } }

    pub fn register<H: MethodHandler + 'static>(&mut self, method: &'static str, h: H) {
        self.map.insert(method, Arc::new(h));
    }

    pub fn dispatch<'de>(&self, frame: &Frame<'de>, ctx: &mut Ctx) -> DiagResult<()> {
        if let Some(m) = frame.method {
            if let Some(h) = self.map.get(m) {
                h.handle(frame, ctx)
            } else if ctx.strict {
                Err(vec![Diag {
                    code: "jsonrpc.methodUnknown",
                    severity: Severity::Error,
                    path: "/method".into(),
                    message: format!("Unknown method: {m}"),
                    data: None,
                }])
            } else {
                ctx.warn("/method", "jsonrpc.methodUnknown", format!("Unknown method: {m}"));
                Ok(WithDiags { value: (), diags: std::mem::take(&mut ctx.diags) })
            }
        } else {
            // Response or notification w/o method: let other code handle
            Ok(WithDiags { value: (), diags: std::mem::take(&mut ctx.diags) })
        }
    }
}

// ---------- Example: a borrowed-params handler ----------
#[derive(Deserialize)]
struct PingParams<'a> {
    #[serde(borrow)]
    message: Option<&'a str>,   // zero-copy borrow from RawValue
}

struct Ping;

impl MethodHandler for Ping {
    fn handle<'de>(&self, frame: &Frame<'de>, ctx: &mut Ctx) -> DiagResult<()> {
        let Some(raw) = frame.params else {
            return Ok(WithDiags { value: (), diags: std::mem::take(&mut ctx.diags) });
        };

        // Build a serde_json deserializer over the *borrowed* slice
        // from RawValue. Any failure gets a JSON Pointer path via
        // serde_path_to_error.
        let mut de = serde_json::Deserializer::from_str(raw.get());
        let ping: PingParams<'de> = match serde_path_to_error::deserialize(&mut de) {
            Ok(v) => v,
            Err(err) => {
                let path = err.path().to_string();
                if ctx.strict {
                    return Err(vec![Diag {
                        code: "params.invalid",
                        severity: Severity::Error,
                        path,
                        message: err.to_string(),
                        data: None,
                    }]);
                } else {
                    ctx.warn(path, "params.invalid", err.to_string());
                    return Ok(WithDiags { value: (), diags: std::mem::take(&mut ctx.diags) });
                }
            }
        };

        // Optional mutation example: inject a default message if missing
        if ping.message.is_none() {
            // Build a tiny replacement JSON. In real code you might
            // patch only params; here we replace the whole message body.
            let new_obj = serde_json::json!({
                "jsonrpc": "2.0",
                "id": frame.id.clone().unwrap_or(serde_json::Value::Null),
                "method": "ping",
                "params": { "message": "default" }
            });
            let mut buf = BytesMut::new();
            serde_json::to_writer((&mut buf).writer(), &new_obj).unwrap();
            ctx.replace_body(buf.freeze());          // pass to write-side
        }

        Ok(WithDiags { value: (), diags: std::mem::take(&mut ctx.diags) })
    }
}

Notes:
	•	Frame borrows from the input buffer; RawValue::get() returns the borrowed JSON slice; with #[serde(borrow)] your typed params can borrow from it — zero allocations.  ￼ ￼
	•	serde_path_to_error emits the exact path where decoding failed — perfect for Diag.path (JSON Pointer).  ￼ ￼
	•	Mutations create a new Bytes (using BytesMut + serde_json::to_writer), which the write-side can pick up. Bytes/BytesMut keep copies minimal and are cheap to clone/slice.  ￼

⸻

Diagnostics: shaping and surfacing
	•	Shape (suggested): {code, severity, path, message, data?}.
	•	Derive path from serde_path_to_error or from schema error locations; prefer JSON Pointer so tools can jump to fields.  ￼ ￼
	•	For severity levels, mirroring LSP (Error/Warning/Information/Hint) makes sense and is familiar to devs.  ￼
	•	Transporting diagnostics:
	•	Non-fatal: include as side-channel notification or attach to results (like LSP’s publishDiagnostics).  ￼
	•	Fatal: map to JSON-RPC error object; in strict mode use canonical codes. (MCP requires JSON-RPC 2.0 envelopes.)  ￼

⸻

Performance checklist
	•	Parse only when needed (RawValue everywhere else).  ￼
	•	Borrow when parsing (#[serde(borrow)]) to avoid allocations.  ￼
	•	Bound queues (Tokio mpsc::channel) to enforce backpressure; avoid unbounded.  ￼ ￼
	•	Streaming bodies (Hyper) and small buffers; don’t accidentally buffer whole messages unless you’re mutating.  ￼
	•	Bytes/BytesMut for any in-memory staging or rewrite.  ￼

⸻

mcpspec add-ons (beyond the proxy)
	•	JSON Schema validation (compiled): run in parallel with registry handlers, collect all errors, and map them into Diags for human-readable output.  ￼
	•	Optionally publish an OpenRPC document so external clients/tools understand your method set; mcpspec can verify the OpenRPC → behavior alignment.  ￼

⸻

FAQ, based on your notes
	•	“Do I still need a buffer?”
Yes, a small one. You’re reading → maybe mutating → writing. Keep buffers bounded and rely on Hyper/Tokio backpressure for the rest.  ￼
	•	“Will Any force copies?”
If you store decoded values behind Box<dyn Any>, those values must be 'static (owned). That generally means copies. Use the visitor approach when you want borrowed, zero-copy typed views.  ￼
	•	“Can I preserve unknown fields?”
For the few places you do fully type, add #[serde(flatten)] extras: HashMap<String, Value> so you never lose data.  ￼

⸻

If you want, I can turn this into a small crate skeleton (mcp-core, mcp-validate, mcp-intercept) with the registry, diagnostics, and strict/lenient wiring pre-built so you can drop it into the proxy and mcpspec.