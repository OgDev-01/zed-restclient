# REST Client for Zed

A powerful HTTP client extension for Zed that brings professional API testing directly into your editor. Send HTTP requests, view formatted responses, manage environments, and chain requests—all without leaving your development workflow.

**Inspired by the popular VS Code REST Client extension.**

## Quick Install

```bash
git clone https://github.com/ogdev-01/zed-restclient.git && cd zed-restclient && ./install-dev.sh
```

Then restart Zed (Cmd+Q and reopen).

## ✨ Key Features

- **🚀 Full HTTP Support** - All HTTP methods (GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD)
- **📝 Simple Syntax** - Write requests in plain text `.http` or `.rest` files
- **🎨 Beautiful Responses** - Auto-formatted JSON, XML, and HTML with syntax highlighting
- **🔄 Request Chaining** - Capture response values and use in subsequent requests (JSONPath)
- **🌍 Environment Management** - Switch between dev, staging, and production with one command
- **📦 Powerful Variables** - System variables (`{{$guid}}`, `{{$timestamp}}`), environment vars, and custom variables
- **🔐 Secure Secrets** - Use environment variables to keep API keys out of version control
- **⚡ Code Generation** - Generate JavaScript, Python code from your requests
- **🌐 GraphQL Ready** - Full GraphQL query and mutation support
- **🔧 cURL Integration** - Import cURL commands, export requests as cURL
- **💡 Smart LSP Features** - Code lenses, auto-complete, hover hints, real-time diagnostics ([Learn more](docs/LSP_FEATURES.md))
- **📜 History Tracking** - Automatic request/response history

