# Task 002: OAuth 2.1 Flow Implementation with PKCE

**Phase:** 5 (Reverse Proxy & Authentication)  
**Week:** 1 (Core Infrastructure)  
**Day:** 2  
**Priority:** Critical  
**Estimated Time:** 6-8 hours

## Overview

Implement OAuth 2.1 authorization code flow with mandatory PKCE (Proof Key for Code Exchange) support. This task establishes the foundation authentication layer that enables secure client authentication without forwarding client tokens upstream (critical MCP requirement).

## Success Criteria

- [x] Research validated oauth2 crate as optimal OAuth 2.1 library
- [ ] OAuth 2.1 client configuration and initialization
- [ ] PKCE code challenge/verifier generation
- [ ] Authorization URL generation with PKCE parameters
- [ ] Authorization code exchange for access tokens
- [ ] Token refresh flow implementation
- [ ] Secure token storage (never forward client tokens)
- [ ] Authorization state management
- [ ] Integration with HTTP server from Task 001
- [ ] All tests passing (unit + integration)

## Technical Specifications

### OAuth 2.1 Client Implementation
```rust
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, 
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, AuthUrl, TokenUrl,
    basic::{BasicClient, BasicTokenType}, 
    reqwest::async_http_client,
};

pub struct OAuth2Client {
    client: BasicClient,
    pkce_verifier_store: Arc<RwLock<HashMap<String, PkceCodeVerifier>>>,
    state_store: Arc<RwLock<HashMap<String, AuthState>>>,
    config: OAuth2Config,
}

impl OAuth2Client {
    pub async fn new(config: OAuth2Config) -> Result<Self, OAuth2Error> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?);

        Ok(Self {
            client,
            pkce_verifier_store: Arc::new(RwLock::new(HashMap::new())),
            state_store: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }
}
```

### PKCE Implementation (Mandatory)
```rust
impl OAuth2Client {
    pub async fn generate_auth_url(&self, scopes: Vec<String>) -> Result<AuthUrlResponse, OAuth2Error> {
        // Generate PKCE code verifier and challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        
        // Generate CSRF token for state validation
        let (auth_url, csrf_token) = self.client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes.into_iter().map(Scope::new))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let state_key = csrf_token.secret().clone();
        
        // Store PKCE verifier and auth state
        {
            let mut verifier_store = self.pkce_verifier_store.write().await;
            verifier_store.insert(state_key.clone(), pkce_verifier);
        }
        
        {
            let mut state_store = self.state_store.write().await;
            state_store.insert(state_key.clone(), AuthState {
                created_at: Instant::now(),
                scopes: scopes,
                redirect_uri: self.config.redirect_url.clone(),
            });
        }

        Ok(AuthUrlResponse {
            auth_url: auth_url.to_string(),
            state: state_key,
        })
    }
}
```

### Token Exchange Implementation
```rust
impl OAuth2Client {
    pub async fn exchange_code(
        &self,
        code: String,
        state: String,
    ) -> Result<TokenResponse, OAuth2Error> {
        // Validate state and retrieve PKCE verifier
        let pkce_verifier = {
            let mut verifier_store = self.pkce_verifier_store.write().await;
            verifier_store.remove(&state)
                .ok_or(OAuth2Error::InvalidState)?
        };

        // Validate auth state
        let auth_state = {
            let mut state_store = self.state_store.write().await;
            state_store.remove(&state)
                .ok_or(OAuth2Error::InvalidState)?
        };

        // Check state expiration (5 minutes max)
        if auth_state.created_at.elapsed() > Duration::from_secs(300) {
            return Err(OAuth2Error::StateExpired);
        }

        // Exchange authorization code for access token
        let token_result = self.client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await?;

        Ok(TokenResponse {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            scope: token_result.scopes().map(|scopes| {
                scopes.iter().map(|s| s.to_string()).collect()
            }),
        })
    }
}
```

