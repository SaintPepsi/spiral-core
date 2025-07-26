# Dev Container Guide - Spiral Core

## 🚀 Quick Start (Fast Build)

The default setup uses a minimal Rust base that builds dependencies locally for faster container startup:

```json
// devcontainer.json
"dockerComposeFile": "docker-compose.yml"
```

This approach:

- ✅ Container ready in ~2 minutes
- ✅ Uses cargo cache for fast incremental builds
- ✅ Perfect for daily development

## 🔧 Alternative: Pre-built Dependencies

If you need all dependencies pre-compiled in the container image, you can switch to the full Dockerfile:

1. Edit `.devcontainer/devcontainer.json`
2. Change the dockerfile reference:

   ```json
   "dockerFile": "Dockerfile"  // Instead of Dockerfile.fast
   ```

3. Rebuild the container: `Cmd+Shift+P` → "Dev Containers: Rebuild Container"

Use the full build when you need:

- Pre-compiled dependencies in the image
- Reproducible CI/CD environments
- Sharing the container with team members

## 📊 Performance Comparison

| Build Type         | Initial Build | Startup Time | Incremental Build | Use Case       |
| ------------------ | ------------- | ------------ | ----------------- | -------------- |
| **Fast (default)** | 2-3 min       | 30 sec       | 10-30 sec         | Development    |
| **Full**           | 10-15 min     | Immediate    | 3-5 min           | CI/CD, Sharing |

## 🛠️ Available Tools

Both configurations include:

- Rust toolchain with clippy and rustfmt
- cargo-watch for auto-rebuild
- cargo-edit for dependency management
- hurl for API testing
- VS Code extensions for Rust development

## 💡 Tips

1. **First Time Setup**: The fast build will compile dependencies on first run (~5 min)
2. **Cargo Cache**: Mounted at `/usr/local/cargo` - persists between rebuilds
3. **VS Code Settings**: Automatically configured for optimal Rust development

## 🔍 How to Check Current Build Type

Look at the container name in VS Code's status bar:

- Fast build: Shows "Fast Build" in the container name
- Full build: Standard container name

Or check the devcontainer.json:

```bash
# Fast build uses:
"dockerComposeFile": "docker-compose.yml"

# Full build would use:
"dockerFile": "Dockerfile"
```
