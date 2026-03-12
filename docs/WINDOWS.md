# Windows Development Setup for anytag-backend

## Overview

Windows development requires **WSL2 (Windows Subsystem for Linux 2)** because:

1. Nix works best on Linux
2. Docker requires WSL2 on Windows
3. Many development tools are Linux-first

## Quick Start (Recommended)

### Step 1: Install WSL2 with Ubuntu

```powershell
# Run in PowerShell as Administrator
wsl --install

# This installs:
# 1. WSL2 kernel
# 2. Ubuntu distribution
# 3. Sets WSL2 as default

# Restart your computer when prompted
```

### Step 2: Set Up Ubuntu

1. Launch "Ubuntu" from Start Menu
2. Create username/password (Linux credentials, separate from Windows)
3. Update packages:

   ```bash
   sudo apt update && sudo apt upgrade -y
   ```

### Step 3: Install Nix in WSL2 (Determinate Systems installer)

```bash
# In Ubuntu terminal
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# Follow the prompts, then restart shell
exit
# Re-open Ubuntu terminal
```

### Step 4: Install Docker Desktop for Windows

1. Download from <https://www.docker.com/products/docker-desktop/>
2. Install with WSL2 integration enabled
3. Launch Docker Desktop
4. In Settings → Resources → WSL Integration:
   - Enable integration with your Ubuntu distribution
   - Apply & Restart

### Step 5: Clone and Setup Project

```bash
# In Ubuntu terminal
cd ~
git clone <your-repository-url>
cd anytag-backend

# Allow direnv (optional but recommended)
sudo apt-get install direnv
echo 'eval "$(direnv hook bash)"' >> ~/.bashrc
source ~/.bashrc
direnv allow

# Or use nix develop directly
nix develop
```

## Alternative: Manual Setup (If Quick Start Fails)

### 1. Verify WSL2 Installation

```powershell
# In PowerShell
wsl --list --verbose
# Should show Ubuntu with version 2
```

### 2. Install Nix Manually

```bash
# In Ubuntu
sudo apt install curl
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# Add to .bashrc
echo '. /home/$USER/.nix-profile/etc/profile.d/nix.sh' >> ~/.bashrc
source ~/.bashrc
```

### 3. Configure Docker in WSL2

```bash
# Test Docker integration
docker --version
docker-compose --version

# If not working, ensure Docker Desktop WSL2 integration is enabled
```

## VS Code Setup (Recommended)

### 1. Install Extensions

- **WSL** (Microsoft)
- **Remote - Containers** (Microsoft)
- **rust-analyzer**
- **Nix IDE**

### 2. Open Project in WSL

```bash
# In Ubuntu terminal
cd ~/anytag-backend
code .
```

VS Code will:

1. Open in WSL mode
2. Use Linux toolchain
3. Access Docker via WSL2 integration

## Common Issues and Solutions

### Issue: "WSL2 requires update"

```powershell
# Update WSL2 kernel
wsl --update

# Set WSL2 as default
wsl --set-default-version 2
```

### Issue: Docker not working in WSL2

1. Open Docker Desktop
2. Go to Settings → Resources → WSL Integration
3. Ensure Ubuntu is checked
4. Click "Apply & Restart"

### Issue: Nix installation fails

```bash
# Try single-user installation
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install --no-daemon

# Or install via package manager
sudo apt install nix
```

### Issue: PostgreSQL connection refused

```bash
# Check if database is running
docker-compose ps

# Check Docker Desktop is running on Windows
# WSL2 Docker connects to Windows Docker Desktop
```

### Issue: Slow file system performance

```bash
# Move project to WSL2 home directory
mv /mnt/c/Users/.../anytag-backend ~/projects/
cd ~/projects/anytag-backend
```

## Performance Tips

### 1. Store project in WSL2 filesystem

- **Fast**: `~/projects/` (Linux filesystem)
- **Slow**: `/mnt/c/Users/...` (Windows filesystem)

### 2. Increase WSL2 memory (if needed)

Create `%UserProfile%\.wslconfig`:

```ini
[wsl2]
memory=4GB
processors=2
localhostForwarding=true
```

### 3. Use VS Code Remote WSL

- Better performance than Windows filesystem
- Full Linux toolchain access
- Seamless Docker integration

## Native Windows Development (Not Recommended)

### Why Not Native Windows?

1. Nix on Windows is experimental
2. Diesel PostgreSQL requires libpq (Linux library)
3. Docker requires WSL2 anyway
4. Development experience is suboptimal

### If You Must Use Native Windows

1. Install Rust via rustup (<https://rustup.rs/>)
2. Install diesel-cli: `cargo install diesel_cli --no-default-features --features postgres`
3. Install PostgreSQL from <https://www.postgresql.org/download/windows/>
4. Use PowerShell or Git Bash

## Verification Checklist

After setup, verify everything works:

```bash
# In Ubuntu terminal
cd ~/anytag-backend

# 1. Check Nix
nix-shell --version

# 2. Check Rust
rustc --version
cargo --version

# 3. Check Diesel
diesel --version

# 4. Check Docker
docker --version
docker-compose --version

# 5. Start environment
nix develop
# Should see welcome message with all tools listed

# 6. Test database
docker-compose up -d db
docker-compose ps  # Should show db running

# 7. Test migrations
diesel migration run

# 8. Test build
cargo build
```

## Getting Help

### Windows-Specific Issues

- WSL Documentation: <https://docs.microsoft.com/en-us/windows/wsl/>
- Docker WSL2: <https://docs.docker.com/desktop/windows/wsl/>

### Project Issues

- Check [DEVELOPMENT.md](./DEVELOPMENT.md) for general guidance
- Create issue in repository

### Nix Issues

- Nix documentation: <https://nixos.org/learn/>
- Nix Wiki: <https://nixos.wiki/>

## Summary

For Windows development:

1. ✅ Use WSL2 with Ubuntu
2. ✅ Install Nix inside WSL2
3. ✅ Use Docker Desktop with WSL2 integration
4. ✅ Use VS Code with WSL extension
5. ✅ Store project in WSL2 filesystem (~/projects/)

This gives you a Linux-like development environment with all the reproducibility benefits of Nix.
