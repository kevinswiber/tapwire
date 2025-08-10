# OAuth 2.1 and Transport Security

Assessment of OAuth flows, token verification, bind/origin behavior, and related transport protections.

## Findings

### OAuth 2.1 components
- PKCE generation and verification:
```1:47:shadowcat-delta/src/auth/pkce.rs
pub enum PKCEMethod { S256, Plain }
impl PKCEMethod { ... }
#[derive(Debug, Clone)]
pub struct PKCEChallenge { verifier, challenge, method }
impl PKCEChallenge {
  pub fn generate() -> Result<Self> { ... S256 ... }
  fn compute_challenge(...)
  pub fn verify(&self, verifier: &str) -> Result<bool> { ... }
}
```

- OAuth client uses PKCE when required; enforces state:
```212:250:shadowcat-delta/src/auth/oauth.rs
pub fn generate_authorization_url(&self) -> AuthResult<AuthorizationRequest> { ...
  if self.config.pkce_required { Some(PKCEChallenge::generate()?) }
  query_pairs.append_pair("state", &state);
  if let Some(ref pkce) = pkce { code_challenge + code_challenge_method }
}
```
```253:309:shadowcat-delta/src/auth/oauth.rs
pub async fn exchange_code_for_tokens(...) -> AuthResult<TokenResponse> {
  if state != expected_state { return Err(...); }
  if let Some(verifier) = pkce_verifier { params.push(("code_verifier", verifier)); }
  // posts to token_endpoint and parses TokenResponse
}
```

- JWT validation and JWKS retrieval:
```246:291:shadowcat-delta/src/auth/token.rs
validate_token(&self, token) { decode_header; enforce alg allowlist; iss/aud; exp/nbf/leeway; get_decoding_key(...); decode(...); }
```
```379:427:shadowcat-delta/src/auth/token.rs
async fn fetch_jwks_key(...) { reqwest::Client::new().get(jwks_uri) ...; parse JWKS; pick key by kid; build DecodingKey }
```

### Transport security
- Default bind is loopback (good dev default):
```170:176:shadowcat-delta/src/proxy/reverse.rs
bind_address: "127.0.0.1:8080" .parse().expect(...)
```
```236:249:shadowcat-delta/src/config/reverse_proxy.rs
ServerSettings { bind_address: "127.0.0.1:8080", cors_enabled: true, ... }
```
```15:16:shadowcat-delta/src/cli/reverse.rs
#[arg(long, default_value = "127.0.0.1:8080")] pub bind: String,
```

- HTTP MCP header extraction validates protocol version and requires `MCP-Session-Id`:
```47:98:shadowcat-delta/src/transport/http_mcp.rs
pub fn extract_mcp_headers(headers: &HeaderMap) -> ReverseProxyResult<McpHeaders> { ... require mcp-session-id; check version support; optional mcp-client-info }
```

- Use of X-Forwarded-* for client IP without trusted proxy boundary awareness (risk):
```119:152:shadowcat-delta/src/rate_limiting/key_extractors.rs
// reads x-forwarded-for, x-real-ip, cf-connecting-ip directly
```

- Origin/Referer/HSTS/TLS explicit enforcement:
  - Not found in codebase grep for `Origin`, `Referer`, `hsts`, `tls` in relevant server paths.

## Risks
- Lack of explicit Origin/Host validation and anti-DNS-rebinding checks in reverse proxy listener. Default loopback mitigates in dev, but production needs explicit safeguards.
- Trusting `X-Forwarded-*` without configuring a trusted proxy chain can allow spoofed client IPs affecting rate limiting and policy decisions.
- No explicit TLS/HSTS handling in server components; assumed to be terminated at an upstream (but should be documented and enforced by config).

## Recommendations
- Add strict origin and Host validation for HTTP endpoints; reject requests with mismatched or missing Host/Origin unless configured.
- Add an option to require and validate `X-Forwarded-*` only when behind a trusted proxy; otherwise ignore these headers.
- Keep default bind to `127.0.0.1`; require explicit `--bind 0.0.0.0` and display a security warning.
- Document and provide configuration for TLS termination in production; consider HSTS when serving HTTPS directly.
- Ensure any future cookies set by the service include `Secure`, `HttpOnly`, and appropriate `SameSite`.

## Summary
- OAuth 2.1 basics present: PKCE, state, JWT validation with JWKS and audience/issuer checks.
- Transport security defaults are safe for dev; production hardening needed around origin/Host checks and proxy headers.
