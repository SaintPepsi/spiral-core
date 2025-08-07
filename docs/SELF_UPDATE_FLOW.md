# Self-Update System Flow

This document clarifies the complete flow of the self-update system, including all validation phases and git operations.

## Complete Update Flow

### Phase 1: Initialization
1. **System Lock Acquisition** - Prevent concurrent updates
2. **Preflight Checks** - Verify system is ready for updates
3. **Request Validation** - Validate update request parameters

### Phase 2: Planning
4. **Request Analysis** - Claude analyzes the update request
5. **Plan Creation** - Generate detailed implementation plan
6. **Plan Presentation** - Show plan to user for review

### Phase 3: Approval
7. **User Review** - User reviews the implementation plan
8. **Approval Decision** - User approves, rejects, or requests modifications
9. **Approval Gate** - Only proceed if approved

### Phase 4: Implementation
10. **Git Snapshot** - Create safety backup before any changes
11. **Code Modification** - Claude applies changes to working directory
12. **Change Tracking** - Log all modifications for audit trail

### Phase 5: Pre-Restart Validation
13. **Phase 1: Engineering Review** - Engineers reviewing all the work on modified files
    - Part 1: Code Standards Review
    - Part 2: Test Coverage Analysis
    - Part 3: Security Inspection
    - Part 4: Integration Review
14. **Phase 2: Final Assembly Checklist** - Ticking boxes before rolling off the line
    - Part 1: ✓ Compilation Check (`cargo check`)
    - Part 2: ✓ Test Execution (`cargo test`)
    - Part 3: ✓ Formatting Check (`cargo fmt`)
    - Part 4: ✓ Linting Check (`cargo clippy`)
    - Part 5: ✓ Documentation Build (`cargo doc`)
15. **Validation Gate** - If ANY check fails, rollback to snapshot

### Phase 6: System Restart
16. **Shutdown Notification** - Announce system restart
17. **Process Termination** - Gracefully shutdown current system
18. **System Startup** - Start system with new code
19. **Boot Verification** - Ensure system started successfully

### Phase 7: Post-Restart Validation
20. **Health Checks** - Verify system is functioning properly
21. **Integration Tests** - Run tests on live system
22. **Performance Validation** - Ensure no performance degradation
23. **Stability Monitoring** - Confirm system is stable

### Phase 8: Git Operations
24. **Stage Changes** - `git add -A`
25. **Commit Changes** - Create descriptive commit message
26. **Push to Remote** - `git push` to repository
27. **Push Verification** - Confirm successful push

### Phase 9: Completion
28. **Success Notification** - Report successful update to Discord
29. **Change Summary** - Provide detailed report of modifications
30. **System Lock Release** - Allow future updates

## Key Decision Points

### Rollback Triggers
- Pre-restart validation failure → Rollback, no restart
- Post-restart validation failure → Rollback, restart with old code
- System instability detected → Rollback, restart with old code

### Success Criteria
- All validation checks pass
- System restarts successfully
- No performance degradation
- Changes pushed to repository

## Important Clarifications

### "Live System" Definition
The "live system" refers to the running Spiral Core process, NOT the filesystem. Changes are made to files first, then validated, then applied to the live system through a restart.

### Validation Timing
- **Pre-Restart Validation**: Tests changes in working directory BEFORE restart
- **Post-Restart Validation**: Tests running system AFTER restart

### Git Operations Timing
- **Snapshot**: Created BEFORE implementation starts
- **Commit & Push**: Done AFTER all validation passes

## Safety Mechanisms

1. **System Lock** - Prevents concurrent updates
2. **Git Snapshot** - Enables rollback to known good state
3. **Two-Phase Validation** - Catches issues before and after restart
4. **Automatic Rollback** - Reverts on any validation failure
5. **Audit Trail** - Complete logging of all operations

## Error Recovery

### Pre-Restart Failure
1. Validation fails on modified files
2. Rollback to git snapshot
3. System continues running old code
4. Report failure to user

### Post-Restart Failure
1. System unhealthy after restart
2. Rollback to git snapshot
3. Restart with old code
4. Report failure to user

### Push Failure
1. Changes are live and validated
2. Log push failure as warning
3. Continue with success status
4. Manual push may be required

This flow ensures maximum safety while enabling autonomous self-improvement capabilities.