### Token Refresh Implementation
```rust
impl OAuth2Client {
    pub async fn refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<TokenResponse, OAuth2Error> {
        use oauth2::RefreshToken;
        
        let token_result = self.client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await?;

        Ok(TokenResponse {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            scope: token_result.scopes().map(|scopes| {
                scopes.iter().map(|s| s.to_string()).collect()
            }),
        })
    }
}
```

### Secure Token Storage
```rust
// CRITICAL: Never forward client tokens upstream
pub struct SecureTokenStore {
    // In-memory token storage (production: use secure database)
    tokens: Arc<RwLock<HashMap<SessionId, StoredToken>>>,
    encryption_key: Arc<[u8; 32]>, // For token encryption at rest
}

#[derive(Debug, Clone)]
struct StoredToken {
    access_token: String,     // Encrypted
    refresh_token: Option<String>, // Encrypted
    expires_at: Instant,
    scopes: Vec<String>,
    created_at: Instant,
}

impl SecureTokenStore {
    pub async fn store_token(
        &self,
        session_id: SessionId,
        token: TokenResponse,
    ) -> Result<(), TokenStoreError> {
        let expires_at = Instant::now() + token.expires_in.unwrap_or(Duration::from_secs(3600));
        
        let stored_token = StoredToken {
            access_token: self.encrypt_token(&token.access_token)?,
            refresh_token: token.refresh_token.map(|t| self.encrypt_token(&t)).transpose()?,
            expires_at,
            scopes: token.scope.unwrap_or_default(),
            created_at: Instant::now(),
        };

        let mut tokens = self.tokens.write().await;
        tokens.insert(session_id, stored_token);
        
        Ok(())
    }

    // CRITICAL: This method ensures we NEVER return the actual client token
    pub async fn get_auth_context(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<AuthContext>, TokenStoreError> {
        let tokens = self.tokens.read().await;
        
        if let Some(stored_token) = tokens.get(session_id) {
            if stored_token.expires_at > Instant::now() {
                return Ok(Some(AuthContext {
                    session_id: session_id.clone(),
                    scopes: stored_token.scopes.clone(),
                    expires_at: stored_token.expires_at,
                    // NOTE: We only provide metadata, never the actual token
                }));
            }
        }
        
        Ok(None)
    }
}
```

## Implementation Steps

### Step 1: Dependencies
```toml
# Add to shadowcat/Cargo.toml
[dependencies]
oauth2 = "4.4"
reqwest = { version = "0.12", features = ["json"] }
ring = "0.17" # For token encryption
hex = "0.4"   # For encoding
```

### Step 2: Core Module Structure
- `src/auth/oauth2_client.rs`: OAuth 2.1 client implementation
- `src/auth/token_store.rs`: Secure token storage
- `src/auth/types.rs`: OAuth-specific types and errors
- `src/config/oauth2.rs`: OAuth 2.1 configuration

### Step 3: OAuth 2.1 Flow Endpoints
```rust
// Add to reverse proxy router
pub fn add_oauth_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/auth/login", get(handle_oauth_login))
        .route("/auth/callback", get(handle_oauth_callback))
        .route("/auth/refresh", post(handle_token_refresh))
        .route("/auth/logout", post(handle_logout))
}

async fn handle_oauth_login(
    State(app_state): State<AppState>,
    Query(params): Query<LoginParams>,
) -> Result<Response, AuthError> {
    let auth_url = app_state.oauth_client
        .generate_auth_url(params.scopes.unwrap_or_default())
        .await?;
    
    // Return redirect response
    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", auth_url.auth_url)
        .body(Body::empty())?)
}
```

### Step 4: Integration with HTTP Server
- Extend AppState to include OAuth2Client
- Add OAuth routes to main router
- Implement middleware for token validation
- Add session-based authentication context

