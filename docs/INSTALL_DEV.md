# Installing the REST Client Extension in Zed (Development Mode)

This guide shows you how to install and test your locally-built REST Client extension in Zed.

## Prerequisites

- Zed editor installed
- Successfully built the extension: `cargo build --target wasm32-wasip1 --release`
- WASM file exists at: `target/wasm32-wasip1/release/rest_client.wasm`

## Installation Methods

### Method 1: Using Zed's Extensions Directory (Recommended)

#### Step 1: Find Your Zed Extensions Directory

The location depends on your operating system:

- **macOS**: `~/Library/Application Support/Zed/extensions/`
- **Linux**: `~/.local/share/zed/extensions/`
- **Windows**: `%APPDATA%\Zed\extensions\`

Or run this command to create it if it doesn't exist:

```bash
# macOS
mkdir -p ~/Library/Application\ Support/Zed/extensions/

# Linux
mkdir -p ~/.local/share/zed/extensions/

# Windows (PowerShell)
New-Item -ItemType Directory -Force -Path "$env:APPDATA\Zed\extensions"
```

#### Step 2: Create the Extension Directory

Create a directory for your extension:

```bash
# macOS
mkdir -p ~/Library/Application\ Support/Zed/extensions/installed/rest-client

# Linux
mkdir -p ~/.local/share/zed/extensions/installed/rest-client

# Windows (PowerShell)
New-Item -ItemType Directory -Force -Path "$env:APPDATA\Zed\extensions\installed\rest-client"
```

#### Step 3: Copy Extension Files

From your `rest-client` project directory, copy the necessary files:

**macOS/Linux:**
```bash
# Navigate to your project
cd rest-client

# Set the extensions path (choose your OS)
# macOS:
EXTENSIONS_DIR=~/Library/Application\ Support/Zed/extensions/installed/rest-client
# Linux:
# EXTENSIONS_DIR=~/.local/share/zed/extensions/installed/rest-client

# Copy the extension files
cp extension.toml "$EXTENSIONS_DIR/"
cp -r languages "$EXTENSIONS_DIR/"
cp target/wasm32-wasip1/release/rest_client.wasm "$EXTENSIONS_DIR/extension.wasm"

echo "Extension installed to $EXTENSIONS_DIR"
```

**Windows (PowerShell):**
```powershell
# Navigate to your project
cd rest-client

# Set the extensions path
$EXTENSIONS_DIR = "$env:APPDATA\Zed\extensions\installed\rest-client"

# Copy the extension files
Copy-Item extension.toml $EXTENSIONS_DIR\
Copy-Item -Recurse languages $EXTENSIONS_DIR\
Copy-Item target\wasm32-wasip1\release\rest_client.wasm $EXTENSIONS_DIR\extension.wasm

Write-Host "Extension installed to $EXTENSIONS_DIR"
```

#### Step 4: Verify Installation

Check that the files are in place:

```bash
# macOS
ls -la ~/Library/Application\ Support/Zed/extensions/installed/rest-client/

# Linux
ls -la ~/.local/share/zed/extensions/installed/rest-client/

# Windows (PowerShell)
Get-ChildItem "$env:APPDATA\Zed\extensions\installed\rest-client"
```

You should see:
```
extension.toml
extension.wasm
languages/
  http/
    config.toml
    grammar.js
    highlights.scm
```

#### Step 5: Reload Zed

1. **Quit Zed completely** (don't just close windows)
   - macOS: `Cmd+Q`
   - Linux/Windows: `Alt+F4` or close from system tray

2. **Restart Zed**

3. **Verify the extension is loaded**:
   - Open Command Palette (`Cmd+Shift+P` or `Ctrl+Shift+P`)
   - Type "extensions"
   - Select "zed: extensions"
   - Look for "REST Client" in the list

## Testing the Extension

### Step 1: Create a Test File

Create a new file called `test.http`:

```http
### Simple GET request
GET https://api.github.com/users/octocat

###

### POST with JSON body
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "Test Post",
  "body": "Testing REST Client in Zed",
  "userId": 1
}

###

### GET with headers
GET https://httpbin.org/headers
User-Agent: Zed-REST-Client/1.0
Accept: application/json
```

### Step 2: Execute a Request

1. Open the `test.http` file in Zed
2. Place your cursor inside one of the request blocks
3. Open Command Palette (`Cmd+Shift+P` or `Ctrl+Shift+P`)
4. Search for "REST Client" or "Send Request"
5. Execute the command

### Step 3: View the Response

The response should appear in a new buffer/panel showing:
- HTTP status (will show 200 OK due to API limitations)
- Response headers
- Response body (formatted if JSON/XML)
- Request duration
- Response size

## Updating the Extension During Development

When you make changes to the code:

```bash
# 1. Make your code changes

