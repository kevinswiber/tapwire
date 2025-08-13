# Tapwire

**DevTools for Model Context Protocol (MCP)** - The complete platform for MCP inspection, observability, and security.

## Overview

Tapwire is a comprehensive developer platform that brings transparency and control to MCP communications. As AI agents and IDEs increasingly adopt MCP for tool integration, developers need professional-grade tools to inspect, debug, secure, and optimize these interactions.

## Platform Components

### 🐱 [Shadowcat](./shadowcat/) (Open Source)
Our core MCP proxy engine - a high-performance Rust implementation providing:
- Forward & reverse proxy capabilities
- Session recording and replay
- Request interception and modification
- Transport protocol bridging (stdio ↔ HTTP)

### 🔍 Inspector (Coming Soon)
Real-time MCP traffic analyzer with:
- Live session viewer
- Message timeline visualization
- Performance metrics dashboard
- Protocol compliance validation

### 🛡️ Gateway (Coming Soon)
Enterprise security layer featuring:
- OAuth 2.1 authentication enforcement
- Role-based access control
- Rate limiting and quotas
- Audit logging

### 📊 Analytics (Coming Soon)
Comprehensive observability suite:
- Usage analytics and trends
- Performance monitoring
- Error tracking
- Custom dashboards

## Why Tapwire?

### For Developers
- **Debug Faster**: See exactly what's happening between your IDE and MCP servers
- **Test Reliably**: Record and replay sessions for consistent testing
- **Build Safely**: Intercept and modify requests during development

### For Teams
- **Collaborate Better**: Share recorded sessions for debugging
- **Ship Confidently**: Validate MCP compliance before deployment
- **Monitor Continuously**: Track performance and errors in production

### For Enterprises
- **Secure by Default**: Never expose tokens to upstream servers
- **Comply Easily**: Full audit trails and access controls
- **Scale Reliably**: Handle thousands of concurrent sessions

## Quick Start

```bash
# Install Shadowcat (open source proxy)
cd shadowcat
cargo install --path .

# Start proxying an MCP server
shadowcat forward stdio -- npx @modelcontextprotocol/server-everything
```

## Use Cases

- **MCP Server Development**: Debug your server implementation with full visibility
- **IDE Plugin Development**: Test your MCP client integration safely
- **Security Auditing**: Inspect and validate MCP traffic for compliance
- **Performance Optimization**: Identify bottlenecks and optimize server responses
- **Integration Testing**: Record production sessions and replay in CI/CD

## Architecture

Tapwire is built on a modular architecture:

```
┌─────────────────────────────────────┐
│          Tapwire Platform           │
├─────────────────────────────────────┤
│  Inspector │ Gateway │  Analytics   │
├─────────────────────────────────────┤
│        Shadowcat (OSS Core)         │
├─────────────────────────────────────┤
│   MCP Clients │ MCP Servers         │
└─────────────────────────────────────┘
```

## Roadmap

### Phase 1: Foundation (Current)
- [x] Shadowcat core proxy engine
- [ ] Basic recording and replay
- [ ] CLI interface

### Phase 2: Developer Experience
- [ ] Web-based Inspector UI
- [ ] VS Code extension
- [ ] Session sharing

### Phase 3: Enterprise
- [ ] Gateway security features
- [ ] Analytics dashboard
- [ ] Multi-tenant support

## Pricing

| Tier | Features | Price |
|------|----------|-------|
| **Open Source** | Shadowcat proxy, CLI tools, local recording | Free |
| **Pro** | Inspector UI, cloud replay, team sharing | Coming Soon |
| **Enterprise** | Gateway, analytics, SSO, SLA | Contact Us |

## Community

- **Discord**: [Coming Soon]
- **Documentation**: [Coming Soon]
- **Blog**: [Coming Soon]

## Development Tools

### Claude Code Commands

This repository includes custom Claude Code slash commands to streamline development workflow. These commands help manage development plans, track progress, and maintain consistency across work sessions.

#### Available Commands

| Command | Description | Usage |
|---------|-------------|-------|
| `/plan` | Load a development plan for focused work | `/plan transport-refactor` |
| `/plan-list` | List all available development plans | `/plan-list` |
| `/plan-status` | Show detailed status of all active plans | `/plan-status` |
| `/plan-complete` | Mark a plan phase/task as complete | `/plan-complete transport-refactor A.1` |

#### Features

- **Automatic Context Loading**: Commands automatically load relevant files (next-session-prompt, tracker, tasks)
- **Progress Tracking**: Integration with TodoWrite tool for session task management
- **Status Reporting**: Real-time analysis of plan progress with task counts and completion metrics
- **Session Continuity**: Helps maintain context between Claude sessions with structured handoffs

#### Quick Start

1. Start a new work session: `/plan <plan-name>`
2. Check available work: `/plan-list`
3. Review progress: `/plan-status`
4. Complete and transition: `/plan-complete <plan-name> <phase>`

#### Plan Structure

Development plans follow a standardized structure:
```
plans/<plan-name>/
├── next-session-prompt.md  # Current objectives and setup
├── *-tracker.md            # Overall progress tracking
└── tasks/                  # Individual task specifications
    └── *.md
```

See [plans/template/](./plans/template/) for creating new plans.

## Contributing

We welcome contributions to Shadowcat! See the [Shadowcat README](./shadowcat/README.md) for details.

## License

- Shadowcat (core proxy): MIT License
- Tapwire Platform: Proprietary

---

Built with ❤️ for the MCP developer community