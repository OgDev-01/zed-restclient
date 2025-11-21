# Migration Guide: VS Code REST Client to Zed REST Client

This guide helps you migrate from the popular [VS Code REST Client extension](https://github.com/Huachao/vscode-restclient) to the Zed REST Client extension.

## Overview

The Zed REST Client extension is designed to provide a familiar experience for users coming from VS Code REST Client. Most of your existing `.http` and `.rest` files will work without modification, and the syntax is intentionally kept compatible.

## Quick Migration Checklist

- [x] **File Extensions**: Both `.http` and `.rest` files work identically
- [x] **Request Syntax**: Compatible - GET, POST, PUT, DELETE, etc.
- [x] **Request Separator**: `###` delimiter works the same
- [x] **Comments**: Both `#` and `//` comments supported
- [x] **Headers**: Same format - `Header-Name: value`
- [x] **Variables**: `{{variableName}}` syntax identical
- [x] **System Variables**: All common system variables supported (`$guid`, `$timestamp`, etc.)
- [x] **Environment Files**: `.http-client-env.json` format compatible
- [x] **Request Chaining**: `# @capture` syntax for response variables
- [x] **GraphQL**: GraphQL queries supported in request bodies
- [x] **cURL**: Import/export cURL commands

## Syntax Compatibility

### ✅ Fully Compatible Features

#### Basic Requests

Both extensions support identical request syntax:

```http
### Simple GET request
GET https://api.example.com/users

### POST with JSON body
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

### Request with multiple headers
GET https://api.example.com/data
Accept: application/json
Authorization: Bearer {{token}}
User-Agent: My-App/1.0
```

#### Request Delimiters

The `###` delimiter works exactly the same:

```http
### First request
GET https://api.example.com/endpoint1

### Second request
GET https://api.example.com/endpoint2
```

#### Comments

Both comment styles are supported:

```http
# This is a comment
// This is also a comment

### Get user profile
# TODO: Add error handling
GET https://api.example.com/profile
```

#### Variables

Variable syntax is identical:

```http
@baseUrl = https://api.example.com
@token = your-token-here

### Use variables
GET {{baseUrl}}/users
Authorization: Bearer {{token}}
```

#### System Variables

All common system variables work the same:

```http
### System variables
POST https://api.example.com/data
Content-Type: application/json

{
  "requestId": "{{$guid}}",
  "timestamp": {{$timestamp}},
  "datetime": "{{$datetime iso8601}}",
  "random": {{$randomInt 1 100}}
}
```

Supported system variables:
- `{{$guid}}` - Generate UUID
- `{{$timestamp}}` - Unix timestamp
- `{{$timestamp -1 d}}` - Timestamp with offset
- `{{$datetime rfc1123}}` - RFC1123 formatted datetime
- `{{$datetime iso8601}}` - ISO8601 formatted datetime
- `{{$randomInt min max}}` - Random integer in range
- `{{$processEnv VAR_NAME}}` - Process environment variable
- `{{$dotenv VAR_NAME}}` - Variable from .env file

#### Environment Files

The `.http-client-env.json` format is identical:

```json
{
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "{{$processEnv DEV_API_KEY}}"
  },
  "staging": {
    "baseUrl": "https://staging-api.example.com",
    "apiKey": "{{$processEnv STAGING_API_KEY}}"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "{{$processEnv PROD_API_KEY}}"
  }
}
```

#### Request Variables (Response Capture)

The `# @capture` syntax is supported:

```http
### Login
POST {{baseUrl}}/auth/login
Content-Type: application/json

{
  "username": "user@example.com",
  "password": "{{password}}"
}

# @capture authToken = $.token

### Use captured token
GET {{baseUrl}}/protected-resource
Authorization: Bearer {{authToken}}
```

## Key Differences

### Command Differences

| Feature | VS Code REST Client | Zed REST Client | Notes |
|---------|---------------------|-----------------|-------|
| Send Request | `Ctrl/Cmd+Alt+R` or CodeLens | CodeLens "Send Request" | Configure custom keybinding if needed |
| Switch Environment | Environment selector | `/switch-environment` command | Use command palette |
| View History | History panel | Coming soon | History stored, UI in development |
| Generate Code | Context menu | "Generate Code" command | Same functionality |

### Configuration Differences

VS Code REST Client settings need to be migrated to Zed's `settings.json`:

**VS Code (`settings.json`):**
```json
{
  "rest-client.timeoutinmilliseconds": 30000,
  "rest-client.followRedirect": true,
  "rest-client.defaultHeaders": {
    "User-Agent": "vscode-restclient"
  }
}
```

**Zed (`settings.json`):**
```json
{
  "rest-client": {
    "timeout": 30000,
    "followRedirects": true,
    "defaultHeaders": {
      "User-Agent": "Zed-REST-Client"
    }
  }
}
```

See [CONFIGURATION.md](./CONFIGURATION.md) for all available settings.

### Settings Mapping

| VS Code Setting | Zed Setting | Notes |
|----------------|-------------|-------|
| `timeoutinmilliseconds` | `timeout` | Same functionality |
| `followRedirect` | `followRedirects` | Note the plural form |
| `defaultHeaders` | `defaultHeaders` | Identical |
| `excludeHostsForProxy` | `excludeHostsFromProxy` | Slightly different name |
| `previewOption` | `responsePane` | Different name, values: "right", "below", "tab" |

### Feature Parity

| Feature | VS Code | Zed | Status |
|---------|---------|-----|--------|
| Basic HTTP requests | ✅ | ✅ | Fully compatible |
| Variables | ✅ | ✅ | Fully compatible |
| Environments | ✅ | ✅ | Fully compatible |
| System variables | ✅ | ✅ | Fully compatible |
| Request chaining | ✅ | ✅ | Fully compatible |
| GraphQL support | ✅ | ✅ | Fully compatible |
| cURL import/export | ✅ | ✅ | Fully compatible |
| Code generation | ✅ | ✅ | Fully compatible |
| Syntax highlighting | ✅ | ✅ | Tree-sitter based |
| Auto-completion | ✅ | ✅ | LSP-based |
| Request history | ✅ | ⚠️ | Stored, UI coming soon |
| Authentication helpers | ✅ | ✅ | Basic, Bearer supported |
| Response formatting | ✅ | ✅ | JSON, XML, HTML |
| CodeLens | ✅ | ✅ | "Send Request" lens |
| Multi-part forms | ⚠️ | ⚠️ | Basic support |
| Certificate management | ⚠️ | ⚠️ | Coming soon |

**Legend:**
- ✅ Fully supported
- ⚠️ Partial support or in development
- ❌ Not supported

## Migration Steps

### Step 1: Install Zed REST Client

1. Open Zed editor
2. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux/Windows)
3. Search for "zed: extensions"
4. Find "REST Client" and click Install