# 2. Rebuild the extension
cargo build --target wasm32-wasip1 --release

# 3. Copy the new WASM file
# macOS:
cp target/wasm32-wasip1/release/rest_client.wasm ~/Library/Application\ Support/Zed/extensions/installed/rest-client/extension.wasm

# Linux:
cp target/wasm32-wasip1/release/rest_client.wasm ~/.local/share/zed/extensions/installed/rest-client/extension.wasm

# Windows (PowerShell):
Copy-Item target\wasm32-wasip1\release\rest_client.wasm $env:APPDATA\Zed\extensions\installed\rest-client\extension.wasm

# 4. Reload Zed
# Quit and restart Zed, or use the "Reload Extensions" command if available
```

## Troubleshooting

### Extension Doesn't Appear in Zed

**Check the installation path:**
```bash
# macOS
ls -la ~/Library/Application\ Support/Zed/extensions/installed/rest-client/extension.wasm

# Linux
ls -la ~/.local/share/zed/extensions/installed/rest-client/extension.wasm
```

If the file doesn't exist, review Step 3 above.

**Check Zed's extension logs:**
- Look for error messages in Zed's developer console
- Check if there are any WASM loading errors

**Verify extension.toml is correct:**
```bash
cat ~/Library/Application\ Support/Zed/extensions/installed/rest-client/extension.toml
```

Should show:
```toml
id = "rest-client"
name = "REST Client"
...
```

### .http Files Don't Get Syntax Highlighting

**Check language configuration:**
```bash
ls ~/Library/Application\ Support/Zed/extensions/installed/rest-client/languages/http/
```

Should contain:
- `config.toml`
- `grammar.js`
- `highlights.scm`

If missing, copy the `languages/` directory again.

### Commands Don't Appear in Command Palette

This indicates the extension isn't registering its commands. Common causes:

1. **WASM file is corrupted**: Rebuild and recopy
2. **Extension not loaded**: Check Zed logs
3. **Extension API mismatch**: Ensure `zed_extension_api` version matches Zed version

### Request Execution Fails

1. **Check network connectivity**: Try in a browser first
2. **Verify URL format**: Must start with `http://` or `https://`
3. **Check cursor position**: Must be inside a request block (between `###` markers)
4. **Look for error messages**: Zed should show error details

## Alternative: Symlink Method (For Active Development)

If you're actively developing and want to avoid copying files repeatedly:

```bash
# macOS/Linux
cd ~/Library/Application\ Support/Zed/extensions/installed/
# or for Linux: cd ~/.local/share/zed/extensions/installed/

# Remove the directory if it exists
rm -rf rest-client

# Create a symlink to your development directory
ln -s /path/to/your/rest-client rest-client

# Create a symlink for the WASM file specifically
cd rest-client
ln -s target/wasm32-wasip1/release/rest_client.wasm extension.wasm
```

**Note**: You'll still need to reload Zed after rebuilding the WASM file.

## Uninstalling the Extension

To remove the extension:

```bash
# macOS
rm -rf ~/Library/Application\ Support/Zed/extensions/installed/rest-client

# Linux
rm -rf ~/.local/share/zed/extensions/installed/rest-client

# Windows (PowerShell)
Remove-Item -Recurse -Force "$env:APPDATA\Zed\extensions\installed\rest-client"
```

Then restart Zed.

## Quick Install Script

Save this as `install-dev.sh` in your `rest-client` directory:

```bash
#!/bin/bash

# Detect OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    EXTENSIONS_DIR="$HOME/Library/Application Support/Zed/extensions/installed/rest-client"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    EXTENSIONS_DIR="$HOME/.local/share/zed/extensions/installed/rest-client"
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

echo "Installing REST Client extension to: $EXTENSIONS_DIR"

# Build the extension
echo "Building extension..."
cargo build --target wasm32-wasip1 --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

# Create extension directory
mkdir -p "$EXTENSIONS_DIR"

# Copy files
echo "Copying files..."
cp extension.toml "$EXTENSIONS_DIR/"
cp -r languages "$EXTENSIONS_DIR/"
cp target/wasm32-wasip1/release/rest_client.wasm "$EXTENSIONS_DIR/extension.wasm"

echo "✅ Extension installed successfully!"
echo "Please restart Zed to load the extension."
```

Make it executable and run:
```bash
chmod +x install-dev.sh
./install-dev.sh
```

## Next Steps

Once installed:
1. Create some `.http` test files
2. Try the examples in [QUICK_START.md](QUICK_START.md)
3. Report any issues or unexpected behavior
4. Review [known limitations](WASM_BUILD.md#known-limitations)

---

**Happy developing! 🚀**