# Task 003: JWT Validation with JWKS Client Integration

**Phase:** 5 (Reverse Proxy & Authentication)  
**Week:** 1 (Core Infrastructure)  
**Day:** 3  
**Priority:** Critical  
**Estimated Time:** 6-8 hours

## Overview

Implement high-performance JWT token validation with automatic JWKS (JSON Web Key Set) key rotation and caching. This task provides the security validation layer that ensures only legitimate tokens are accepted while maintaining the performance target of < 1ms validation overhead.

## Success Criteria

- [x] Research validated jsonwebtoken + Ring cryptography for optimal performance
- [x] Research validated jwks-client for automatic key rotation
- [ ] JWT signature validation using RS256/ES256 algorithms
- [ ] JWKS client with automatic key rotation and caching
- [ ] Token claims validation (expiry, audience, issuer)
- [ ] Performance target: < 1ms average validation time
- [ ] Memory target: < 500 bytes per cached key
- [ ] Concurrent validation support (thread-safe)
- [ ] Integration with OAuth 2.1 flow from Task 002
- [ ] All tests passing (unit + integration + performance)

## Technical Specifications

### JWT Validation Core Implementation
```rust
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use jwks_client::{JwksClient, JsonWebKey};
use ring::signature;
use std::collections::HashMap;

pub struct TokenValidator {
    jwks_client: Arc<JwksClient>,
    validation_config: Validation,
    key_cache: Arc<RwLock<HashMap<String, CachedKey>>>,
    performance_metrics: Arc<ValidationMetrics>,
}

#[derive(Debug, Clone)]
struct CachedKey {
    decoding_key: DecodingKey,
    algorithm: Algorithm,
    cached_at: Instant,
    kid: String,
}

impl TokenValidator {
    pub async fn new(config: JwtValidationConfig) -> Result<Self, ValidationError> {
        let jwks_client = JwksClient::builder()
            .jwks_uri(config.jwks_uri)
            .cache_ttl(config.cache_ttl)
            .build()?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&config.allowed_audiences);
        validation.set_issuer(&config.allowed_issuers);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = config.clock_skew_tolerance;

        Ok(Self {
            jwks_client: Arc::new(jwks_client),
            validation_config: validation,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(ValidationMetrics::new()),
        })
    }
}
```

### High-Performance Token Validation
```rust
impl TokenValidator {
    pub async fn validate_token(&self, token: &str) -> Result<ValidatedClaims, ValidationError> {
        let start_time = Instant::now();
        
        // 1. Decode header to get key ID (kid)
        let header = decode_header(token)
            .map_err(|e| ValidationError::InvalidToken(e.to_string()))?;
            
        let kid = header.kid
            .ok_or(ValidationError::MissingKeyId)?;

        // 2. Get decoding key (with caching)
        let decoding_key = self.get_decoding_key(&kid).await?;

        // 3. Validate token with Ring-based cryptography
        let token_data = decode::<Claims>(
            token,
            &decoding_key.decoding_key,
            &self.validation_config,
        )
        .map_err(|e| ValidationError::TokenValidation(e.to_string()))?;

        // 4. Additional custom validations
        self.validate_custom_claims(&token_data.claims)?;

        // 5. Record performance metrics
        let validation_time = start_time.elapsed();
        self.performance_metrics.record_validation(validation_time);

        Ok(ValidatedClaims {
            subject: token_data.claims.sub,
            audience: token_data.claims.aud,
            issuer: token_data.claims.iss,
            expires_at: token_data.claims.exp,
            issued_at: token_data.claims.iat,
            scopes: token_data.claims.scope
                .map(|s| s.split_whitespace().map(String::from).collect())
                .unwrap_or_default(),
            custom_claims: token_data.claims.custom,
        })
    }

    async fn get_decoding_key(&self, kid: &str) -> Result<CachedKey, ValidationError> {
        // Check cache first
        {
            let cache = self.key_cache.read().await;
            if let Some(cached_key) = cache.get(kid) {
                // Check if key is still fresh (5 minutes cache)
                if cached_key.cached_at.elapsed() < Duration::from_secs(300) {
                    return Ok(cached_key.clone());
                }
            }
        }

        // Fetch from JWKS endpoint
        let jwk = self.jwks_client.get_opt(kid).await?
            .ok_or(ValidationError::KeyNotFound(kid.to_string()))?;

        let decoding_key = self.jwk_to_decoding_key(&jwk)?;
        let algorithm = self.determine_algorithm(&jwk)?;

        let cached_key = CachedKey {
            decoding_key,
            algorithm,
            cached_at: Instant::now(),
            kid: kid.to_string(),
        };

        // Update cache
        {
            let mut cache = self.key_cache.write().await;
            cache.insert(kid.to_string(), cached_key.clone());
        }

        Ok(cached_key)
    }
}
```