Or install manually from the extension directory.

### Step 2: Copy Your HTTP Files

Your existing `.http` and `.rest` files can be used directly:

```bash
# Simply copy your files to your Zed project
cp ~/vscode-projects/api-tests/*.http ~/zed-projects/api-tests/
```

No modifications needed for most files!

### Step 3: Migrate Environment Files

Copy your `.http-client-env.json` files:

```bash
# Copy environment configuration
cp ~/vscode-projects/api-tests/.http-client-env.json ~/zed-projects/api-tests/
```

The format is identical, so no changes are required.

### Step 4: Update Settings (Optional)

If you have custom REST Client settings in VS Code, migrate them to Zed's `settings.json`:

1. Open VS Code settings and locate `rest-client.*` settings
2. Open Zed settings: `Cmd+,` (macOS) or `Ctrl+,` (Linux/Windows)
3. Add equivalent settings under the `"rest-client"` key (see mapping table above)

### Step 5: Set Up Environment Variables

If you use `{{$processEnv}}` variables:

```bash
# Copy your .env file if you have one
cp ~/vscode-projects/api-tests/.env ~/zed-projects/api-tests/

# Or set environment variables for Zed
export API_KEY="your-api-key"
export BASE_URL="https://api.example.com"
```

### Step 6: Test Your Requests

