# Shadowcat Documentation

Welcome to the Shadowcat documentation! Shadowcat is a high-performance Model Context Protocol (MCP) proxy written in Rust, providing transparent interception, recording, replay, and security enforcement for MCP communications.

## Documentation Overview

### [Architecture Guide](architecture.md)
Comprehensive overview of Shadowcat's system design, components, and data flow patterns. Covers the transport layer, proxy modes, session management, and performance characteristics.

### [Developer Guide](developer-guide.md)
Everything you need to start developing with Shadowcat. Includes setup instructions, development workflow, testing strategies, and patterns for adding new features.

### [API Reference](api-reference.md)
Complete reference for using Shadowcat as a library in your Rust applications. Covers all public APIs including builders, interceptors, session management, and authentication.

### [Deployment Guide](deployment.md)
Production deployment patterns and best practices. Includes Docker, Kubernetes, and systemd configurations, along with monitoring, security hardening, and troubleshooting.

## Quick Links

### Getting Started
```bash
# Clone the repository
git clone --recursive https://github.com/yourusername/tapwire
cd tapwire/shadowcat

# Build and test
cargo build --release
cargo test

# Run forward proxy
cargo run -- forward stdio -- npx @modelcontextprotocol/server-everything
```

### Common Use Cases

#### Development & Debugging
Use the forward proxy mode to intercept and debug MCP traffic:
```bash
shadowcat forward stdio -- your-mcp-server
```

#### Production Deployment
Deploy as a reverse proxy with authentication:
```bash
shadowcat reverse --bind 0.0.0.0:8080 --upstream http://mcp-server:3000 --auth-config auth.yaml
```

#### Recording Sessions
Capture MCP sessions for analysis:
```bash
shadowcat record --output session.tape -- forward stdio -- server-command
```

## Project Status

### Current Version: v0.2.0

**Stable Features:**
- ✅ Forward and reverse proxy modes
- ✅ stdio, HTTP, SSE, and Streamable HTTP transports
- ✅ Session management with SQLite
- ✅ Recording and replay capabilities
- ✅ OAuth 2.1 authentication
- ✅ Rate limiting and policy enforcement

**In Development:**
- 🔥 [Reverse Proxy Session Mapping](../plans/reverse-proxy-session-mapping/) - Dual session IDs
- 🔥 [Multi-Session Forward Proxy](../plans/multi-session-forward-proxy/) - Concurrent clients
- 🎯 [Better CLI Interface](../plans/better-cli-interface/) - Smart transport detection
- 🎯 [Redis Session Storage](../plans/redis-session-storage/) - Distributed sessions

See [plans/README.md](../plans/README.md) for the complete development roadmap.

## Architecture Highlights

### Performance
- **Latency**: < 3% p95 overhead
- **Memory**: ~60KB per session
- **Throughput**: 63,000+ sessions/sec
- **Startup**: < 50ms

### Security
- OAuth 2.1 with PKCE support
- JWT validation with JWKS
- Multi-tier rate limiting
- Policy-based access control
- Complete audit trail

### Extensibility
- Pluggable interceptor system
- Custom transport support
- Storage provider abstraction
- WebAssembly module integration (planned)

## Resources

### Internal Resources
- **Active Plans**: [plans/README.md](../plans/README.md)
- **Shadowcat Source**: [shadowcat/src/](../shadowcat/src/)
- **Integration Tests**: [shadowcat/tests/](../shadowcat/tests/)

### MCP References
Located in `~/src/modelcontextprotocol/`:
- **Specifications**: Official MCP protocol specs (all versions)
- **Inspector**: Web-based debugging tool
- **TypeScript SDK**: Reference implementation
- **Rust SDK**: Official Rust implementation (rmcp)
- **Example Servers**: Test servers for integration

## Contributing

We welcome contributions! Please ensure:

1. **Code Quality**: Run `cargo fmt` and `cargo clippy --all-targets -- -D warnings`
2. **Testing**: All tests pass with `cargo test`
3. **Documentation**: Update relevant docs
4. **Commits**: Use conventional commit format

See the [Developer Guide](developer-guide.md) for detailed contribution guidelines.

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/tapwire/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/tapwire/discussions)
- **Documentation**: This directory

## License

Shadowcat is part of the Tapwire project. See the LICENSE file for details.