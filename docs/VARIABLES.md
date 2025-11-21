# Variables Guide

Complete reference for using variables in the REST Client extension for Zed.

## Table of Contents

1. [Overview](#overview)
2. [Variable Syntax](#variable-syntax)
3. [System Variables](#system-variables)
4. [Environment Variables](#environment-variables)
5. [File Variables](#file-variables)
6. [Variable Resolution Order](#variable-resolution-order)
7. [Environment File Format](#environment-file-format)
8. [Switching Environments](#switching-environments)
9. [Nested Variables](#nested-variables)
10. [Security Best Practices](#security-best-practices)
11. [Examples](#examples)
12. [Troubleshooting](#troubleshooting)
13. [Migration from VS Code REST Client](#migration-from-vs-code-rest-client)

## Overview

Variables allow you to create dynamic, reusable HTTP requests by substituting values at runtime. The REST Client extension supports multiple types of variables:

- **System Variables**: Dynamically generated values (UUIDs, timestamps, random numbers)
- **Environment Variables**: User-defined values that vary by environment (dev, staging, production)
- **File Variables**: Variables defined within `.http` files (less common)
- **Request Variables**: Captured from previous responses (Phase 3 feature, coming soon)

### Why Use Variables?

- **Environment Management**: Easily switch between dev, staging, and production
- **Dynamic Values**: Generate unique IDs, timestamps, and random data
- **Security**: Keep secrets out of version control using environment variables
- **Reusability**: Define once, use everywhere in your requests
- **Request Chaining**: Capture values from responses to use in subsequent requests

## Variable Syntax

Variables are enclosed in double curly braces: `{{variableName}}`

```http
GET {{baseUrl}}/api/users
Authorization: Bearer {{apiKey}}
X-Request-ID: {{$guid}}
```

### System Variables

System variables start with a dollar sign (`$`) and are resolved at request execution time:

```http
{{$guid}}              # Generate new UUID
{{$timestamp}}         # Current Unix timestamp
{{$datetime iso8601}}  # Formatted datetime
{{$randomInt 1 100}}   # Random integer
{{$processEnv VAR}}    # Process environment variable
{{$dotenv VAR}}        # Variable from .env file
```

### Environment Variables

Environment variables are defined in `.http-client-env.json` and resolved based on the active environment:

```http
{{baseUrl}}      # From active environment or $shared
{{apiKey}}       # Environment-specific value
{{apiVersion}}   # Can be overridden per environment
```

### File Variables

Variables can be defined at the file level (less common, mainly for testing):

```http
@host = api.example.com
@port = 8080

GET https://{{host}}:{{port}}/api/users
```

**Note**: Environment variables are preferred over file variables for better organization.

## System Variables

System variables generate dynamic values at runtime. All system variables start with `$`.

### `{{$guid}}`

Generates a new UUID v4 (universally unique identifier).

**Usage:**
```http
POST https://api.example.com/users
Content-Type: application/json
X-Request-ID: {{$guid}}

{
  "id": "{{$guid}}",
  "correlationId": "{{$guid}}"
}
```

**Note**: Each `{{$guid}}` generates a **new** unique ID.

### `{{$timestamp}}`

Returns the current Unix timestamp in seconds.

**Usage:**
```http
GET https://api.example.com/data?timestamp={{$timestamp}}
```

**With Offset:**
```http
# One day ago
GET https://api.example.com/data?since={{$timestamp -1 d}}

# Two hours from now
GET https://api.example.com/data?until={{$timestamp +2 h}}
```

**Supported offset units:**
- `s` - seconds
- `m` - minutes
- `h` - hours
- `d` - days

**Format**: `{{$timestamp [+/-]number unit}}`

**Examples:**
- `{{$timestamp -30 m}}` - 30 minutes ago
- `{{$timestamp +7 d}}` - 7 days from now
- `{{$timestamp -3600 s}}` - 1 hour ago (3600 seconds)

### `{{$datetime}}`

Returns formatted datetime strings in RFC 1123 or ISO 8601 format.

**Formats:**
- `rfc1123` - RFC 1123 format (e.g., "Mon, 21 Nov 2025 12:00:00 GMT")
- `iso8601` - ISO 8601 format (e.g., "2025-11-21T12:00:00.000Z")

**Usage:**
```http
# RFC 1123 (common in HTTP headers)
GET https://api.example.com/data
If-Modified-Since: {{$datetime rfc1123}}

# ISO 8601 (common in JSON APIs)
POST https://api.example.com/events
Content-Type: application/json

{
  "createdAt": "{{$datetime iso8601}}"
}
```

**With Offset:**
```http
# RFC 1123, 1 day ago
If-Modified-Since: {{$datetime rfc1123 -1 d}}

# ISO 8601, 2 hours from now
{
  "scheduledAt": "{{$datetime iso8601 +2 h}}"
}
```

**Format**: `{{$datetime format [offset]}}`

### `{{$randomInt}}`

Generates a random integer within a specified range (inclusive).

**Format**: `{{$randomInt min max}}`

**Usage:**
```http
POST https://api.example.com/test
Content-Type: application/json

{
  "userId": {{$randomInt 1000 9999}},
  "priority": {{$randomInt 1 5}},
  "port": {{$randomInt 8000 9000}}
}
```

**Note**: Both min and max are inclusive. `{{$randomInt 1 5}}` can return 1, 2, 3, 4, or 5.

### `{{$processEnv}}`

Reads values from process environment variables.

**Required Variable (errors if not set):**
```http
GET https://api.example.com/secure
Authorization: Bearer {{$processEnv API_TOKEN}}
```

**Optional Variable (returns empty string if not set):**
```http
GET https://api.example.com/data
X-Debug-Mode: {{$processEnv %DEBUG_MODE}}
```

**Syntax:**
- `{{$processEnv VAR_NAME}}` - Required, throws error if not set
- `{{$processEnv %VAR_NAME}}` - Optional, returns empty string if not set

**Setting environment variables:**

**macOS/Linux:**
```bash
export API_TOKEN="your-token-here"
export DEBUG_MODE="true"
```

**Windows (PowerShell):**
```powershell
$env:API_TOKEN = "your-token-here"
$env:DEBUG_MODE = "true"
```

**Note**: You may need to restart Zed after setting new environment variables.

### `{{$dotenv}}`

Reads values from a `.env` file in your workspace directory (or up to 3 parent directories).

**Usage:**
```http
POST https://api.example.com/login
Content-Type: application/json

{
  "apiKey": "{{$dotenv API_KEY}}",
  "apiSecret": "{{$dotenv API_SECRET}}"
}
```

**Example `.env` file:**
```bash
# .env file in workspace root
API_KEY=dev-api-key-12345
API_SECRET=dev-secret-67890
BASE_URL=http://localhost:3000
AUTH_TOKEN=dev-token-abc
ENVIRONMENT=development
```

**Notes:**
- File is searched starting from workspace root, then up to 3 parent directories
- Format: `KEY=value` (one per line)
- Comments start with `#`
- Quotes are optional but recommended for values with spaces

## Environment Variables

Environment variables are defined in `.http-client-env.json` files and allow you to maintain different configurations for different environments.

### Environment File Location

The extension searches for environment files in this order:

1. `.http-client-env.json` in workspace root
2. `http-client.env.json` in workspace root
3. Same files in up to 3 parent directories

### Basic Example

Create `.http-client-env.json` in your workspace root:

```json
{
  "$shared": {
    "contentType": "application/json",
    "apiVersion": "v1",
    "userAgent": "REST-Client/1.0"
  },
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-api-key-12345",
    "timeout": "30"
  },
  "staging": {
    "baseUrl": "https://staging.api.example.com",
    "apiKey": "staging-api-key-67890",
    "timeout": "60"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "{{$processEnv PROD_API_KEY}}",
    "timeout": "120"
  },
  "active": "dev"
}
```

### File Structure

- **`$shared`** (optional): Variables available in all environments
- **Environment objects**: Named environments with their specific variables
- **`active`** (optional): Default environment to activate on load

### Using Environment Variables

```http
GET {{baseUrl}}/api/{{apiVersion}}/users
Authorization: Bearer {{apiKey}}
Content-Type: {{contentType}}
```

When `dev` is active:
- `{{baseUrl}}` → `http://localhost:3000`
- `{{apiKey}}` → `dev-api-key-12345`
- `{{timeout}}` → `30`
- `{{contentType}}` → `application/json` (from `$shared`)
- `{{apiVersion}}` → `v1` (from `$shared`)

## File Variables

File variables can be defined directly in `.http` files using the `@variableName = value` syntax.

**Example:**
```http
@baseUrl = https://api.example.com
@apiKey = test-key-12345

###

GET {{baseUrl}}/users
Authorization: Bearer {{apiKey}}
```

**Note**: Environment files are the recommended approach for better organization and environment management. File variables are mainly useful for quick tests or demos.

## Variable Resolution Order

When you use `{{variableName}}` in a request, the system resolves it in this order:

1. **System variables** - If the name starts with `$` (e.g., `{{$guid}}`)
2. **Request variables** - Captured from previous responses (Phase 3 feature)
3. **File-level variables** - Defined with `@variableName = value`
4. **Active environment variables** - From the current environment
5. **Shared variables** - From the `$shared` section
6. **Not found** - Empty string + warning diagnostic

### Example

Given this environment file:

```json
{
  "$shared": {
    "apiVersion": "v1",
    "guid": "shared-guid-123"
  },
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiVersion": "v2"
  }
}
```

With `dev` active:

- `{{$guid}}` → New UUID (system variable has highest priority)
- `{{apiVersion}}` → `v2` (from dev, overrides shared)
- `{{baseUrl}}` → `http://localhost:3000` (from dev)
- `{{guid}}` → `shared-guid-123` (from shared, since not in dev)
- `{{missing}}` → `""` (empty string + warning)

## Environment File Format

### Complete Structure

```json
{
  "$shared": {
    "variable1": "value1",
    "variable2": "value2"
  },
  "environment1": {
    "variable1": "env1-value1",
    "variable3": "env1-value3"
  },
  "environment2": {
    "variable1": "env2-value1",
    "variable4": "env2-value4"
  },
  "active": "environment1"
}
```

### Variable Value Types

**Simple strings:**
```json
{
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key-12345"
  }
}
```

**References to other variables:**
```json
{
  "$shared": {
    "apiPath": "/api/{{apiVersion}}",
    "fullUrl": "{{baseUrl}}{{apiPath}}"
  },
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiVersion": "v1"
  }
}
```

**System variables in environment values:**
```json
{
  "production": {
    "apiKey": "{{$processEnv PROD_API_KEY}}",
    "requestId": "{{$guid}}",
    "timestamp": "{{$timestamp}}"
  }
}
```

### Validation Rules

- Environment names must be valid JSON keys
- Variable names are case-sensitive
- No circular references allowed
- Maximum nesting depth: 10 levels

## Switching Environments

### Using the Slash Command

**List available environments:**
```
/switch-environment
```

Output:
```
Available Environments:

  dev
→ staging (active)
  production

Current active environment: staging
```

**Switch to a specific environment:**
```
/switch-environment production
```

Output:
```
Switched to environment: production
```

### Active Environment Persistence

- The active environment **persists** across requests in the same Zed session
- Opening new `.http` files uses the same active environment
- The active environment is **NOT** saved when Zed is closed
- To set a default, use the `"active"` field in your environment file

### Command Availability

The `/switch-environment` command is available:
- From any file in Zed (not just `.http` files)
- In the command palette
- Via slash command in any editor

## Nested Variables

Variables can reference other variables, creating resolution chains.

### Simple Nesting

```json
{
  "$shared": {
    "apiVersion": "v1",
    "apiPath": "/api/{{apiVersion}}"
  },
  "dev": {
    "host": "localhost:3000",
    "baseUrl": "http://{{host}}"
  }
}
```

**Usage:**
```http
GET {{baseUrl}}{{apiPath}}/users
```

**Resolution:**
- `{{baseUrl}}` → `http://{{host}}` → `http://localhost:3000`
- `{{apiPath}}` → `/api/{{apiVersion}}` → `/api/v1`
- Final URL: `http://localhost:3000/api/v1/users`

### Multi-Level Nesting

```json
{
  "$shared": {
    "protocol": "https",
    "apiVersion": "v2",
    "apiPath": "/api/{{apiVersion}}",
    "usersEndpoint": "{{apiPath}}/users"
  },
  "production": {
    "domain": "api.example.com",
    "baseUrl": "{{protocol}}://{{domain}}",
    "fullUsersUrl": "{{baseUrl}}{{usersEndpoint}}"
  }
}
```

**Usage:**
```http
GET {{fullUsersUrl}}
```

**Resolution chain:**
1. `{{fullUsersUrl}}` → `{{baseUrl}}{{usersEndpoint}}`
2. `{{baseUrl}}` → `{{protocol}}://{{domain}}` → `https://api.example.com`
3. `{{usersEndpoint}}` → `{{apiPath}}/users`
4. `{{apiPath}}` → `/api/{{apiVersion}}` → `/api/v2`
5. Final: `https://api.example.com/api/v2/users`

### System Variables in Nested Contexts

```json
{
  "production": {
    "apiKey": "{{$processEnv PROD_API_KEY}}",
    "authHeader": "Bearer {{apiKey}}"
  }
}
```

**Usage:**
```http
GET https://api.example.com/data
Authorization: {{authHeader}}
```

**Resolution:**
- `{{authHeader}}` → `Bearer {{apiKey}}`
- `{{apiKey}}` → `{{$processEnv PROD_API_KEY}}` → (reads from environment)
- Final: `Authorization: Bearer your-prod-key-value`

## Security Best Practices

### ✅ DO: Use $processEnv for Secrets

**Good:**
```json
{
  "production": {
    "apiKey": "{{$processEnv PROD_API_KEY}}",
    "dbPassword": "{{$processEnv DB_PASSWORD}}",
    "webhookSecret": "{{$processEnv WEBHOOK_SECRET}}"
  }
}
```

**Why:** Secrets stay out of version control and are read from secure environment variables.

### ✅ DO: Use $dotenv for Development

**Good:**
```json
{
  "dev": {
    "apiKey": "{{$dotenv DEV_API_KEY}}",
    "dbUrl": "{{$dotenv DATABASE_URL}}"
  }
}
```

**With `.env` file:**
```bash
# .env (add to .gitignore!)
DEV_API_KEY=dev-key-safe-locally
DATABASE_URL=postgresql://localhost/mydb
```

### ✅ DO: Add Environment Files to .gitignore (When Needed)

If you need local overrides or have secrets in your environment file:

```gitignore
# .gitignore
.http-client-env.local.json
.env
.env.local
```

**Commit a template instead:**
```json
{
  "$shared": {
    "apiVersion": "v1"
  },
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key-safe-to-commit"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "{{$processEnv PROD_API_KEY}}"
  }
}
```

### ❌ DON'T: Hardcode Secrets

**Bad:**
```json
{
  "production": {
    "apiKey": "sk_live_1234567890abcdef",
    "dbPassword": "super-secret-password-123"
  }
}
```

**Why:** These will be committed to version control and exposed.

### ❌ DON'T: Store Secrets in .http Files

**Bad:**
```http
POST https://api.example.com/data
Authorization: Bearer sk_live_hardcoded_secret
```

**Good:**
```http
POST https://api.example.com/data
Authorization: Bearer {{apiKey}}
```

### Security Checklist

- [ ] Production secrets use `{{$processEnv}}`
- [ ] `.env` files are in `.gitignore`
- [ ] No hardcoded API keys in `.http` files
- [ ] Environment file template is safe to commit
- [ ] Team members know how to set required environment variables

## Examples

### Example 1: Multi-Environment API Testing

**`.http-client-env.json`:**
```json
{
  "$shared": {
    "contentType": "application/json",
    "apiVersion": "v1"
  },
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key-12345",
    "debugMode": "true"
  },
  "staging": {
    "baseUrl": "https://staging.api.example.com",
    "apiKey": "{{$dotenv STAGING_API_KEY}}",
    "debugMode": "true"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "{{$processEnv PROD_API_KEY}}",
    "debugMode": "false"
  },
  "active": "dev"
}
```

**`api-tests.http`:**
```http
### Get users
GET {{baseUrl}}/api/{{apiVersion}}/users
Content-Type: {{contentType}}
Authorization: Bearer {{apiKey}}
X-Debug-Mode: {{debugMode}}

### Create user
POST {{baseUrl}}/api/{{apiVersion}}/users
Content-Type: {{contentType}}
Authorization: Bearer {{apiKey}}
X-Request-ID: {{$guid}}

{
  "id": "{{$guid}}",
  "name": "Test User",
  "email": "test{{$randomInt 100 999}}@example.com",
  "createdAt": "{{$datetime iso8601}}"
}
```

### Example 2: Time-Based Queries

```http
### Get analytics for last 30 days
GET {{baseUrl}}/api/analytics
Authorization: Bearer {{apiKey}}
Content-Type: application/json

{
  "startDate": "{{$datetime iso8601 -30 d}}",
  "endDate": "{{$datetime iso8601}}",
  "granularity": "daily"
}

### Get data since yesterday
GET {{baseUrl}}/api/data?since={{$timestamp -1 d}}&until={{$timestamp}}
Authorization: Bearer {{apiKey}}
```

### Example 3: Combining All Variable Types

```http
### Complex request with all variable types
POST {{baseUrl}}/api/{{apiVersion}}/events
Content-Type: {{contentType}}
Authorization: Bearer {{apiKey}}
X-Request-ID: {{$guid}}
X-Timestamp: {{$timestamp}}
X-User: {{$processEnv %USER}}

{
  "eventId": "{{$guid}}",
  "eventType": "user.action",
  "timestamp": {{$timestamp}},
  "scheduledAt": "{{$datetime iso8601 +2 h}}",
  "priority": {{$randomInt 1 5}},
  "environment": "{{baseUrl}}",
  "metadata": {
    "apiVersion": "{{apiVersion}}",
    "debugMode": {{debugMode}},
    "userId": {{$randomInt 1000 9999}},
    "sessionId": "{{$guid}}"
  }
}
```

## Troubleshooting

### Variable Not Resolving

**Problem:** `{{variableName}}` appears as empty string or doesn't resolve

**Solutions:**
1. Check variable name spelling (case-sensitive)
2. Verify variable exists in active environment or `$shared`
3. Ensure environment file is valid JSON
4. Check that an environment is active: `/switch-environment`
5. Look for warning diagnostics in the editor

### System Variable Not Working

**Problem:** `{{$timestamp}}` or other system variables don't work

**Checklist:**
- ✓ Variable name starts with `$`
- ✓ Syntax is correct (e.g., `{{$timestamp}}` not `{{ $timestamp }}`)
- ✓ For `$processEnv`, environment variable is set
- ✓ For `$dotenv`, `.env` file exists and contains the variable

### Environment Not Switching

**Problem:** Variables still resolve to old environment values

**Solutions:**
1. Run `/switch-environment` again to confirm switch
2. Check for typos in environment name
3. Verify environment exists in `.http-client-env.json`
4. Restart Zed if environment file was just created

### $processEnv Returns Empty

**Problem:** `{{$processEnv VAR}}` returns empty or causes error

**Solutions:**

**Check if variable is set:**
```bash
# macOS/Linux
echo $VAR

# Windows PowerShell
echo $env:VAR
```

**Set the variable:**
```bash
# macOS/Linux
export VAR=value

# Windows PowerShell
$env:VAR = "value"
```

**Restart Zed** after setting new environment variables.

**Use optional syntax** if variable might not exist:
```http
X-Debug: {{$processEnv %DEBUG_MODE}}
```

### Circular Reference Error

**Problem:** Error about circular variable references

**Example of circular reference:**
```json
{
  "dev": {
    "var1": "{{var2}}",
    "var2": "{{var1}}"
  }
}
```

**Solution:** Break the circular dependency by using a different variable structure.

### .env File Not Found

**Problem:** `{{$dotenv VAR}}` returns empty

**Solutions:**
1. Create `.env` file in workspace root
2. Ensure file is named exactly `.env` (case-sensitive)
3. Check file format: `KEY=value` (one per line)
4. Verify file is within workspace or up to 3 parent directories

## Migration from VS Code REST Client

If you're migrating from the VS Code REST Client extension, here are the key differences:

### Syntax Compatibility

✅ **Fully Compatible:**
- Variable syntax: `{{variableName}}`
- System variables: `{{$guid}}`, `{{$timestamp}}`, `{{$randomInt}}`, etc.
- Environment file format: `.http-client-env.json`
- Request delimiter: `###`
- Comments: `#` and `//`

⚠️ **Differences:**

| Feature | VS Code REST Client | Zed REST Client |
|---------|---------------------|-----------------|
| Environment switching | Settings UI | `/switch-environment` command |
| File variables | `@var = value` | `@var = value` (supported but environment files preferred) |
| Request chaining | `@name` and `{{name.response.body}}` | Phase 3 feature (coming soon) |

### Migration Steps

1. **Copy environment files** - Your `.http-client-env.json` files work as-is
2. **Copy .http files** - All request syntax is compatible
3. **Switch environments** - Use `/switch-environment` instead of settings UI
4. **Update documentation** - Reference this guide for Zed-specific features

### Notable Enhancements

- **Better autocomplete** - Type `{{` to see all available variables with descriptions
- **Hover tooltips** - Hover over variables to see current values
- **Slash commands** - Quick environment switching from anywhere
- **Session persistence** - Active environment persists across all requests

### Known Limitations (Coming in Phase 3)

- Request chaining with captured variables (use workarounds for now)
- GraphQL variable extraction
- Response assertions

## Additional Resources

- [Complete Examples File](../examples/with-variables.http) - All variable types demonstrated
- [System Variables Examples](../examples/system-variables.http) - System variable reference
- [Environment Variables Examples](../examples/environment-variables.http) - Environment usage
- [Environment Management Guide](./ENVIRONMENTS.md) - Detailed environment documentation
- [Getting Started Guide](./GETTING_STARTED.md) - Quick start tutorial
- [LSP Features Guide](./LSP_FEATURES.md) - Autocomplete and hover documentation

## Quick Reference

### System Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `{{$guid}}` | New UUID v4 | `550e8400-e29b-41d4-a716-446655440000` |
| `{{$timestamp}}` | Unix timestamp (seconds) | `1700567890` |
| `{{$timestamp -1 d}}` | Timestamp with offset | `1700481490` |
| `{{$datetime iso8601}}` | ISO 8601 datetime | `2025-11-21T12:00:00.000Z` |
| `{{$datetime rfc1123}}` | RFC 1123 datetime | `Mon, 21 Nov 2025 12:00:00 GMT` |
| `{{$randomInt 1 100}}` | Random integer | `42` |
| `{{$processEnv VAR}}` | Process env (required) | Value of `$VAR` |
| `{{$processEnv %VAR}}` | Process env (optional) | Value or empty string |
| `{{$dotenv VAR}}` | From .env file | Value from `.env` |

### Timestamp Offset Units

| Unit | Description | Example |
|------|-------------|---------|
| `s` | Seconds | `{{$timestamp -3600 s}}` |
| `m` | Minutes | `{{$timestamp -30 m}}` |
| `h` | Hours | `{{$timestamp +2 h}}` |
| `d` | Days | `{{$timestamp -7 d}}` |

### Resolution Priority

1. System variables (`{{$...}}`)
2. Request variables (Phase 3)
3. File variables
4. Environment variables
5. Shared variables
6. Not found (empty + warning)

### Common Commands

- `/switch-environment` - List environments
- `/switch-environment dev` - Switch to dev
- Hover over `{{var}}` - See current value
- Type `{{` - Autocomplete variables