1. Open an `.http` file in Zed
2. Click the "Send Request" CodeLens above a request
3. Verify the response appears correctly

### Step 7: Configure Keybindings (Optional)

If you want the same keyboard shortcuts as VS Code:

```json
{
  "context": "Editor && (extension == 'http' || extension == 'rest')",
  "bindings": {
    "ctrl-alt-r": "rest-client: send request",
    "ctrl-alt-e": "rest-client: switch environment"
  }
}
```

Add this to your Zed `keymap.json`.

## Common Migration Issues

### Issue 1: Request Not Executing

**Symptom:** Clicking "Send Request" does nothing

**Solution:**
- Ensure you have the latest version of the extension
- Check that the file has `.http` or `.rest` extension
- Verify the request syntax is valid (method, URL required)
- Check the Zed logs for error messages

### Issue 2: Variables Not Resolving

**Symptom:** Variables show as `{{variableName}}` in requests

**Solution:**
- Verify environment file is in the workspace root or parent directories
- Check environment file is valid JSON
- Use `/switch-environment` to ensure correct environment is active
- Verify variable names match exactly (case-sensitive)

### Issue 3: Environment Not Found

**Symptom:** "Environment file not found" warning

**Solution:**
- Create `.http-client-env.json` in your workspace root
- Ensure the file is valid JSON
- Check file permissions (must be readable)

### Issue 4: Authentication Not Working

**Symptom:** 401 Unauthorized responses

**Solution:**
- Verify `Authorization` header syntax: `Authorization: Bearer {{token}}`
- Check that variables containing tokens are properly resolved
- Ensure environment variables are set if using `{{$processEnv}}`
- Try hard-coding the token temporarily to isolate the issue

### Issue 5: Response Not Formatted

**Symptom:** Response shows as raw text instead of formatted JSON/XML

**Solution:**
- Verify the API returns proper `Content-Type` header
- Check that the response is valid JSON/XML
- Try toggling raw view to see the original response
- Large responses (>10MB) may not be formatted automatically

## Feature Comparison Examples

### Sending Requests

**VS Code:**
- Press `Ctrl/Cmd+Alt+R`
- Click "Send Request" CodeLens
- Right-click → "Send Request"

**Zed:**
- Click "Send Request" CodeLens
- Use custom keybinding (if configured)
- Command palette: "rest-client: send request"

### Switching Environments

**VS Code:**
- Click environment selector in status bar
- Press `Ctrl/Cmd+Alt+E`

**Zed:**
- Command palette: `/switch-environment`
- Select from list of available environments

### Viewing History

**VS Code:**
- View → Command Palette → "Rest Client: Request History"
- History panel shows all requests

**Zed:**
- History is automatically saved
- UI for viewing history coming soon
- Check `~/.config/zed/extensions/rest-client/history.json` directly

### Generating Code

**VS Code:**
- Right-click → "Generate Code Snippet"
- Select language and library

**Zed:**
- Command palette: "rest-client: generate code"
- Select language and library from dialog

## Advanced Features

### Request Chaining

Both extensions support request chaining with JSONPath:

```http
### Create a resource
POST {{baseUrl}}/api/resources
Content-Type: application/json

{
  "name": "New Resource"
}

# @capture resourceId = $.id

### Update the resource
PUT {{baseUrl}}/api/resources/{{resourceId}}
Content-Type: application/json

{
  "name": "Updated Resource"
}
```

### GraphQL Queries

GraphQL support is identical:

```http
### GraphQL query
POST {{baseUrl}}/graphql
Content-Type: application/json

{
  "query": "query { users { id name email } }"
}

### GraphQL with variables
POST {{baseUrl}}/graphql
Content-Type: application/json

{
  "query": "query GetUser($id: ID!) { user(id: $id) { name email } }",
  "variables": {
    "id": "{{userId}}"
  }
}
```