## 📖 Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Basic Usage](#basic-usage)
- [Documentation](#documentation)
- [Configuration](#configuration)
- [Examples](#examples)
- [LSP Features](#lsp-features)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Troubleshooting](#troubleshooting)
- [Migration from VS Code](#migration-from-vs-code)
- [Contributing](#contributing)

## Installation

### Prerequisites

Before installing the extension, you need to have Rust installed on your system.

#### Install Rust

If you don't have Rust installed, install it using [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

For Windows, download and run the installer from [rustup.rs](https://rustup.rs/).

After installation, add the WebAssembly target:

```bash
rustup target add wasm32-wasip1
```

**Verify your installation:**
```bash
rustc --version
cargo --version
```

You should see version numbers for both commands.

### Installation Steps

**⚠️ IMPORTANT: You cannot use "Install Dev Extension" from Zed's UI.**  
The extension requires building both WASM and a native LSP server binary.

#### Quick Install (Recommended)

1. Clone this repository:
   ```bash
   git clone https://github.com/ogdev-01/zed-restclient.git
   cd zed-restclient
   ```

2. Run the installation script:
   ```bash
   # macOS/Linux
   ./install-dev.sh
   
   # Windows (PowerShell)
   .\install-dev.ps1
   ```
   
   The script will automatically:
   - Check if Rust and Cargo are installed
   - Install the `wasm32-wasip1` target if not present
   - Build the LSP server and WASM extension
   - Copy files to the correct Zed directories

3. **Completely quit and restart Zed** (Cmd+Q, not just close the window)

4. Verify installation:
   - Open command palette (`Cmd+Shift+P`)
   - Type "zed: extensions"
   - You should see "REST Client" listed

#### What the Install Script Does

- ✅ Verifies Rust and Cargo are installed
- ✅ Automatically installs `wasm32-wasip1` target if missing
- ✅ Builds the LSP server binary (native, ~3.8MB with `reqwest`)
- ✅ Builds the WASM extension (~1.7MB)
- ✅ Copies all files to Zed's extension directories (`installed/` and `work/`)
- ✅ Sets correct permissions on the LSP server binary

#### Manual Build (Advanced)

If you prefer to build manually:
   ```bash
   cargo build --target wasm32-wasip1 --release
   ```
4. Install the extension in Zed by copying it to your extensions directory

See [Installation Guide](docs/INSTALL_DEV.md) for detailed build instructions and development setup.

## 🚀 Quick Start

### 1. Create an HTTP File

Create a file named `api-test.http`:

```http
### Get GitHub user
GET https://api.github.com/users/octocat

### Create a post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "My Post",
  "body": "This is the content",
  "userId": 1
}
```

### 2. Send a Request

1. Click the **"Send Request"** button that appears above each request
2. Or use the command palette: `Cmd+Shift+P` → "rest-client: send request"
3. View the formatted response in a split pane

### 3. Use Variables

Create `.http-client-env.json` in your workspace root:

```json
{
  "development": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key-123"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "{{$processEnv PROD_API_KEY}}"
  }
}
```

Use in your requests:

```http
GET {{baseUrl}}/users
Authorization: Bearer {{apiKey}}
```

Switch environments:
```
/switch-environment production
```

**👉 New to the extension?** Check out the [Getting Started Guide](docs/GETTING_STARTED.md) for a complete walkthrough.

## 📚 Basic Usage

## Configuration

The REST Client extension can be customized through Zed settings. Add configuration to your `settings.json`:

```json
{
  "rest-client": {
    "timeout": 30000,
    "validateSsl": true,
    "historyLimit": 1000,
    "responsePane": "right",
    "defaultHeaders": {
      "User-Agent": "Zed-REST-Client/1.0"
    }
  }
}
```

### Simple Request Format

**Minimal GET request:**
```http
GET https://api.example.com/users
```

Or even simpler (GET is assumed):
```http
https://api.example.com/users
```

**POST with JSON:**
```http
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

**With headers:**
```http
GET https://api.example.com/data
Accept: application/json
Authorization: Bearer YOUR_TOKEN
User-Agent: MyApp/1.0
```

**Multiple requests in one file:**
```http
### Get all users
GET https://api.example.com/users

### Create a user
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Jane Doe"
}

### Get specific user
GET https://api.example.com/users/123
```

Use `###` to separate requests.

### Variables and Environments

**File variables:**
```http
@baseUrl = https://api.example.com
@apiVersion = v1

GET {{baseUrl}}/{{apiVersion}}/users
```

**System variables:**
```http
POST {{baseUrl}}/events
Content-Type: application/json

{
  "id": "{{$guid}}",
  "timestamp": {{$timestamp}},
  "datetime": "{{$datetime iso8601}}"
}
```

**Environment files** (`.http-client-env.json`):
```json
{
  "development": {
    "baseUrl": "http://localhost:3000"
  },
  "production": {
    "baseUrl": "https://api.example.com"
  }
}
```

Switch with: `/switch-environment production`

### Request Chaining

Capture values from responses:

```http
### Login
POST {{baseUrl}}/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secret"
}

# @capture authToken = $.token

### Use the token
GET {{baseUrl}}/protected
Authorization: Bearer {{authToken}}
```

## 📖 Documentation

### Core Guides

| Guide | Description |
|-------|-------------|
| **[Getting Started](docs/GETTING_STARTED.md)** | Complete beginner's guide with examples |
| **[Features](docs/FEATURES.md)** | Detailed explanation of all features |
| **[Migration Guide](docs/MIGRATION.md)** | Moving from VS Code REST Client |
| **[Troubleshooting](docs/TROUBLESHOOTING.md)** | Common issues and solutions |

### Feature Documentation

| Topic | Description |
|-------|-------------|
| **[Configuration](docs/CONFIGURATION.md)** | All settings, defaults, and validation |
| **[Variables](docs/VARIABLES.md)** | Variable types, syntax, and usage |
| **[Environments](docs/ENVIRONMENTS.md)** | Environment management and switching |
| **[Request Chaining](docs/REQUEST_VARIABLES.md)** | JSONPath, response capture, workflows |
| **[GraphQL](docs/GRAPHQL.md)** | GraphQL queries and mutations |
| **[Code Generation](docs/CODE_GENERATION.md)** | Generate code in multiple languages |
| **[cURL Commands](docs/CURL_COMMANDS_USAGE.md)** | Import/export cURL |
| **[LSP Features](docs/LSP_FEATURES.md)** | Auto-complete, diagnostics, hover |

### Examples

All examples are in the [`examples/`](examples/) directory:

- `basic-requests.http` - Simple GET/POST/PUT/DELETE
- `with-variables.http` - All variable types
- `request-chaining.http` - Response capture and chaining
- `graphql-examples.http` - GraphQL queries
- `.http-client-env.json` - Multi-environment setup

## ⚙️ Configuration

Add to your Zed `settings.json`:

```json
{
  "rest-client": {
    "timeout": 30000,
    "followRedirects": true,
    "validateSSL": true,
    "historyLimit": 1000,
    "responsePane": "right",
    "defaultHeaders": {
      "User-Agent": "Zed-REST-Client/1.0"
    }
  }
}
```

### Common Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `timeout` | 30000 | Request timeout (milliseconds) |
| `validateSSL` | true | Validate SSL/TLS certificates |
| `followRedirects` | true | Follow HTTP redirects |
| `maxRedirects` | 5 | Maximum redirect hops |
| `historyLimit` | 1000 | Max requests in history |
| `responsePane` | "right" | Response position: "right", "below", "tab" |
| `defaultHeaders` | {} | Headers added to all requests |

**📘 See [Configuration Guide](docs/CONFIGURATION.md) for all settings and examples.**

## 📝 Examples

#### Simple GET Request

```http
GET https://api.github.com/users/octocat
```

#### GET Request with Headers

```http
GET https://api.example.com/data
Accept: application/json
Authorization: Bearer YOUR_TOKEN_HERE
```

#### POST Request with JSON Body

```http
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "My Post",
  "body": "This is the post content",
  "userId": 1
}
```

#### Multiple Requests in One File

Separate multiple requests with `###`:

```http
### Get all posts
GET https://jsonplaceholder.typicode.com/posts

### Create a new post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "New Post",
  "body": "Content here",
  "userId": 1
}

### Get a specific post
GET https://jsonplaceholder.typicode.com/posts/1
```

### Sending Requests

1. Place your cursor anywhere within the request you want to send
2. Open the command palette (`Cmd+Shift+P` on macOS, `Ctrl+Shift+P` on Linux/Windows)
3. Search for "rest-client: send request"
4. Press Enter

The response will appear in a new editor pane showing:
- HTTP status code and reason
- Response headers
- Formatted response body
- Request duration and size

### Comments

Add comments to your request files using `#` or `//`:

```http
# This is a comment
// This is also a comment

### Get user data
GET https://api.example.com/users/123
```

## Request Format Specification

### Request Line

```
METHOD URL [HTTP_VERSION]
```

- `METHOD`: HTTP method (GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD, TRACE, CONNECT)
- `URL`: Complete URL including protocol (http:// or https://)
- `HTTP_VERSION`: Optional, defaults to HTTP/1.1

### Headers

Headers follow the request line, one per line:

```
Header-Name: Header-Value
```

### Request Body

Add a blank line after headers, then include your request body:

```http
POST https://api.example.com/data
Content-Type: application/json

{
  "key": "value"
}
```

## Examples

Check the [`examples/`](examples/) directory for complete working examples:

```http
### Simple GET
GET https://api.github.com/users/octocat

### POST with JSON
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "My Post",
  "body": "Content here",
  "userId": 1
}

### With authentication
GET https://api.example.com/protected
Authorization: Bearer {{token}}

### Form data
POST https://httpbin.org/post
Content-Type: application/x-www-form-urlencoded

username=johndoe&password=secret
```

**More examples:**
- [Basic Requests](examples/basic-requests.http)
- [Variables](examples/with-variables.http)
- [Request Chaining](examples/request-chaining.http)
- [GraphQL](examples/graphql-examples.http)

## 💡 LSP Features

The REST Client includes a powerful Language Server that provides intelligent editing features:

### Code Lenses
Clickable **"▶ Send Request"** buttons appear above each HTTP request. Click to execute requests instantly.

```http
▶ Send Request
GET https://api.github.com/users/octocat

# @name CreateUser
▶ Send Request: CreateUser
POST https://api.example.com/users
```

### Variable Autocompletion
Type `{{` to trigger smart completions for:
- Environment variables from `.http-client-env.json`
- System variables (`$guid`, `$timestamp`, `$datetime`, `$randomInt`)
- File variables defined in your `.http` file

### Hover Information
Hover over variables to see:
- Current resolved value
- Variable source (environment, file, system)
- Detailed descriptions and examples

### Syntax Diagnostics
Real-time error detection for:
- Invalid HTTP methods and malformed URLs
- Undefined variables and typos
- JSON syntax errors in request bodies
- Missing or incorrect headers

### Environment Switching
Switch between dev, staging, and production environments seamlessly:

```http
/switch-environment production
```

All variable values update automatically based on the active environment.

**📘 See [LSP Features Guide](docs/LSP_FEATURES.md) for complete documentation with examples and troubleshooting.**

## ⌨️ Keyboard Shortcuts

Add custom shortcuts to your Zed `keymap.json`:

```json
{
  "context": "Editor && (extension == 'http' || extension == 'rest')",
  "bindings": {
    "ctrl-alt-r": "rest-client: send request",
    "ctrl-alt-e": "rest-client: switch environment",
    "ctrl-alt-g": "rest-client: generate code"
  }
}
```

**Available Commands:**
- `rest-client: send request` - Execute the current request
- `rest-client: switch environment` - Change active environment
- `rest-client: generate code` - Generate code from request
- `rest-client: copy as cURL` - Export as cURL command
- `rest-client: paste cURL` - Import cURL command
- `rest-client: save response` - Save response to file
- `rest-client: copy response` - Copy response to clipboard

## Troubleshooting

### Installation Issues

**Error: "can't find crate for `core`" or "wasm32-wasip1 target may not be installed"**

This means the WebAssembly target isn't installed. The install script should handle this automatically, but if you see this error:

```bash
rustup target add wasm32-wasip1
```

Then run the install script again.

**Error: "rustc: command not found" or "cargo: command not found"**

Rust is not installed or not in your PATH. Install Rust:

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download from https://rustup.rs/
```

After installation, restart your terminal and try again.

**Install script fails on Windows**

Make sure you're running PowerShell (not Command Prompt):
```powershell
.\install-dev.ps1
```

If you get execution policy errors:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Extension not loading

1. Ensure you have the latest version of Zed installed
2. Check the Zed logs for any error messages
3. **Completely quit Zed** (Cmd+Q on macOS, Alt+F4 on Windows) and restart
4. Try reinstalling the extension

### Requests failing

1. Verify your URL is correct and includes the protocol (`http://` or `https://`)
2. Check your internet connection
3. Verify any authentication tokens or API keys are valid
4. Check if the API endpoint requires specific headers

### Syntax highlighting not working

1. Ensure your file has the `.http` or `.rest` extension
2. Reload Zed or restart the editor
3. Check if the language mode is set to "HTTP" in the status bar

## 🔧 Troubleshooting

### Common Issues

**Request not sending?**
- Ensure URL includes `http://` or `https://`
- Check for syntax errors (red squiggly lines)
- Verify cursor is within the request block

**Variables not resolving?**
- Check variable names match exactly (case-sensitive)
- Ensure environment file exists and is valid JSON
- Use `/switch-environment` to verify active environment

**SSL errors?**
```json
{
  "rest-client": {
    "validateSSL": false  // For development only!
  }
}
```

**📘 See [Troubleshooting Guide](docs/TROUBLESHOOTING.md) for detailed solutions.**

## 🔄 Migration from VS Code

Migrating from VS Code REST Client? Your files will work as-is!

- ✅ Same `.http` file format
- ✅ Same variable syntax `{{var}}`
- ✅ Same environment file format
- ✅ Same request separator `###`
- ✅ Same system variables

**📘 See [Migration Guide](docs/MIGRATION.md) for complete details and settings mapping.**

## 🎯 Roadmap

**Completed:**
- ✅ Full HTTP method support
- ✅ Syntax highlighting (Tree-sitter)
- ✅ Response formatting (JSON, XML, HTML)
- ✅ Variable substitution and environments
- ✅ Request chaining with JSONPath
- ✅ GraphQL support
- ✅ Code generation (JavaScript, Python)
- ✅ cURL import/export
- ✅ LSP features (autocomplete, diagnostics)
- ✅ Configuration system

**Coming Soon:**
- ⏳ Request history UI
- ⏳ More code generation languages
- ⏳ Response time graphs
- ⏳ Certificate management

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Development

### Prerequisites

- Rust toolchain (latest stable)
- `wasm32-wasip1` target installed:
  ```bash
  rustup target add wasm32-wasip1
  ```

### Building the Extension

Build the WASM extension:

```bash
# Using the optimized build script (recommended)
./build-optimized.sh

# Or manually with cargo
cargo build --target wasm32-wasip1 --release
```

### Building the LSP Server

The REST Client includes a Language Server Protocol (LSP) server for enhanced editor features like auto-completion, hover hints, and diagnostics.

**Quick Build (Current Platform)**:

```bash
# macOS/Linux
./build-lsp.sh

# Windows
.\build-lsp.ps1
```

**Cross-Platform Build**:

```bash
# Build for all supported platforms
./build-lsp.sh --all

# Build for specific platform
./build-lsp.sh --target x86_64-apple-darwin
```

**Manual Build**:

```bash
# Build optimized release binary
cargo build --bin lsp-server --release

# Binary location:
# - macOS/Linux: target/release/lsp-server
# - Windows: target\release\lsp-server.exe
```

**Supported Platforms**:
- macOS (Intel): `x86_64-apple-darwin`
- macOS (Apple Silicon): `aarch64-apple-darwin`
- Linux (x86_64): `x86_64-unknown-linux-gnu`
- Windows (x86_64): `x86_64-pc-windows-msvc`

**Binary Size**: ~2.8MB (optimized with LTO and stripping)

For detailed build instructions, troubleshooting, and CI/CD integration, see [BUILD.md](BUILD.md).

### Testing

```bash
cargo test
```

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

- Inspired by [VS Code REST Client](https://github.com/Huachao/vscode-restclient)
- Built for [Zed](https://zed.dev/) editor
- Thanks to all contributors and the Zed community

## 📞 Support & Community

**Need help?**
- 📖 Check the [Documentation](docs/)
- 🐛 [Report bugs](https://github.com/yourusername/repo/issues)
- 💬 [Ask questions](https://github.com/yourusername/repo/discussions)
- 📝 Review [Examples](examples/)

**Found a bug?** Please include:
- Zed version
- Extension version
- Minimal `.http` file that reproduces the issue
- Error messages from logs

---

**Happy API Testing! 🚀**

*Built with ❤️ for the Zed community*