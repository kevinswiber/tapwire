# Task A.0: Analyze Current Forward Proxy Implementation

## Objective
Understand the current single-session forward proxy implementation to identify exactly what needs to change for multi-session support.

## Key Questions
1. How does the current accept/connect flow work?
2. What assumes a single session?
3. How are transports lifecycle managed?
4. What state is shared vs per-session?
5. How does shutdown currently work?

## Process

### 1. Review ForwardProxy Structure
- [ ] Examine `src/proxy/forward.rs`
- [ ] Identify single-session assumptions in fields
- [ ] Review lifecycle methods (new, start, run_with_shutdown)
- [ ] Document current state management

### 2. Analyze Message Flow
- [ ] Trace client_to_server task
- [ ] Trace server_to_client task
- [ ] Identify blocking points
- [ ] Review error handling and recovery

### 3. Transport Interaction
- [ ] How accept() is called (once only)
- [ ] How connect() is called (once only)
- [ ] Transport lifecycle management
- [ ] Session ID assignment to transports

### 4. Integration Points
- [ ] SessionManager interaction
- [ ] InterceptorChain usage
- [ ] TapeRecorder integration
- [ ] RateLimiter application

### 5. CLI and API Layer
- [ ] Review `src/cli/forward.rs`
- [ ] Check `src/api.rs` forward methods
- [ ] Understand configuration flow
- [ ] Document user-facing interface

## Deliverables
Create `analysis/current-forward-proxy.md` with:
1. Architecture diagram of current implementation
2. List of single-session assumptions
3. Shared vs per-session state analysis
4. Required changes for multi-session

## Success Criteria
- [ ] Complete understanding of current implementation
- [ ] All single-session assumptions identified
- [ ] Clear picture of required refactoring
- [ ] Risk points documented