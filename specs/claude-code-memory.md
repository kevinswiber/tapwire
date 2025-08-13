# Claude Code Memory (CLAUDE.md) Best Practices

## Overview
CLAUDE.md files provide persistent memory for Claude Code, helping maintain context and project-specific knowledge across sessions.

## Best Practices

### 1. Be Specific
- Use precise, concrete instructions rather than vague guidelines
- ✅ Good: "Use 2-space indentation"
- ❌ Bad: "Format code properly"

### 2. Organizational Structure
- Format memories as bullet points for clarity
- Group related memories under descriptive markdown headings
- Use clear sections with logical hierarchy
- Example structure:
  ```markdown
  ## Build Commands
  - Run tests: `cargo test`
  - Format code: `cargo fmt`
  
  ## Code Style
  - Use 2-space indentation
  - Prefer early returns over nested if statements
  ```

### 3. What to Include

#### Essential Commands
- Build commands specific to your project
- Test commands and testing patterns
- Linting and formatting commands
- Development server commands
- Deployment procedures

#### Code Style & Conventions
- Language-specific style preferences
- Naming conventions for variables, functions, files
- Import/module organization patterns
- Comment style guidelines

#### Architecture Patterns
- Project structure overview
- Key design patterns used
- Module communication flow
- Important architectural decisions

#### Project-Specific Guidelines
- Security requirements
- Performance targets
- Error handling patterns
- Logging conventions

### 4. What NOT to Include
- Generic language documentation (Claude already knows this)
- Standard library functions
- Common programming concepts
- Overly verbose explanations

### 5. Formatting Guidelines
- Use markdown for clear hierarchy
- Keep instructions concise and actionable
- Use code blocks for commands and examples
- Organize with clear headings and subheadings

### 6. Memory Management Tips
- Review and update memories periodically
- Remove outdated information
- Ensure context remains current with project evolution
- Keep file size reasonable (avoid information overload)

### 7. Import Capabilities
CLAUDE.md supports importing other files for modular memory management:

```markdown
# Import another memory file
@path/to/additional-memory.md
```

- Supports both relative and absolute paths
- Maximum recursive import depth: 5 hops
- Useful for shared team conventions or splitting large configurations

### 8. Memory Scope Hierarchy
Memories can exist at different levels:
1. **Enterprise**: Organization-wide conventions
2. **Project**: Project-specific guidelines (checked into repo)
3. **User**: Personal preferences (~/.claude/CLAUDE.md)

Claude Code reads all applicable levels, with more specific scopes taking precedence.

### 9. Interactive Editing
- Use the `/memory` command during a session to easily edit memory files
- Changes take effect immediately in the current session
- Helps iterate on memory configuration quickly

## Example Optimized CLAUDE.md Structure

```markdown
# Project Name

## Essential Commands
- Build: `npm run build`
- Test: `npm test -- --coverage`
- Lint: `npm run lint:fix`
- Dev server: `npm run dev -- --port 3000`

## Code Style
- TypeScript strict mode enabled
- 2-space indentation
- Prefer const over let
- Use early returns to reduce nesting

## Architecture
- Feature-based folder structure
- State management: Redux Toolkit
- API calls: RTK Query
- Component pattern: Composition over inheritance

## Testing Guidelines
- Unit tests for utilities: `*.test.ts`
- Component tests: React Testing Library
- E2E tests: Playwright
- Minimum coverage: 80%

## Performance Targets
- Initial load: < 3s
- API response: < 200ms p95
- Bundle size: < 500KB gzipped

## Git Workflow
- Branch naming: `feature/ticket-description`
- Commit format: conventional commits
- PR requires 2 approvals
- Squash merge to main
```

## Optimization Checklist
- [ ] Remove redundant or obvious information
- [ ] Consolidate related commands under clear headings
- [ ] Use bullet points instead of paragraphs
- [ ] Include specific version numbers where relevant
- [ ] Add project-specific quirks and gotchas
- [ ] Reference key documentation files
- [ ] Set up imports for shared team conventions
- [ ] Keep total size under 1000 lines for optimal performance