### cURL Integration

Import cURL commands the same way:

```bash
# Copy a cURL command from documentation
curl -X POST https://api.example.com/data \
  -H "Content-Type: application/json" \
  -d '{"key":"value"}'
```

In Zed:
1. Command palette: "rest-client: paste cURL"
2. The cURL command is converted to HTTP format

Export to cURL:
1. Position cursor in a request
2. Command palette: "rest-client: copy as cURL"
3. cURL command is copied to clipboard

## Tips for Smooth Migration

### 1. Keep Your Files Organized

```
project/
├── .http-client-env.json
├── api-tests/
│   ├── auth.http
│   ├── users.http
│   └── products.http
└── .env
```

### 2. Use Environment Variables for Secrets

**Don't do this:**
```json
{
  "production": {
    "apiKey": "sk_live_abc123xyz789"  // ❌ Secret in file
  }
}
```

**Do this:**
```json
{
  "production": {
    "apiKey": "{{$processEnv PROD_API_KEY}}"  // ✅ From environment
  }
}
```

### 3. Document Your Requests

```http
### User Authentication API
# Authenticates a user and returns a JWT token
# 
# Required environment variables:
# - BASE_URL: API base URL
# 
# Returns:
# - token: JWT authentication token (valid for 24 hours)
# - userId: User's unique identifier

POST {{BASE_URL}}/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "{{password}}"
}

# @capture authToken = $.token
# @capture userId = $.userId
```

### 4. Use Descriptive Variable Names

```http
# ✅ Good - clear and descriptive
@apiBaseUrl = https://api.example.com
@authToken = Bearer eyJhbGc...
@currentUserId = 12345

# ❌ Bad - unclear abbreviations
@url = https://api.example.com
@tk = Bearer eyJhbGc...
@id = 12345
```

### 5. Test in Development First

Before migrating production workflows:
1. Start with development environment
2. Test a few simple requests
3. Verify variables and environments work
4. Test request chaining if you use it
5. Verify code generation if you rely on it
6. Then migrate staging and production

## Getting Help

### Documentation

- **[Getting Started](./GETTING_STARTED.md)** - Basic usage guide
- **[Features Guide](./FEATURES.md)** - All features explained
- **[Configuration](./CONFIGURATION.md)** - Settings reference
- **[Variables](./VARIABLES.md)** - Variable usage guide
- **[Troubleshooting](./TROUBLESHOOTING.md)** - Common issues and solutions

### Examples

Check the `examples/` directory for working examples:
- `examples/basic-requests.http` - Simple requests
- `examples/with-variables.http` - Variable usage
- `examples/request-chaining.http` - Response capture
- `examples/graphql-examples.http` - GraphQL queries

### Community

- Report issues on GitHub
- Check existing issues for known problems
- Share your `.http` files as examples

## What's Different (Summary)

### Advantages of Zed REST Client

- ✅ **Faster**: Rust + WebAssembly implementation
- ✅ **Native Zed Integration**: Uses Zed's LSP and extension APIs
- ✅ **Tree-sitter Grammar**: Professional syntax highlighting
- ✅ **Performance**: Handles large responses efficiently

### Things to Be Aware Of

- ⚠️ **History UI**: Currently in development (history is saved, UI coming)
- ⚠️ **Extension Ecosystem**: Smaller than VS Code (but growing)
- ⚠️ **Platform Support**: Zed is macOS and Linux (Windows support coming)

## Conclusion

Migrating from VS Code REST Client to Zed REST Client is straightforward. Most of your existing files will work without modification. The syntax is intentionally kept compatible, and the features you rely on are available.

For most users, migration is as simple as:
1. Install the Zed REST Client extension
2. Copy your `.http` files and `.http-client-env.json`
3. Start sending requests!

If you encounter any issues, check the [Troubleshooting Guide](./TROUBLESHOOTING.md) or report them on GitHub.

Happy testing! 🚀