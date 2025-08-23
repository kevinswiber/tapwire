# XTask Lint Enhancements - Analysis

## Overview
This directory contains analysis documents for the xtask lint enhancement project.

## Documents

### [current-state-analysis.md](current-state-analysis.md)
**Status**: ✅ Complete

Comprehensive analysis of shadowcat's current lint gaps:
- Quantified 1082+ unwrap() calls in production code
- Identified 170+ debug output instances
- Mapped high-risk areas by module
- Incorporated GPT-5's pragmatic feedback

Key findings:
- Critical production stability risk from panics
- No async hygiene enforcement
- Missing complexity limits
- Good foundation with existing module boundary checks

### violation-assessment.md
**Status**: ⬜ Pending (after implementation)

Will contain:
- Categorized list of all violations found
- Severity assessment (critical/high/medium/low)
- Fix priority and effort estimates
- Migration timeline

### clippy-overlap-analysis.md
**Status**: ⬜ Future

Will analyze:
- Which custom lints duplicate Clippy
- Opportunities to delegate to Clippy
- Custom lints that must remain

### escape-hatch-usage.md
**Status**: ⬜ Future

Will track:
- Where escape hatches are used
- Reasons provided
- Patterns that might indicate missing features

## Key Metrics

### Current State
| Metric | Count | Risk |
|--------|-------|------|
| unwrap() calls | 1082+ | CRITICAL |
| expect() calls | ~400 | HIGH |
| panic!() calls | ~20 | HIGH |
| println!() calls | ~100 | MEDIUM |
| No escape mechanism | All | HIGH |

### Target State
| Metric | Target | Timeline |
|--------|--------|----------|
| Unwrap in production | 0* | 4 weeks |
| Debug output | 0* | 2 weeks |
| Async blocking | 0 | 6 weeks |
| Function >150 LOC | 0 | 8 weeks |

*With documented escape hatches

## Recommendations Priority

1. **Immediate** (Week 1):
   - Implement no_unwrap checker
   - Add escape hatch system
   - Start fixing critical unwraps

2. **Short Term** (Weeks 2-4):
   - Clippy integration
   - Debug output enforcement
   - CI pipeline integration

3. **Medium Term** (Months 2-3):
   - Async hygiene
   - Complexity limits
   - Architecture boundaries

## Tools Assessment

### Keep Using
- `syn` - AST parsing (already in use)
- `walkdir` - File traversal (already in use)
- Clippy - Leverage existing lints

### Add
- `cargo-deny` - Security advisories
- `gitleaks` - Secret scanning
- `cargo-nextest` - Better test runner

### Don't Build
- Size checking (use Clippy)
- Secret regex (use real scanners)
- File size limits (use function metrics)

## Success Metrics

Track these after implementation:
- Violations found vs fixed per week
- Escape hatch usage rate
- Developer friction reports
- Time to fix violations
- CI failure rate