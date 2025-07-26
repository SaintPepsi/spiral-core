# Development Practices for Spiral Core

This document outlines specific development practices and restrictions for working on the Spiral Core project.

## Package Management Practices

### ❌ Prohibited Commands

**Never run global npm installs:**

```bash
# NEVER RUN THESE:
npm install -g <anything>
sudo npm install -g <anything>
```

**Never run destructive system commands:**

```bash
# NEVER RUN THESE - FILESYSTEM DESTRUCTION:
rm -rf /
rm -rf /*
rm -rf ~/*
rm -rf $HOME
dd if=/dev/zero of=/dev/sda
mkfs.*

# NEVER RUN THESE - PERMISSION/OWNERSHIP CHANGES:
sudo chown -R $USER:$(id -gn) /usr
sudo chmod -R 777 /
sudo chmod -R u+w /usr
sudo chmod -R 755 /etc

# NEVER RUN THESE - PROCESS/SYSTEM CONTROL:
sudo killall -9
pkill -f
sudo reboot
sudo shutdown
sudo halt
sudo systemctl stop
sudo service stop

# NEVER RUN THESE - NETWORK/SECURITY:
sudo iptables -F
sudo ufw disable
sudo setenforce 0
sudo passwd
sudo visudo

# NEVER RUN THESE - PACKAGE MANAGEMENT DESTRUCTION:
sudo apt-get remove --purge
sudo yum remove
sudo brew uninstall --force
npm install -g
sudo npm install -g

# NEVER RUN THESE - DISK/PARTITION OPERATIONS:
fdisk
parted
gparted
sudo umount -f

# NEVER RUN THESE - RECURSIVE OPERATIONS ON SYSTEM DIRS:
find / -delete
find /etc -exec rm -rf {} \;
chmod -R 000 /
```

### ✅ Approved Practices

**Use local package installations:**

```bash
# Always use local packages
npm install --save-dev <package-name>
npx <command>  # Use npx to run local packages
```

**Package management:**

```bash
npm audit                    # Check for vulnerabilities
npm ls --depth=0            # List installed packages
npm outdated                # Check for updates
```

## Reasoning

### Why No Global Installs?

1. **Consistency**: Global packages can vary between development environments
2. **Version Control**: Local packages are tracked in package.json with specific versions
3. **Isolation**: Prevents conflicts between different projects
4. **Reproducibility**: Other developers can run the exact same environment
5. **CI/CD Compatibility**: Build systems work better with local dependencies

### Why No Destructive Commands?

1. **Safety**: Prevents accidental system damage
2. **Security**: Avoids privilege escalation issues
3. **Reversibility**: Local changes are easier to undo than system-wide changes

## Pre-commit Automation

The project uses automated formatting and linting on commit:

```json
{
  "lint-staged": {
    "*.md": [
      "markdownlint --config .markdownlint.json --fix",
      "git add"
    ]
  }
}
```

This ensures consistent documentation formatting without manual intervention.

## Available Scripts

```bash
npm run lint:md         # Check markdown formatting
npm run lint:md:fix     # Auto-fix markdown issues
npm run format:md       # Alias for lint:md:fix
npm audit               # Security audit
```

## Development Environment Setup

1. **Clone repository**
2. **Install dependencies**: `npm install` (no -g flag needed)
3. **Use npx**: All commands run through `npx` for local package execution
4. **Commit hooks**: Automatically configured via husky

This approach ensures a consistent, safe, and reproducible development environment for all contributors.
