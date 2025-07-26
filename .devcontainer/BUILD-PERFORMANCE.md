# Dev Container Build Performance Guide

## 🐌 Why Docker Builds Take So Long

### Primary Bottlenecks

1. **Cargo Tool Compilation (90% of build time)**

   - `cargo install hurl` - Compiles 100+ dependencies (~2-5 minutes)
   - `cargo install cargo-watch` - Heavy dependencies (~3-8 minutes)
   - `cargo install cargo-edit` - Complex tool chain (~2-5 minutes)

2. **ARM64 Architecture**

   - Most tools don't have pre-built ARM64 binaries
   - Everything must be compiled from source
   - Cross-compilation adds overhead

3. **Dependency Downloads**
   - Each cargo install downloads crates individually
   - No shared caching between tools
   - Network latency for crate downloads

## 🚀 Optimization Solutions

### Option 1: Fast Build (Recommended for Development)

**Use**: `devcontainer.fast.json`
**Build Time**: ~30 seconds
**Strategy**: Install tools on first container startup

```bash
cp .devcontainer/devcontainer.fast.json .devcontainer/devcontainer.json
```

### Option 2: Pre-built Tools (Advanced)

Create a custom base image with tools pre-installed:

```dockerfile
# Build once, use many times
FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye as builder
RUN cargo install hurl cargo-watch cargo-edit --locked

FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye
COPY --from=builder /usr/local/cargo/bin/* /usr/local/cargo/bin/
```

### Option 3: Tool Selection

Only install what you actually use:

```bash
# Essential only
cargo install cargo-watch --locked

# Install others on-demand
./scripts/install-cargo-tools.sh cargo-audit
```

## 📊 Performance Comparison

| Configuration | Build Time                  | Container Size | First Startup    |
| ------------- | --------------------------- | -------------- | ---------------- |
| Fast Build    | 30s                         | 1.5GB          | +2-3 min (tools) |
| Full Build    | 10-15 min                   | 2.7GB          | Immediate        |
| Custom Base   | 5 min (first) / 30s (after) | 2.2GB          | Immediate        |

## 🛠️ Caching Strategies

### Docker Layer Caching

- Keep heavy operations in separate layers
- Order Dockerfile commands by change frequency
- Use BuildKit for better caching

### Registry Caching

```bash
# Use registry cache for team development
docker buildx build --cache-from=registry.com/spiral-core:cache
```

### Local Caching

```bash
# Enable BuildKit
export DOCKER_BUILDKIT=1

# Use local cache mount
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install hurl --locked
```

## 🎯 Recommendations

**For Daily Development**: Use Fast Build

- Quick container rebuilds
- Tools installed once on first startup
- Faster iteration cycles

**For CI/CD**: Use Full Build

- Predictable environment
- All tools immediately available
- Better for automated workflows

**For Teams**: Consider Custom Base Image

- Build once, share with team
- Consistent tool versions
- Balance of speed and convenience