### Step 5: State Management and Cleanup
```rust
impl OAuth2Client {
    // Periodic cleanup of expired states and verifiers
    pub async fn cleanup_expired_states(&self) {
        let mut verifier_store = self.pkce_verifier_store.write().await;
        let mut state_store = self.state_store.write().await;
        
        let now = Instant::now();
        let expired_states: Vec<String> = state_store
            .iter()
            .filter(|(_, state)| now.duration_since(state.created_at) > Duration::from_secs(300))
            .map(|(key, _)| key.clone())
            .collect();
            
        for state_key in expired_states {
            state_store.remove(&state_key);
            verifier_store.remove(&state_key);
        }
    }
}
```

## Dependencies

### Blocked By
- Task 001: Axum HTTP Server Setup (foundation required)

### Blocks
- Task 003: JWT Validation with JWKS Client
- Task 004: AuthGateway Core Implementation

### Integrates With
- HTTP server router from Task 001
- Session management from existing Phase 4 infrastructure

## Testing Requirements

### Unit Tests
- [ ] PKCE code challenge/verifier generation
- [ ] Authorization URL generation with all parameters
- [ ] Token exchange flow with valid/invalid codes
- [ ] Token refresh flow
- [ ] State validation and cleanup
- [ ] Token encryption/decryption

### Integration Tests
- [ ] End-to-end OAuth 2.1 flow simulation
- [ ] PKCE flow compliance validation
- [ ] State tampering attack prevention
- [ ] Token storage security validation
- [ ] Concurrent authentication handling

### Security Tests
- [ ] PKCE enforcement (reject flows without PKCE)
- [ ] State parameter validation
- [ ] Token leakage prevention
- [ ] Timing attack resistance
- [ ] Cross-site request forgery protection

## Configuration Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub default_scopes: Vec<String>,
    pub token_encryption_key: String, // Base64 encoded
}

#[derive(Debug, Clone)]
pub struct AuthState {
    pub created_at: Instant,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<Duration>,
    pub refresh_token: Option<String>,
    pub scope: Option<Vec<String>>,
}
```

## Security Requirements

### PKCE Enforcement
- All authorization flows MUST use PKCE
- Code verifiers MUST be stored securely and cleaned up
- Code challenges MUST use SHA256

### Token Security
- Client tokens MUST NEVER be forwarded upstream
- Tokens MUST be encrypted at rest
- Token storage MUST be secure and regularly cleaned
- Refresh tokens MUST be rotated when possible

### State Management
- CSRF tokens MUST be cryptographically secure
- State parameters MUST be validated
- Expired states MUST be cleaned up automatically

## Performance Requirements

- Authorization URL generation: < 1ms
- Token exchange: < 100ms (network dependent)
- Token validation: < 1ms
- Memory usage: < 1KB per active auth session

## Risk Assessment

**Medium Risk**: OAuth 2.1 implementation complexity, security-critical functionality.

**Mitigation Strategies**:
- Use battle-tested oauth2 crate
- Comprehensive security testing
- Regular security audit of token handling
- Clear documentation of security requirements

## Completion Checklist

- [ ] OAuth 2.1 client properly configured
- [ ] PKCE flow fully implemented and tested
- [ ] Token exchange working correctly
- [ ] Token refresh flow functional
- [ ] Secure token storage implemented
- [ ] Authorization state management working
- [ ] HTTP endpoints integrated with server
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Security tests validate no token leakage
- [ ] Configuration schema documented
- [ ] Code review completed

## Files Modified/Created

### New Files
- `src/auth/oauth2_client.rs`: OAuth 2.1 client implementation
- `src/auth/token_store.rs`: Secure token storage
- `src/auth/types.rs`: OAuth types and errors
- `src/config/oauth2.rs`: OAuth configuration
- `tests/unit/oauth2_client_test.rs`: Unit tests
- `tests/integration/oauth2_flow_test.rs`: Integration tests

### Modified Files
- `src/auth/mod.rs`: Export new OAuth modules
- `src/proxy/reverse.rs`: Add OAuth routes to router
- `Cargo.toml`: Add oauth2 and related dependencies
- `src/config/mod.rs`: Include OAuth configuration

## Next Task
Upon completion, proceed to **Task 003: JWT Validation with JWKS Client** which builds upon the OAuth 2.1 foundation to provide token validation capabilities.