# AgentOrchestrator Migration Plan

## Overview

We have created 4 focused service modules to replace the god object pattern in AgentOrchestrator, but they are not yet integrated. This document outlines the safe migration path.

## Current State

### Created Services (Not Yet Integrated)

- `task_queue.rs` - Task queue management with size limits
- `agent_registry.rs` - Agent registration and lookup with factory pattern  
- `result_store.rs` - Task result storage with repository pattern
- `status_manager.rs` - Agent and task status tracking

### Existing God Object

- `orchestrator/mod.rs` - Still contains all responsibilities inline
- Working and in production
- 600+ lines of intertwined logic

## Migration Strategy

### Phase 1: Incremental Integration (Low Risk)

Start by integrating one service at a time while keeping the old code as fallback.

1. **TaskQueue Integration**

   ```rust
   // Add to orchestrator
   task_queue_service: TaskQueue,
   
   // Gradually replace self.task_queue with self.task_queue_service
   ```

2. **StatusManager Integration**  
   - Replace status tracking logic with StatusManager calls
   - Keep old status fields temporarily for rollback

3. **ResultStore Integration**
   - Migrate result storage to ResultStore
   - Keep old HashMaps temporarily

4. **AgentRegistry Integration**
   - Most complex - involves changing agent instantiation
   - Replace hardcoded agent creation with factory

### Phase 2: Cleanup (After Validation)

1. Remove old fields from orchestrator struct
2. Delete unused methods
3. Update tests to use new services

### Phase 3: Full Service Orientation

1. Make orchestrator purely a coordinator
2. Move remaining logic to appropriate services
3. Consider dependency injection for services

## Risk Mitigation

### Testing Strategy

1. **Unit Tests**: Test each service in isolation
2. **Integration Tests**: Test orchestrator with new services
3. **Regression Tests**: Ensure no functionality is lost
4. **Performance Tests**: Verify no performance degradation

### Rollback Plan

- Keep old implementation commented but available
- Use feature flags if needed for gradual rollout
- Maintain backwards compatibility during transition

## Implementation Checklist

- [ ] Create comprehensive test suite for current behavior
- [ ] Integrate TaskQueue service
- [ ] Integrate StatusManager service  
- [ ] Integrate ResultStore service
- [ ] Integrate AgentRegistry service
- [ ] Remove old implementations
- [ ] Update documentation
- [ ] Performance validation
- [ ] Production deployment plan

## Why This Wasn't Done Immediately

Integrating these services requires careful consideration because:

1. **Running System**: The orchestrator is actively used in production
2. **Complex Dependencies**: Many parts of the system depend on the orchestrator
3. **Risk of Breakage**: Hasty integration could introduce subtle bugs
4. **Testing Requirements**: Needs comprehensive testing before integration

## Next Steps

1. Create a dedicated branch for this migration
2. Set up comprehensive test coverage
3. Integrate services one at a time
4. Thoroughly test each integration
5. Get code review before merging

## Estimated Complexity

- **Risk Level**: 8 (High)
- **Complexity Rating**: 8 (High)
- **Specific Risk Factors**:
  - Concurrent task execution edge cases
  - State consistency during migration
  - Performance implications of service boundaries
  - Integration with existing Discord bot and API

---

*This migration should be done carefully in a dedicated effort, not as part of other work.*
