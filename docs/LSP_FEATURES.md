# LSP Features for REST Client

A comprehensive guide to the Language Server Protocol (LSP) features that power intelligent editing for `.http` and `.rest` files in the REST Client extension.

## Table of Contents

- [Introduction](#introduction)
- [Code Lenses](#code-lenses)
- [Variable Autocompletion](#variable-autocompletion)
- [Hover Information](#hover-information)
- [Syntax Diagnostics](#syntax-diagnostics)
- [Environment Switching](#environment-switching)
- [Troubleshooting](#troubleshooting)

## Introduction

The REST Client extension includes a built-in Language Server that provides intelligent editing features for HTTP request files. These features work automatically when you open `.http` or `.rest` files in Zed, enhancing your productivity with:

- **Code Lenses** - Clickable "Send Request" buttons above each HTTP request
- **Variable Autocompletion** - Smart suggestions for environment and system variables
- **Hover Information** - View variable values and metadata on hover
- **Syntax Diagnostics** - Real-time error detection and validation
- **Environment Switching** - Seamlessly switch between dev, staging, and production

All LSP features work together to provide a seamless API testing experience directly in your editor.

## Code Lenses

Code lenses are interactive buttons that appear above HTTP requests, allowing you to execute requests with a single click.

### What You'll See

When you open an `.http` file, you'll see clickable **"▶ Send Request"** buttons above each HTTP request:

```http
▶ Send Request
GET https://api.github.com/users/octocat

▶ Send Request
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe"
}
```

### Named Requests

Use the `@name` directive to give your requests meaningful names. The code lens will display the request name:

```http
# @name GetUserProfile
▶ Send Request: GetUserProfile
GET https://api.example.com/users/me
Authorization: Bearer {{token}}
```

**Benefits of named requests:**
- Easier to identify requests in large files
- Better organization and documentation
- More meaningful code lens labels
- Useful for request chaining (capture responses by name)

### How It Works

1. **Automatic Detection** - The LSP server scans your file and identifies HTTP requests
2. **Code Lens Generation** - A clickable button is added above each request
3. **Click to Execute** - Click the lens to send the request
4. **View Response** - Response appears in a split pane with formatting

### Supported HTTP Methods

Code lenses appear for all HTTP methods:
- `GET`, `POST`, `PUT`, `PATCH`, `DELETE`
- `HEAD`, `OPTIONS`, `TRACE`, `CONNECT`

### Request Separation

Use `###` to explicitly separate requests:

```http
### User Management

# @name ListUsers
GET https://api.example.com/users

###

# @name CreateUser
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Jane Doe"
}
```

## Variable Autocompletion

Get intelligent suggestions for variables as you type, with support for environment variables, file variables, and system variables.

### Triggering Autocompletion

Autocompletion is triggered automatically when you type `{{`:

```http
GET https://api.example.com/{{
                              ↑
                    Completion list appears here
```

### What You'll See

A completion list appears showing all available variables:

**Environment Variables** (from active environment):
- `baseUrl` → `http://localhost:3000`
- `apiKey` → `dev-key-123`
- `apiVersion` → `v1`

**System Variables** (dynamic):
- `$guid` → generates a new UUID v4
- `$timestamp` → current Unix timestamp
- `$datetime` → formatted datetime
- `$randomInt` → random integer
- `$processEnv` → process environment variable
- `$dotenv` → variable from .env file

**File Variables** (defined in your .http file):
- Custom variables defined with `@variableName = value`

### Completion Details

Each completion item shows:
- **Variable Name** - The text to insert
- **Current Value** - The resolved value (for environment variables)
- **Description** - What the variable does (for system variables)

### Example Usage

```http
# Type {{ and select from completions
GET {{baseUrl}}/api/{{apiVersion}}/users
Authorization: Bearer {{apiKey}}
X-Request-ID: {{$guid}}
```

### Variable Types

#### 1. Environment Variables

Defined in `.http-client-env.json`:

```json
{
  "development": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key-123"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "prod-key-xyz"
  }
}
```

#### 2. Shared Variables

Available across all environments:

```json
{
  "$shared": {
    "apiVersion": "v1",
    "timeout": "30"
  }
}
```

#### 3. File Variables

Defined at the top of your `.http` file:

```http
@baseUrl = https://api.example.com
@apiVersion = v1
@userId = 123

GET {{baseUrl}}/{{apiVersion}}/users/{{userId}}
```

#### 4. System Variables

Built-in dynamic variables:

- **`$guid`** - Generates UUID v4: `550e8400-e29b-41d4-a716-446655440000`
- **`$timestamp`** - Unix timestamp: `1700000000`
- **`$timestamp -1 d`** - Timestamp with offset (1 day ago)
- **`$datetime iso8601`** - ISO 8601: `2025-11-22T12:00:00.000Z`
- **`$datetime rfc1123`** - RFC 1123: `Mon, 22 Nov 2025 12:00:00 GMT`
- **`$randomInt 1 100`** - Random integer between 1 and 100
- **`$processEnv API_TOKEN`** - Read from process environment
- **`$dotenv API_KEY`** - Read from .env file

### Variable Resolution Priority

When multiple sources define the same variable name:

1. **System Variables** (prefix with `$`)
2. **Request Variables** (captured from previous responses)
3. **File Variables** (defined in .http file)
4. **Environment Variables** (from active environment)
5. **Shared Variables** (fallback)

## Hover Information

Hover over variables to see their current values, sources, and descriptions.

### How to Use

Move your mouse cursor over any variable reference (e.g., `{{baseUrl}}`) and a tooltip will appear showing detailed information.

### What You'll See

#### For Environment Variables

```
Variable: baseUrl
Value: http://localhost:3000
Source: environment variable (development)
```

#### For System Variables

```
System Variable: $guid
Description: generates a new UUID v4
Example value: 550e8400-e29b-41d4-a716-446655440000

⚠️ Will be resolved at runtime
```

#### For File Variables

```
Variable: userId
Value: 123
Source: file variable
```

#### For Undefined Variables

```
Variable: unknownVar

⚠️ Undefined variable

This variable is not defined in:
- Request variables
- File variables
- Environment variables
- Shared variables

Suggestion: Check for typos or define the variable
```

### Hover for System Variables

Detailed information for each system variable:

**`$guid`**
```
System Variable: $guid
Description: generates a new UUID v4
Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
Example: 550e8400-e29b-41d4-a716-446655440000
```

**`$timestamp`**
```
System Variable: $timestamp
Description: current Unix timestamp in seconds
Optional: offset with format [+/-]number [s|m|h|d]
Examples:
  {{$timestamp}}      → 1700000000
  {{$timestamp -1 d}} → 1699914000 (1 day ago)
  {{$timestamp +2 h}} → 1700007200 (2 hours from now)
```

**`$datetime`**
```
System Variable: $datetime
Description: formatted datetime string
Required: format (iso8601 or rfc1123)
Optional: offset [+/-]number [s|m|h|d]
Examples:
  {{$datetime iso8601}}           → 2025-11-22T12:00:00.000Z
  {{$datetime rfc1123}}           → Mon, 22 Nov 2025 12:00:00 GMT
  {{$datetime iso8601 -1 d}}      → 2025-11-21T12:00:00.000Z
```

**`$randomInt`**
```
System Variable: $randomInt
Description: random integer in range
Required: min and max values
Format: {{$randomInt min max}}
Example: {{$randomInt 1 100}} → 42
```

**`$processEnv`**
```
System Variable: $processEnv
Description: reads from process environment variables
Required: variable name
Optional: prefix with % to avoid errors if undefined
Examples:
  {{$processEnv API_TOKEN}}   → value or error if not set
  {{$processEnv %DEBUG_MODE}} → value or empty string
```

**`$dotenv`**
```
System Variable: $dotenv
Description: reads from .env file in workspace
Required: variable name
Format: {{$dotenv VAR_NAME}}
Example: {{$dotenv API_KEY}} → your-api-key-here
```

### Benefits

- **Verify Values** - Check variable values before sending requests
- **Catch Typos** - Identify undefined variables immediately
- **Learn Syntax** - See examples for system variables
- **Debug Issues** - Understand which environment is active

## Syntax Diagnostics

Real-time error detection and validation for your HTTP request files.

### What You'll See

Diagnostics appear as colored squiggly underlines:
- **Red** - Errors that will prevent the request from executing
- **Yellow** - Warnings that may cause issues
- **Blue** - Informational hints

### Error Types

#### 1. Invalid HTTP Method

```http
FETCH https://api.example.com/users
^^^^^
Error: Invalid HTTP method 'FETCH'
Suggestion: Use GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD, TRACE, or CONNECT
```

#### 2. Malformed URL

```http
GET api.example.com/users
    ^^^^^^^^^^^^^^^^^^^^^
Error: Invalid URL - missing protocol
Suggestion: Add http:// or https:// prefix
```

```http
GET https://api example.com/users
              ^
Error: URL contains whitespace
Suggestion: Remove spaces or URL-encode them
```

#### 3. Invalid Headers

```http
GET https://api.example.com/users
Content Type: application/json
^^^^^^^^^^^^
Error: Invalid header format
Suggestion: Use 'Content-Type' (hyphen, no space)
```

#### 4. Undefined Variables

```http
GET {{baseUrl}}/users
    ^^^^^^^^^
Warning: Undefined variable 'baseUrl'
Suggestion: Define in environment file or as file variable
```

#### 5. JSON Syntax Errors

```http
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
  ^
Error: Missing comma or closing brace
}
```

#### 6. Missing Required Headers

```http
POST https://api.example.com/users

{
  "name": "John"
}
^
Warning: Request body without Content-Type header
Suggestion: Add 'Content-Type: application/json'
```

#### 7. System Variable Syntax Errors

```http
X-Timestamp: {{$timestamp 1 d}}
                          ^
Error: Invalid offset format
Suggestion: Use format [+/-]number [s|m|h|d], e.g., '+1 d' or '-2 h'
```

```http
Date: {{$datetime}}
      ^^^^^^^^^^^^^
Error: datetime requires format argument
Suggestion: Use {{$datetime iso8601}} or {{$datetime rfc1123}}
```

### How to Fix Errors

1. **Hover over the error** - See detailed message and suggestions
2. **Follow the suggestion** - Apply the recommended fix
3. **Check documentation** - Review examples for correct syntax
4. **Verify variables** - Ensure all variables are defined

### Diagnostic Categories

| Category | Severity | Description |
|----------|----------|-------------|
| Syntax Errors | Error | Invalid HTTP syntax, malformed URLs |
| Variable Issues | Warning | Undefined variables, invalid syntax |
| Header Problems | Warning | Typos, missing required headers |
| JSON Validation | Error | Invalid JSON in request body |
| URL Validation | Error | Protocol missing, invalid characters |

### Best Practices

- **Fix errors before sending** - Diagnostics catch issues early
- **Use hover for details** - Get actionable suggestions
- **Define all variables** - Avoid "undefined" warnings
- **Validate JSON** - Use proper formatting and syntax
- **Check headers** - Ensure correct header names and formats

## Environment Switching

Seamlessly switch between different environments (dev, staging, production) without modifying your request files.

### Setting Up Environments

Create `.http-client-env.json` in your workspace root:

```json
{
  "$shared": {
    "apiVersion": "v1",
    "contentType": "application/json"
  },
  "development": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key-123",
    "debug": "true"
  },
  "staging": {
    "baseUrl": "https://staging-api.example.com",
    "apiKey": "{{$processEnv STAGING_API_KEY}}",
    "debug": "true"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "{{$processEnv PROD_API_KEY}}",
    "debug": "false"
  }
}
```

### Switching Environments

Use the command in your `.http` file:

```http
# Switch to production environment
/switch-environment production

# Now all requests use production variables
GET {{baseUrl}}/api/{{apiVersion}}/users
Authorization: Bearer {{apiKey}}
```

Or use the Zed command palette:
1. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux/Windows)
2. Type "rest-client: switch environment"
3. Select the environment from the list

### How It Works

1. **Variable Resolution** - Variables are resolved from the active environment
2. **Hot Reload** - Environment changes apply immediately (no restart needed)
3. **Shared Variables** - `$shared` variables are available in all environments
4. **Override Support** - Environment-specific values override shared values

### Environment Variable Usage

```http
### Development (default)
# baseUrl = http://localhost:3000
# apiKey = dev-key-123

GET {{baseUrl}}/users
Authorization: Bearer {{apiKey}}

###

# Switch to production
/switch-environment production

### Production
# baseUrl = https://api.example.com
# apiKey = [from process.env.PROD_API_KEY]

GET {{baseUrl}}/users
Authorization: Bearer {{apiKey}}
```

### Verifying Active Environment

Use hover to check which environment is active:

```http
# Hover over {{baseUrl}} to see:
Variable: baseUrl
Value: http://localhost:3000
Source: environment variable (development)
         ↑
         Shows active environment name
```

### Best Practices

#### 1. Use Shared Variables

Put common values in `$shared`:

```json
{
  "$shared": {
    "apiVersion": "v1",
    "userAgent": "REST-Client/1.0",
    "timeout": "30000"
  }
}
```

#### 2. Secure API Keys

Never hardcode production keys - use environment variables:

```json
{
  "production": {
    "apiKey": "{{$processEnv PROD_API_KEY}}"
  }
}
```

Then set in your shell:
```bash
export PROD_API_KEY="your-secret-key"
```

#### 3. Environment-Specific Settings

Customize behavior per environment:

```json
{
  "development": {
    "baseUrl": "http://localhost:3000",
    "validateSSL": "false",
    "logLevel": "debug"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "validateSSL": "true",
    "logLevel": "error"
  }
}
```

#### 4. Multiple Environment Files

Support different team setups:
- `.http-client-env.json` - Shared, committed to git
- `http-client.private.env.json` - Personal, in `.gitignore`

### Environment File Locations

The LSP searches for environment files in this order:
1. Workspace root: `.http-client-env.json`
2. Alternative: `http-client.env.json`
3. Private: `http-client.private.env.json`

## Troubleshooting

### LSP Not Starting

**Symptoms:**
- No code lenses appear
- Autocompletion doesn't work
- No hover information

**Solutions:**

1. **Check LSP server binary exists:**
   ```bash
   ls -la rest-client/lsp-server
   # Should show the binary file (~2.8MB)
   ```

2. **Rebuild the LSP server:**
   ```bash
   cd rest-client
   ./build-lsp.sh
   ```

3. **Check Zed LSP logs:**
   - Open Zed logs: Menu → Zed → View → Debug → LSP Logs
   - Look for errors related to "rest-client"

4. **Restart Zed:**
   - Quit Zed completely
   - Reopen and check if LSP starts

5. **Verify file extension:**
   - Ensure your file ends with `.http` or `.rest`
   - Check status bar shows "HTTP" language mode

### Code Lenses Not Appearing

**Symptoms:**
- No "Send Request" buttons above requests

**Solutions:**

1. **Check request syntax:**
   ```http
   # ✅ Correct - code lens appears
   GET https://api.example.com/users
   
   # ❌ Wrong - missing protocol
   GET api.example.com/users
   ```

2. **Ensure request separation:**
   ```http
   # Use ### to separate requests
   GET https://api.example.com/users
   
   ###
   
   POST https://api.example.com/users
   ```

3. **Check for syntax errors:**
   - Red squiggly lines indicate errors
   - Fix errors and code lenses will appear

4. **Reload the window:**
   - Command palette → "Reload Window"

### Autocompletion Not Working

**Symptoms:**
- No suggestions when typing `{{`
- Completion list is empty

**Solutions:**

1. **Type the trigger exactly:**
   ```http
   # ✅ Correct trigger
   GET {{
   
   # ❌ Won't trigger
   GET {
   ```

2. **Check environment file exists:**
   ```bash
   ls .http-client-env.json
   ```

3. **Validate environment file JSON:**
   ```bash
   # Check for syntax errors
   cat .http-client-env.json | jq .
   ```

4. **Verify active environment:**
   ```http
   /switch-environment development
   ```

5. **Check file variables syntax:**
   ```http
   # ✅ Correct
   @baseUrl = https://api.example.com
   
   # ❌ Wrong
   baseUrl = https://api.example.com
   ```

### Hover Not Showing Variable Values

**Symptoms:**
- Hover tooltip is empty
- Shows "undefined" for valid variables

**Solutions:**

1. **Ensure environment is loaded:**
   ```http
   # Add at top of file
   /switch-environment development
   ```

2. **Check variable is defined:**
   - Open `.http-client-env.json`
   - Verify variable exists in active environment or `$shared`

3. **Check spelling:**
   ```http
   # Variable names are case-sensitive
   GET {{baseUrl}}  # ✅ Correct
   GET {{baseurl}}  # ❌ Wrong
   ```

4. **Wait for LSP initialization:**
   - LSP needs a moment to start after opening file
   - Try hovering again after 1-2 seconds

### Variables Not Resolving

**Symptoms:**
- Variables shown as literal text in response
- Request fails with "undefined"

**Solutions:**

1. **Check environment file format:**
   ```json
   {
     "development": {
       "baseUrl": "http://localhost:3000"
     }
   }
   ```

2. **Verify environment is active:**
   - Use hover to check: "Source: environment variable (development)"

3. **Check for typos:**
   - Variable names must match exactly
   - Case-sensitive

4. **Use shared variables for common values:**
   ```json
   {
     "$shared": {
       "apiVersion": "v1"
     }
   }
   ```

### Diagnostics Showing False Errors

**Symptoms:**
- Red squiggly lines on valid syntax
- Incorrect error messages

**Solutions:**

1. **Check HTTP syntax:**
   ```http
   # ✅ Correct format
   GET https://api.example.com/users
   Content-Type: application/json
   
   # ❌ Wrong - blank line needed before body
   POST https://api.example.com/users
   Content-Type: application/json
   {"name": "John"}
   ```

2. **Validate JSON body:**
   - Use a JSON validator: https://jsonlint.com/
   - Check for missing commas, quotes, braces

3. **Verify headers format:**
   ```http
   # ✅ Correct
   Content-Type: application/json
   
   # ❌ Wrong
   Content Type: application/json
   ```

4. **Reload the window:**
   - Sometimes diagnostics need refresh
   - Command palette → "Reload Window"

### Environment Switching Not Working

**Symptoms:**
- Variables don't change after switching
- Wrong environment values used

**Solutions:**

1. **Check command syntax:**
   ```http
   # ✅ Correct
   /switch-environment production
   
   # ❌ Wrong
   /switch-environment: production
   ```

2. **Verify environment exists:**
   ```json
   {
     "development": { ... },
     "production": { ... }
     ↑
     Use exact name in switch command
   }
   ```

3. **Check for typos in environment name:**
   - Case-sensitive: "production" ≠ "Production"

4. **Reload environments:**
   - Save `.http-client-env.json`
   - LSP automatically reloads on file changes

### Common Errors

#### Error: "LSP server crashed"

**Solution:**
```bash
# Rebuild LSP server
cd rest-client
cargo build --bin lsp-server --release
```

#### Error: "Environment file not found"

**Solution:**
- Create `.http-client-env.json` in workspace root
- Or use alternative name: `http-client.env.json`

#### Error: "Invalid JSON in environment file"

**Solution:**
```bash
# Validate JSON
cat .http-client-env.json | jq .

# Fix common issues:
# - Missing commas between properties
# - Unquoted strings
# - Trailing commas
```

#### Error: "System variable syntax error"

**Solution:**
```http
# ✅ Correct system variable syntax
{{$timestamp}}
{{$timestamp -1 d}}
{{$datetime iso8601}}
{{$randomInt 1 100}}
{{$processEnv API_KEY}}

# ❌ Wrong
{{$timestamp(-1 d)}}      # No parentheses
{{$datetime}}             # datetime needs format
{{$randomInt}}            # randomInt needs min/max
```

### Getting Help

If you're still experiencing issues:

1. **Check the documentation:**
   - [Getting Started Guide](GETTING_STARTED.md)
   - [Configuration Guide](CONFIGURATION.md)
   - [Variables Guide](VARIABLES.md)

2. **Enable debug logging:**
   - Add to Zed settings:
   ```json
   {
     "lsp": {
       "rest-client": {
         "log_level": "debug"
       }
     }
   }
   ```

3. **Check examples:**
   - Review files in `examples/` directory
   - Compare with working examples

4. **Report a bug:**
   - Include Zed version
   - Include extension version
   - Include minimal `.http` file that reproduces issue
   - Include error messages from LSP logs

## See Also

- **[Getting Started Guide](GETTING_STARTED.md)** - Complete beginner's guide
- **[Features Overview](FEATURES.md)** - All extension features
- **[Variables Guide](VARIABLES.md)** - Detailed variable documentation
- **[Environments Guide](ENVIRONMENTS.md)** - Environment management
- **[Configuration Guide](CONFIGURATION.md)** - Settings and customization
- **[Troubleshooting Guide](TROUBLESHOOTING.md)** - General troubleshooting

---

**Need more help?** Check the [examples/](../examples/) directory for working demonstrations of all LSP features.