# Environment Management Guide

This guide covers how to use environments in the REST Client extension to manage different configurations (dev, staging, production, etc.) for your HTTP requests.

## Table of Contents

1. [Overview](#overview)
2. [Environment File Format](#environment-file-format)
3. [Switching Environments](#switching-environments)
4. [Variable Resolution](#variable-resolution)
5. [Best Practices](#best-practices)
6. [Examples](#examples)
7. [Troubleshooting](#troubleshooting)

## Overview

Environments allow you to define different sets of variables for different contexts (development, staging, production, etc.) and easily switch between them. This is particularly useful when working with APIs that have different base URLs, API keys, or other configuration for different deployment environments.

### Key Features

- **Multiple Environments**: Define as many environments as you need
- **Shared Variables**: Common variables available across all environments
- **Environment-Specific Variables**: Override shared variables per environment
- **Active Environment Persistence**: Selected environment persists across requests
- **Secure Secrets**: Use `{{$processEnv}}` to reference environment variables for sensitive data

## Environment File Format

Create a `.http-client-env.json` file in your workspace root (or up to 3 parent directories):

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
- **Environment objects** (e.g., `dev`, `staging`, `production`): Named environments with their variables
- **`active`** (optional): Default environment to activate on load

### Supported Filenames

The extension searches for environment files in this order:
1. `.http-client-env.json`
2. `http-client.env.json`

The first file found (searching workspace root, then up to 3 parent directories) is loaded.

## Switching Environments

### Using Slash Command

In any file in Zed, use the `/switch-environment` slash command:

**List available environments:**
```
/switch-environment
```

This displays:
- All available environments
- Currently active environment (marked with →)
- Instructions for switching

**Switch to a specific environment:**
```
/switch-environment production
```

This:
- Activates the specified environment
- Shows confirmation message
- Updates variable resolution for subsequent requests

### Current Environment Indication

When listing environments, the active one is marked:

```
Available Environments:

  dev
→ staging (active)
  production

Current active environment: staging
```

## Variable Resolution

### Resolution Order

When you use a variable like `{{baseUrl}}` in your requests, the system resolves it in this order:

1. **System variables** (if prefixed with `$`, e.g., `{{$guid}}`)
2. **Request variables** (captured from previous responses)
3. **File-level variables** (defined in `.http` file)
4. **Active environment variables** (from current environment)
5. **Shared variables** (from `$shared` section)
6. **Not found** → empty string + warning

### Example Resolution

Given this environment file:

```json
{
  "$shared": {
    "apiVersion": "v2",
    "timeout": "30"
  },
  "dev": {
    "baseUrl": "http://localhost:3000",
    "timeout": "60"
  }
}
```

With `dev` active:
- `{{baseUrl}}` → `http://localhost:3000` (from dev environment)
- `{{timeout}}` → `60` (from dev environment, overrides shared)
- `{{apiVersion}}` → `v2` (from shared, not in dev)
- `{{missing}}` → `""` (not found, produces warning)

### Nested Variables

Variables can reference other variables:

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

Using `{{fullUrl}}` resolves to: `http://localhost:3000/api/v1`

### System Variables in Environments

You can use system variables within environment values:

```json
{
  "production": {
    "apiKey": "{{$processEnv PROD_API_KEY}}",
    "requestId": "{{$guid}}",
    "timestamp": "{{$timestamp}}"
  }
}
```

This is particularly useful for keeping secrets out of version control.

## Best Practices

### 1. Keep Secrets Secure

❌ **Don't** hardcode secrets in environment files:
```json
{
  "production": {
    "apiKey": "sk_live_abc123xyz789"  // Bad!
  }
}
```

✅ **Do** use environment variables:
```json
{
  "production": {
    "apiKey": "{{$processEnv PROD_API_KEY}}"  // Good!
  }
}
```

### 2. Use Shared Variables for Common Values

```json
{
  "$shared": {
    "contentType": "application/json",
    "apiVersion": "v1",
    "userAgent": "MyApp/1.0",
    "acceptLanguage": "en-US"
  },
  "dev": {
    "baseUrl": "http://localhost:3000"
  },
  "production": {
    "baseUrl": "https://api.example.com"
  }
}
```

### 3. Organize by Service

For multiple APIs, create service-specific environments:

```json
{
  "local-auth": {
    "authUrl": "http://localhost:8080",
    "authKey": "local-auth-key"
  },
  "local-data": {
    "dataUrl": "http://localhost:9090",
    "dataKey": "local-data-key"
  },
  "prod-auth": {
    "authUrl": "https://auth.example.com",
    "authKey": "{{$processEnv PROD_AUTH_KEY}}"
  },
  "prod-data": {
    "dataUrl": "https://data.example.com",
    "dataKey": "{{$processEnv PROD_DATA_KEY}}"
  }
}
```

### 4. Version Control

**Commit environment file templates:**
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

**Optionally add to `.gitignore`** if you need local overrides:
```
.http-client-env.local.json
```

### 5. Use Meaningful Names

✅ Good:
- `baseUrl`, `apiKey`, `timeout`, `maxRetries`
- `dev`, `staging`, `production`, `qa`

❌ Avoid:
- `url1`, `key`, `t`, `x`
- `env1`, `test`, `prod1`

## Examples

### Basic Multi-Environment Setup

```json
{
  "$shared": {
    "contentType": "application/json",
    "apiVersion": "v1"
  },
  "dev": {
    "baseUrl": "http://localhost:3000",
    "debugMode": "true"
  },
  "staging": {
    "baseUrl": "https://staging.api.example.com",
    "debugMode": "true"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "debugMode": "false"
  },
  "active": "dev"
}
```

### Using in Requests

```http
### Get users from active environment
GET {{baseUrl}}/api/{{apiVersion}}/users
Content-Type: {{contentType}}
X-Debug-Mode: {{debugMode}}

### Create user
POST {{baseUrl}}/api/{{apiVersion}}/users
Content-Type: {{contentType}}

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### Advanced: Multiple Services

```json
{
  "$shared": {
    "apiVersion": "v2",
    "timeout": "30"
  },
  "dev-auth": {
    "service": "auth",
    "baseUrl": "http://localhost:8080",
    "clientId": "dev-client-id",
    "clientSecret": "dev-client-secret"
  },
  "dev-api": {
    "service": "api",
    "baseUrl": "http://localhost:9090",
    "bearerToken": "dev-token-123"
  },
  "prod-auth": {
    "service": "auth",
    "baseUrl": "https://auth.example.com",
    "clientId": "{{$processEnv AUTH_CLIENT_ID}}",
    "clientSecret": "{{$processEnv AUTH_CLIENT_SECRET}}"
  },
  "prod-api": {
    "service": "api",
    "baseUrl": "https://api.example.com",
    "bearerToken": "{{$processEnv API_BEARER_TOKEN}}"
  }
}
```

## Troubleshooting

### Environment File Not Found

**Problem:** `/switch-environment` shows "No environment configuration found"

**Solutions:**
1. Create `.http-client-env.json` in your workspace root
2. Ensure the file is valid JSON
3. Check that the file is within 3 parent directories of workspace root

### Environment Not Switching

**Problem:** Variables still resolve to old environment values

**Solutions:**
1. Verify the switch was successful (check confirmation message)
2. Ensure variable names match exactly (case-sensitive)
3. Reload the environment file if you made changes after loading

### Variables Not Resolving

**Problem:** `{{variableName}}` shows as empty or doesn't resolve

**Checklist:**
- ✓ Variable exists in active environment or `$shared`
- ✓ Variable name is spelled correctly (case-sensitive)
- ✓ Environment file is valid JSON
- ✓ An environment is active (`/switch-environment` to check)

### Secrets Not Loading

**Problem:** `{{$processEnv VARIABLE}}` returns empty

**Solutions:**
1. Ensure the environment variable is set in your shell:
   ```bash
   echo $VARIABLE
   ```
2. Restart Zed after setting new environment variables
3. Use `export VARIABLE=value` in your shell profile

### Invalid Environment File

**Problem:** JSON parsing errors when loading environments

**Solutions:**
1. Validate JSON syntax with a JSON validator
2. Ensure all strings are quoted
3. Check for trailing commas (not allowed in JSON)
4. Verify environment names are valid identifiers (alphanumeric, underscores, hyphens)

Example of **invalid** JSON:
```json
{
  "$shared": {
    "key": "value",  // ❌ trailing comma
  },
  dev: {  // ❌ unquoted key
    "url": "http://localhost"
  }
}
```

Example of **valid** JSON:
```json
{
  "$shared": {
    "key": "value"
  },
  "dev": {
    "url": "http://localhost"
  }
}
```

## Additional Resources

- [Variable Substitution Examples](../examples/variable-substitution.http)
- [Environment Variables Examples](../examples/environment-variables.http)
- [Getting Started Guide](./GETTING_STARTED.md)
- [Quick Start Guide](./QUICK_START.md)

## Session Persistence

The active environment persists for your entire Zed session. This means:

- ✓ Environment remains active across multiple requests
- ✓ Opening new `.http` files uses the same environment
- ✓ Switching environments affects all subsequent requests
- ✗ Environment selection is **not** persisted when Zed is closed
- ✗ Each workspace can have a different active environment

To set a default environment, use the `"active"` field in your environment file:

```json
{
  "dev": { ... },
  "production": { ... },
  "active": "dev"
}
```

This will automatically activate the specified environment when the file is first loaded.