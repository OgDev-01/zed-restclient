# LSP Features for REST Client

This document describes the Language Server Protocol (LSP) features available for `.http` and `.rest` files in the REST Client extension.

## Overview

The REST Client extension provides intelligent editor assistance through two main LSP features:

1. **Variable Autocompletion** - Suggests available variables as you type
2. **Variable Hover** - Shows variable values and descriptions when hovering over variable references

## Variable Autocompletion

### Trigger

Autocompletion is triggered automatically when you type `{{` in a `.http` file.

### Completion Sources

Completions are provided from multiple sources, in order of priority:

1. **Environment Variables** (from active environment)
2. **Shared Variables** (available across all environments)
3. **File Variables** (defined in the current `.http` file)
4. **System Variables** (built-in dynamic variables)

### Example

When you type `{{` in your request:

```http
GET https://api.example.com/{{
```

You'll see a completion list containing:

#### Environment Variables
- `baseUrl` = `http://localhost:3000`
- `apiKey` = `dev-key-123`

#### System Variables
- `$guid` - generates a new UUID v4
- `$timestamp` - current Unix timestamp (can use offset like -1 d)
- `$datetime` - formatted datetime (requires format: iso8601 or rfc1123)
- `$randomInt` - random integer (requires min and max)
- `$processEnv` - process environment variable
- `$dotenv` - variable from .env file

### Completion Details

Each completion item shows:
- **Label**: The variable name to insert
- **Detail**: The current value or description
- **Insert Text**: The full text to insert (includes closing `}}`)

## Variable Hover

### Usage

Hover your cursor over any variable reference (e.g., `{{baseUrl}}`) to see detailed information about that variable.

### Hover Information

The hover tooltip displays:

#### For Resolved Variables (Environment, Shared, File)

```
Variable: `baseUrl`
Value: `http://localhost:3000`
Source: environment variable (dev)
```

#### For System Variables

```
System Variable: `$guid`
Description: generates a new UUID v4
Example value: `550e8400-e29b-41d4-a716-446655440000`

*Will be resolved at runtime*
```

#### For Undefined Variables

```
Variable: `unknownVar`

⚠️ Undefined variable

This variable is not defined in:
- Request variables
- File variables
- Environment variables
- Shared variables
```

## Variable Types

### 1. System Variables

Built-in variables that generate dynamic values at request execution time.

#### `$guid`
```http
X-Request-ID: {{$guid}}
```
Generates a new UUID v4 each time the request is executed.

#### `$timestamp`
```http
X-Timestamp: {{$timestamp}}
# With offset
X-Past-Time: {{$timestamp -1 d}}
X-Future-Time: {{$timestamp +2 h}}
```
Generates Unix timestamp in seconds. Supports offsets with units: s (seconds), m (minutes), h (hours), d (days).

#### `$datetime`
```http
Date: {{$datetime rfc1123}}
# or
X-ISO-Date: {{$datetime iso8601}}
# With offset
X-Yesterday: {{$datetime iso8601 -1 d}}
```
Generates formatted datetime string. Formats: `rfc1123`, `iso8601`.

#### `$randomInt`
```http
X-Random-ID: {{$randomInt 1 1000}}
```
Generates random integer between min and max (inclusive).

#### `$processEnv`
```http
Authorization: Bearer {{$processEnv API_TOKEN}}
```
Reads from process environment variables.

#### `$dotenv`
```http
Authorization: Bearer {{$dotenv API_KEY}}
```
Reads from `.env` file in workspace.

### 2. Environment Variables

Variables defined in `.http-client-env.json` file:

```json
{
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key-123"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "prod-key-xyz"
  }
}
```

Usage:
```http
GET {{baseUrl}}/users
Authorization: Bearer {{apiKey}}
```

### 3. Shared Variables

Variables available across all environments:

```json
{
  "$shared": {
    "apiVersion": "v1",
    "timeout": "30"
  }
}
```

Usage:
```http
GET https://api.example.com/{{apiVersion}}/users
```

### 4. File Variables

Variables defined within the `.http` file itself (future feature).

## Variable Resolution Precedence

When a variable name exists in multiple sources, resolution follows this priority:

1. **System Variables** (identified by `$` prefix)
2. **Request Variables** (captured from previous responses)
3. **File Variables** (defined in `.http` file)
4. **Environment Variables** (from active environment)
5. **Shared Variables** (fallback)

## Integration Notes

### Current Status

The completion and hover features are implemented as helper functions that can be integrated into a full LSP server:

- `provide_completions(position, document, environments, file_variables)` - Returns completion items
- `provide_hover(position, document, context)` - Returns hover information

### Future Integration

These functions are designed to be called by Zed's LSP integration when:
- User types `{{` → call `provide_completions()`
- User hovers over text → call `provide_hover()`

## Example Workflow

1. **Create environment file** `.http-client-env.json`:
```json
{
  "$shared": {
    "apiVersion": "v1"
  },
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key-123"
  },
  "active": "dev"
}
```

2. **Write HTTP request** in `api.http`:
```http
GET {{baseUrl}}/api/{{apiVersion}}/users
Authorization: Bearer {{apiKey}}
X-Request-ID: {{$guid}}
```

3. **Use LSP features**:
   - Type `{{` → See completions for `baseUrl`, `apiKey`, `apiVersion`, and system variables
   - Hover over `{{baseUrl}}` → See value: `http://localhost:3000`
   - Hover over `{{$guid}}` → See description and example value

4. **Switch environment**:
```
/switch-environment production
```

5. **Hover again**:
   - Hover over `{{baseUrl}}` → Now shows: `https://api.example.com`

## Error Handling

### Undefined Variables

When you hover over an undefined variable, you'll see a warning with suggestions:

```
⚠️ Undefined variable

This variable is not defined in:
- Request variables
- File variables  
- Environment variables
- Shared variables
```

### Invalid System Variable Syntax

Hovering over a system variable with invalid syntax shows an error:

```
❌ Error: datetime requires format argument (rfc1123 or iso8601)
```

## Best Practices

1. **Use descriptive variable names** for better autocomplete discovery
2. **Hover before executing** to verify variable values
3. **Check undefined variables** to catch typos early
4. **Use system variables** for dynamic data instead of hardcoding
5. **Organize variables** by environment for easy switching

## See Also

- [Environment Management](./ENVIRONMENTS.md)
- [Variable Substitution](../README.md#variables)
- [System Variables Reference](../README.md#system-variables)