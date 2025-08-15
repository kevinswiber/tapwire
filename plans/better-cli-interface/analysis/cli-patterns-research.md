# CLI UX Patterns Research

## Overview

Analysis of successful CLI tools to extract patterns that make them intuitive and delightful to use. Focus on tools that handle similar concepts: networking, proxying, process management, and developer tools.

## Tools Analyzed

### 1. ngrok - Tunneling Made Simple

**Why it's great:** Makes complex networking simple with smart defaults

**Key Patterns:**
```bash
# Simplest case is magical
ngrok http 3000              # Just works - exposes localhost:3000

# Smart detection
ngrok 3000                   # Assumes HTTP
ngrok tcp 22                 # Explicit protocol when needed
ngrok tls 443                # Different protocol

# File-based config
ngrok start myapp           # Reads from config file
```

**Lessons:**
- ✅ First argument determines mode
- ✅ Smart defaults (HTTP for web ports)
- ✅ Minimal typing for common case
- ✅ Protocol prefix for clarity when needed

### 2. Docker - Contextual Commands

**Why it's great:** Verbs make intent crystal clear

**Key Patterns:**
```bash
# Action-oriented
docker run nginx             # Intent is clear
docker build .               # Context from current directory
docker pull ubuntu           # Explicit action

# Smart detection of context
docker run -it ubuntu bash   # Detects interactive terminal
docker build -t myapp .      # Detects Dockerfile
```

**Lessons:**
- ✅ Verbs clarify intent
- ✅ Positional arguments for main target
- ✅ Flags modify behavior, not define it
- ✅ Context awareness (current directory, terminal type)

### 3. Git - Subcommands with Intelligence

**Why it's great:** Consistent patterns with helpful shortcuts

**Key Patterns:**
```bash
# Common operations are short
git push                     # Uses current branch
git pull                     # Smart defaults
git checkout main            # Branch detection

# Auto-detection
git clone <url>              # Detects protocol from URL
git remote add origin <url>  # Understands git@ vs https://
```

**Lessons:**
- ✅ Defaults to current context
- ✅ URL parsing for protocol detection
- ✅ Common operations have shortest syntax
- ✅ Power features require explicit commands

### 4. PostgreSQL (psql) - Connection String Magic

**Why it's great:** Multiple ways to connect, all intuitive

**Key Patterns:**
```bash
# URL format
psql postgresql://user:pass@localhost/db

# Positional for local
psql mydb

# Explicit flags when needed
psql -h localhost -U user -d database

# Environment variables as fallback
PGHOST=localhost psql
```

**Lessons:**
- ✅ URL parsing for complete connection info
- ✅ Positional argument for simple local case
- ✅ Environment variables for defaults
- ✅ Multiple input methods for same result

### 5. curl - URL-First Design

**Why it's great:** The target is always clear

**Key Patterns:**
```bash
# URL is the star
curl https://api.example.com

# Method detection from flags
curl -X POST https://api.example.com
curl -d @file.json https://api.example.com  # POST implied

# Output handling
curl -o file.txt https://example.com
```

**Lessons:**
- ✅ Main argument is always the target
- ✅ Flags modify behavior predictably
- ✅ Smart inference from flag combinations
- ✅ Progressive disclosure of complexity

### 6. SSH - Smart Connection Parsing

**Why it's great:** Flexible connection specification

**Key Patterns:**
```bash
# Simple host
ssh server

# User@host pattern
ssh user@server

# Port specification
ssh server -p 2222
ssh ssh://user@server:2222  # URL style

# Config-based
ssh myalias                  # From ~/.ssh/config
```

**Lessons:**
- ✅ Multiple syntax styles supported
- ✅ Common patterns (user@host) understood
- ✅ Config file for aliases
- ✅ URL format as alternative

### 7. npm/yarn - Context-Aware Commands

**Why it's great:** Understands project context

**Key Patterns:**
```bash
# Detects package.json
npm install                  # In project directory
npm install express          # Specific package

# Scripts are first-class
npm run dev                  # From package.json
npm test                     # Common scripts are shortcuts

# Global vs local context
npm install -g shadwocat     # Explicit global
```

