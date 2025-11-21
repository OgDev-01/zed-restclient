# Quick Installation Guide

## TL;DR - Install in 3 Steps

### 1. Build
```bash
cd rest-client
cargo build --target wasm32-wasip1 --release
```

### 2. Run Install Script

**macOS/Linux:**
```bash
./install-dev.sh
```

**Windows:**
```powershell
.\install-dev.ps1
```

### 3. Restart Zed
- Quit Zed completely (Cmd+Q / Alt+F4)
- Restart Zed
- Verify: `Cmd+Shift+P` → "zed: extensions" → Look for "REST Client"

---

## Manual Installation (If Scripts Don't Work)

### Find Your Extensions Directory

- **macOS**: `~/Library/Application Support/Zed/extensions/installed/rest-client`
- **Linux**: `~/.local/share/zed/extensions/installed/rest-client`  
- **Windows**: `%APPDATA%\Zed\extensions\installed\rest-client`

### Copy These Files

```
extension.toml          → [extensions]/rest-client/extension.toml
languages/              → [extensions]/rest-client/languages/
target/.../rest_client.wasm → [extensions]/rest-client/extension.wasm
```

**Important**: The WASM file MUST be renamed to `extension.wasm`

---

## Test It Works

1. Create `test.http`:
```http
GET https://api.github.com/users/octocat
```

2. Open in Zed
3. Cursor in the request
4. `Cmd+Shift+P` → Search "REST" or "Send Request"
5. Execute!

---

## Troubleshooting

**Extension doesn't appear?**
- Check the file path is correct
- Verify `extension.wasm` exists and is 2-4 MB
- Restart Zed completely

**No syntax highlighting?**
- Ensure `languages/` folder was copied
- Check `languages/http/config.toml` exists

**Commands don't work?**
- Check Zed logs for errors
- Rebuild: `cargo clean && cargo build --target wasm32-wasip1 --release`
- Reinstall

---

## Updating After Code Changes

```bash
# Rebuild
cargo build --target wasm32-wasip1 --release

# Reinstall
./install-dev.sh  # or install-dev.ps1 on Windows

# Restart Zed
```

---

## Full Documentation

See [INSTALL_DEV.md](docs/INSTALL_DEV.md) for complete installation guide.