### JWKS Client Configuration
```rust
impl TokenValidator {
    fn jwk_to_decoding_key(&self, jwk: &JsonWebKey) -> Result<DecodingKey, ValidationError> {
        match jwk.algorithm.as_deref() {
            Some("RS256") | Some("RS384") | Some("RS512") => {
                // RSA public key
                if let (Some(n), Some(e)) = (&jwk.n, &jwk.e) {
                    let decoding_key = DecodingKey::from_rsa_components(n, e)
                        .map_err(|e| ValidationError::KeyConversion(e.to_string()))?;
                    Ok(decoding_key)
                } else {
                    Err(ValidationError::InvalidRsaKey)
                }
            }
            Some("ES256") | Some("ES384") | Some("ES512") => {
                // ECDSA public key
                if let (Some(x), Some(y)) = (&jwk.x, &jwk.y) {
                    let decoding_key = DecodingKey::from_ec_components(x, y)
                        .map_err(|e| ValidationError::KeyConversion(e.to_string()))?;
                    Ok(decoding_key)
                } else {
                    Err(ValidationError::InvalidEcKey)
                }
            }
            _ => Err(ValidationError::UnsupportedAlgorithm(
                jwk.algorithm.clone().unwrap_or_default()
            ))
        }
    }

    fn determine_algorithm(&self, jwk: &JsonWebKey) -> Result<Algorithm, ValidationError> {
        match jwk.algorithm.as_deref() {
            Some("RS256") => Ok(Algorithm::RS256),
            Some("RS384") => Ok(Algorithm::RS384),
            Some("RS512") => Ok(Algorithm::RS512),
            Some("ES256") => Ok(Algorithm::ES256),
            Some("ES384") => Ok(Algorithm::ES384),
            Some("ES512") => Ok(Algorithm::ES512),
            _ => Err(ValidationError::UnsupportedAlgorithm(
                jwk.algorithm.clone().unwrap_or_default()
            ))
        }
    }
}
```

### Claims Structure and Validation
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // Subject
    pub aud: Vec<String>,      // Audience
    pub iss: String,           // Issuer  
    pub exp: u64,              // Expiry time
    pub iat: u64,              // Issued at
    pub nbf: Option<u64>,      // Not before
    pub scope: Option<String>, // OAuth scopes
    
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>, // Custom claims
}

#[derive(Debug, Clone)]
pub struct ValidatedClaims {
    pub subject: String,
    pub audience: Vec<String>,
    pub issuer: String,
    pub expires_at: u64,
    pub issued_at: u64,
    pub scopes: Vec<String>,
    pub custom_claims: HashMap<String, serde_json::Value>,
}

impl TokenValidator {
    fn validate_custom_claims(&self, claims: &Claims) -> Result<(), ValidationError> {
        // Additional business logic validations
        
        // Ensure token is not expired (with clock skew tolerance)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        if claims.exp <= now {
            return Err(ValidationError::TokenExpired);
        }

