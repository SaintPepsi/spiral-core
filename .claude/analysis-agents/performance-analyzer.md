# Performance Analysis Agent

## Purpose

Analyze code changes for performance implications and optimization opportunities.

## Trigger

Post-commit hook (focus on .rs files with algorithmic changes)

## Analysis Tasks

1. **Complexity Analysis**
   - Big-O complexity of new/modified functions
   - Nested loop detection
   - Recursive depth concerns
   - Unnecessary allocations

2. **Resource Usage Patterns**
   - Excessive cloning
   - Arc/Mutex proliferation
   - Large structs being copied
   - Inefficient string operations

3. **Async/Await Patterns**
   - Blocking operations in async contexts
   - Missing parallelization opportunities
   - Inefficient task spawning
   - Lock contention risks

4. **Memory Patterns**
   - Memory leak risks
   - Unnecessary heap allocations
   - Large stack frames
   - Reference counting issues

## Output Format

Generate `reports/performance-analysis-report.md` with:

- Performance score (relative to baseline)
- Hot paths identified
- Optimization opportunities ranked by impact
- Benchmark suggestions
- Memory usage estimates

## Discord Notification Triggers

Notify if:

- O(n³) or worse complexity introduced
- Memory leak risk detected
- Performance regression >20% estimated
- Blocking operation in critical async path

## Report Template

```markdown
# Performance Analysis Report

**Generated**: [DATE]
**Commit**: [HASH]

## Performance Impact Summary
- Estimated Impact: [Positive/Negative/Neutral]
- Risk Level: [1-21 Fibonacci]

## Complexity Analysis
| Function | Complexity | Risk |
|----------|-----------|------|
| func_name | O(n) | Low |

## Optimization Opportunities
1. **[Highest Impact]**
   - Location: file:line
   - Current: [description]
   - Suggested: [improvement]
   - Expected Gain: X%

## Memory Concerns
[Any memory-related issues]

## Recommended Benchmarks
```rust
#[bench]
fn bench_[name](b: &mut Bencher) {
    // Suggested benchmark
}
```

```