**Lessons:**
- ✅ File detection for context
- ✅ Common operations get shortcuts
- ✅ Explicit flags for scope changes
- ✅ Configuration file drives behavior

## Extracted Patterns

### 1. Smart Detection Patterns

**Pattern**: First argument type determines behavior
```bash
tool <command>               # Explicit command
tool <file>                  # File extension detection
tool <url>                   # Protocol from URL
tool <host:port>             # Network endpoint
tool <number>                # Port number
```

### 2. Progressive Disclosure

**Pattern**: Simple cases are simple, complex cases are possible
```bash
# Level 1: Magic
tool myserver

# Level 2: Some control
tool myserver --verbose

# Level 3: Full control
tool forward stdio --rate-limit -- myserver --args
```

### 3. Context Awareness

**Pattern**: Use environment to reduce typing
- Current directory (Dockerfile, package.json, .git)
- Environment variables for defaults
- Config files for common settings
- Terminal capabilities (TTY, color support)

### 4. Helpful Error Messages

**Pattern**: Errors suggest solutions
```
Error: Cannot connect to 'myserver'

Did you mean:
  shadowcat forward stdio -- myserver
  shadowcat myserver.local
  shadowcat http://myserver:8080
```

### 5. Verb-Noun Clarity

**Pattern**: Action-object relationship is clear
```bash
# Clear intent
docker run image
git push origin
npm install package

# For Shadowcat
shadowcat forward myserver
shadowcat gateway :8080
shadowcat record session
```

### 6. URL/URI Parsing

**Pattern**: Understand common formats
```bash
# Parse these intelligently
http://localhost:3000        # HTTP proxy target
user@server                   # SSH-style notation
:8080                        # Port binding
./myfile.tape                # File path
mycommand                    # Command name
```

## Recommendations for Shadowcat

### Priority 1: Smart Detection

Implement first-argument detection:
```bash
shadowcat my-mcp-server          # → forward stdio
shadowcat http://localhost:3000  # → forward http or gateway
shadowcat :8080                  # → gateway binding
shadowcat session.tape           # → replay
shadowcat record my-session      # → record (verb detection)
```

### Priority 2: Clear Verbs

Keep verbs for explicit control:
```bash
shadowcat forward my-server      # Explicit forward proxy
shadowcat gateway :8080          # Explicit gateway mode
shadowcat record my-session      # Explicit recording
shadowcat replay session.tape    # Explicit replay
```

### Priority 3: Helpful Errors

```
Error: 'my-server' is not a recognized command

Based on your input, did you mean:
  shadowcat forward stdio -- my-server    # Run my-server as MCP server
  shadowcat gateway http://my-server      # Use my-server as upstream
  
Run 'shadowcat --help' for more options
```

### Priority 4: Progressive Complexity

```bash
# Simple (90% of users)
shadowcat my-server

# Intermediate (9% of users)
shadowcat forward my-server --rate-limit

# Advanced (1% of users)
shadowcat forward stdio --rate-limit-rpm 100 --session-timeout 600 -- my-server --debug
```

## Anti-Patterns to Avoid

### 1. Required Flags for Common Cases
❌ `shadowcat forward --transport stdio -- my-server`
✅ `shadowcat my-server`

### 2. Unclear Terminology
❌ `shadowcat reverse --upstream target`
✅ `shadowcat gateway target`

### 3. Inconsistent Argument Position
❌ Sometimes first arg is command, sometimes target
✅ First arg always identifies intent

### 4. Hidden Behavior
❌ Magic that's not discoverable
✅ `--verbose` or `--dry-run` to see what will happen

### 5. No Escape Hatch
❌ Magic you can't override
✅ Explicit commands always available

## Conclusion

The best CLI tools share common patterns:
1. **Smart detection** from first argument
2. **Progressive disclosure** of complexity
3. **Clear verb-noun** relationships
4. **Helpful error messages** with suggestions
5. **Multiple input methods** for same result

For Shadowcat, the key improvements are:
1. Replace "reverse" with "gateway" (terminology)
2. Add smart detection for common patterns (UX)
3. Keep explicit commands available (control)
4. Improve error messages with suggestions (discoverability)