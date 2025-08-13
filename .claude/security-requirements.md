# Security Requirements

## Authentication and Authorization
- OAuth 2.1 compliance for auth gateway
- JWT validation with proper audience checking
- PKCE (Proof Key for Code Exchange) support
- **NEVER forward client tokens to upstream servers**
- Resource server metadata discovery (RFC 9728)

## Transport Security
- Localhost binding by default for development
- Origin validation for HTTP transport
- DNS rebinding protection
- TLS termination for production deployments

## Audit and Compliance
- Comprehensive event logging
- Session tracking and replay capabilities
- Rate limiting with configurable tiers
- Policy enforcement at multiple layers

## Critical Security Rules
- **NEVER pass through client tokens to upstream servers**
- **NEVER commit secrets** - use proper configuration management
- **ALWAYS validate JWT audience claims**
- **ALWAYS use HTTPS in production**
- **ALWAYS sanitize user inputs in interceptors**