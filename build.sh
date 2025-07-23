#!/bin/bash

# VS Code Agent - Build and Install Script

echo "🔧 Building VS Code Agent..."

# Build the project
~/.cargo/bin/cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    
    # Copy to local bin for easy access
    mkdir -p ~/.local/bin
    cp target/release/vscode-agent ~/.local/bin/
    
    echo "📦 Installed to ~/.local/bin/vscode-agent"
    echo ""
    echo "🚀 Usage:"
    echo "   vscode-agent check              # Check VS Code setup"
    echo "   vscode-agent dev 'task'         # Generate code for task"
    echo "   vscode-agent list               # List generated workspaces"
    echo "   vscode-agent clean              # Clean old workspaces"
    echo "   vscode-agent test               # Run integration tests"
    echo ""
    echo "💡 Make sure ~/.local/bin is in your PATH:"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "🎉 VS Code Agent is ready!"
else
    echo "❌ Build failed!"
    exit 1
fi