        // Validate not-before claim if present
        if let Some(nbf) = claims.nbf {
            if nbf > now {
                return Err(ValidationError::TokenNotYetValid);
            }
        }

        // Validate custom business rules here
        // Example: Ensure specific scopes are present
        if let Some(scope_str) = &claims.scope {
            let scopes: Vec<&str> = scope_str.split_whitespace().collect();
            if !scopes.contains(&"mcp:access") {
                return Err(ValidationError::InsufficientScope);
            }
        }

        Ok(())
    }
}
```

### Performance Monitoring
```rust
pub struct ValidationMetrics {
    validation_times: Arc<RwLock<Vec<Duration>>>,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    validation_errors: AtomicU64,
}

impl ValidationMetrics {
    pub fn record_validation(&self, duration: Duration) {
        let mut times = self.validation_times.write().unwrap();
        times.push(duration);
        
        // Keep only last 1000 measurements for rolling average
        if times.len() > 1000 {
            times.drain(0..100);
        }
    }

    pub fn get_average_validation_time(&self) -> Duration {
        let times = self.validation_times.read().unwrap();
        if times.is_empty() {
            return Duration::from_nanos(0);
        }
        
        let total: Duration = times.iter().sum();
        total / times.len() as u32
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        
        if hits + misses == 0 {
            return 0.0;
        }
        
        hits as f64 / (hits + misses) as f64
    }
}
```

## Implementation Steps

### Step 1: Dependencies
```toml
# Add to shadowcat/Cargo.toml
[dependencies]
jsonwebtoken = "9.3"
jwks-client = "0.4"
ring = "0.17"
base64 = "0.22"
```

### Step 2: Core Module Structure
- `src/auth/token_validator.rs`: JWT validation core
- `src/auth/jwks_client.rs`: JWKS client wrapper
- `src/auth/claims.rs`: Claims structures and validation
- `src/auth/metrics.rs`: Performance metrics
- `src/config/jwt.rs`: JWT validation configuration

### Step 3: Integration with AuthGateway
```rust
// Integration point for Task 004
pub struct AuthGateway {
    oauth_client: Arc<OAuth2Client>,
    token_validator: Arc<TokenValidator>,
    token_store: Arc<SecureTokenStore>,
}

impl AuthGateway {
    pub async fn authenticate_request(
        &self,
        bearer_token: &str,
        session_id: &SessionId,
    ) -> Result<AuthContext, AuthError> {
        // Validate JWT token
        let validated_claims = self.token_validator
            .validate_token(bearer_token)
            .await?;

        // Create authentication context (no token forwarding)
        Ok(AuthContext {
            session_id: session_id.clone(),
            subject: validated_claims.subject,
            scopes: validated_claims.scopes,
            expires_at: UNIX_EPOCH + Duration::from_secs(validated_claims.expires_at),
            custom_claims: validated_claims.custom_claims,
        })
    }
}
```

### Step 4: Middleware Integration
```rust
// JWT validation middleware for Axum
pub async fn jwt_auth_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    // Extract Bearer token from Authorization header
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingAuthHeader)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidAuthFormat);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix
    
    // Get token validator from app state
    let token_validator = request.extensions()
        .get::<Arc<TokenValidator>>()
        .ok_or(AuthError::InternalError)?;

    // Validate token
    let validated_claims = token_validator
        .validate_token(token)
        .await?;

    // Add auth context to request extensions
    request.extensions_mut().insert(AuthContext::from(validated_claims));

    Ok(next.run(request).await)
}
```

### Step 5: Performance Testing and Optimization
- Implement validation benchmarks
- Test concurrent validation scenarios
- Optimize key caching strategy
- Monitor memory usage patterns

## Dependencies

### Blocked By
- Task 002: OAuth 2.1 Flow Implementation (for integration testing)

### Blocks
- Task 004: AuthGateway Core Implementation
- Task 006: Extended RuleBasedInterceptor with HTTP Conditions

### Integrates With
- OAuth 2.1 client from Task 002
- HTTP server middleware from Task 001

## Testing Requirements

### Unit Tests
- [ ] JWT signature validation (RS256, ES256)
- [ ] Claims validation (expiry, audience, issuer)
- [ ] JWKS key conversion accuracy
- [ ] Key caching behavior
- [ ] Performance metrics accuracy
- [ ] Error handling for malformed tokens

### Integration Tests
- [ ] End-to-end token validation flow
- [ ] JWKS key rotation handling
- [ ] Concurrent validation performance
- [ ] Cache hit/miss scenarios
- [ ] Network failure recovery

### Performance Tests
- [ ] Validation time benchmarks (target: < 1ms average)
- [ ] Memory usage per cached key (target: < 500 bytes)
- [ ] Concurrent validation throughput
- [ ] Cache efficiency measurements
- [ ] JWKS fetch performance

### Security Tests
- [ ] Token tampering detection
- [ ] Expired token rejection
- [ ] Invalid signature rejection
- [ ] Key ID (kid) manipulation attempts
- [ ] Algorithm confusion attacks

## Configuration Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtValidationConfig {
    pub jwks_uri: String,
    pub allowed_audiences: Vec<String>,
    pub allowed_issuers: Vec<String>,
    pub cache_ttl: Duration,
    pub clock_skew_tolerance: Duration,
    pub supported_algorithms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksConfig {
    pub cache_size: usize,
    pub refresh_interval: Duration,
    pub request_timeout: Duration,
    pub retry_attempts: u32,
}
```

## Performance Requirements

### Validation Performance
- **Average validation time:** < 1ms (Ring cryptography target: ~45µs)
- **p95 validation time:** < 2ms
- **p99 validation time:** < 5ms
- **Concurrent validations:** 1000+ simultaneous

### Memory Usage
- **Per cached key:** < 500 bytes
- **Cache size:** Configurable (default: 100 keys)
- **Memory leak prevention:** Automatic cache cleanup

### Network Performance
- **JWKS fetch time:** < 100ms
- **Cache hit rate:** > 95% under normal load
- **Retry strategy:** Exponential backoff with jitter

## Risk Assessment

**Low Risk**: Using mature jsonwebtoken library with Ring cryptography, well-established patterns.

**Mitigation Strategies**:
- Comprehensive unit test coverage for all algorithms
- Performance benchmarking during development
- Security testing against common JWT attacks
- Production monitoring of validation metrics

## Completion Checklist

- [ ] JWT validation working with RS256/ES256 algorithms
- [ ] JWKS client fetching and caching keys correctly
- [ ] Performance targets met (< 1ms average validation)
- [ ] Memory usage within limits (< 500 bytes per key)
- [ ] Claims validation working correctly
- [ ] Error handling for all failure scenarios
- [ ] Integration with middleware complete
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Performance benchmarks meeting targets
- [ ] Security tests validate attack resistance
- [ ] Configuration schema documented
- [ ] Code review completed

## Files Modified/Created

### New Files
- `src/auth/token_validator.rs`: Core JWT validation logic
- `src/auth/jwks_client.rs`: JWKS client wrapper
- `src/auth/claims.rs`: Claims structures and validation
- `src/auth/metrics.rs`: Performance metrics
- `src/config/jwt.rs`: JWT configuration
- `tests/unit/token_validator_test.rs`: Unit tests
- `tests/integration/jwt_validation_test.rs`: Integration tests
- `benches/jwt_validation_bench.rs`: Performance benchmarks

### Modified Files
- `src/auth/mod.rs`: Export JWT validation modules
- `src/proxy/reverse.rs`: Add JWT middleware to router
- `Cargo.toml`: Add JWT and JWKS dependencies
- `src/config/mod.rs`: Include JWT configuration

## Next Task
Upon completion, proceed to **Task 004: AuthGateway Core Implementation** which integrates OAuth 2.1 and JWT validation into a unified authentication gateway.