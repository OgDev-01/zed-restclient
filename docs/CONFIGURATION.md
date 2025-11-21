# Configuration Guide

The REST Client extension for Zed can be customized through the Zed settings system. This guide covers all available configuration options, their defaults, and examples.

## Quick Start

Add configuration to your Zed `settings.json`:

```json
{
  "rest-client": {
    "timeout": 30000,
    "validateSSL": true,
    "historyLimit": 1000
  }
}
```

## Configuration Options

### Network Settings

#### `timeout`
- **Type:** Integer (milliseconds)
- **Default:** `30000` (30 seconds)
- **Description:** Maximum time to wait for a complete HTTP response, including connection, headers, and body download.
- **Validation:** Must be greater than 0

**Example:**
```json
{
  "rest-client": {
    "timeout": 60000  // 60 seconds
  }
}
```

#### `followRedirects`
- **Type:** Boolean
- **Default:** `true`
- **Description:** Automatically follow HTTP 3xx redirect responses

**Example:**
```json
{
  "rest-client": {
    "followRedirects": false  // Don't follow redirects
  }
}
```

#### `maxRedirects`
- **Type:** Integer
- **Default:** `10`
- **Description:** Maximum number of redirects to follow (only applies when `followRedirects` is `true`)
- **Validation:** Must be >= 0

**Example:**
```json
{
  "rest-client": {
    "followRedirects": true,
    "maxRedirects": 5
  }
}
```

#### `validateSSL`
- **Type:** Boolean
- **Default:** `true`
- **Description:** Validate SSL/TLS certificates for HTTPS requests
- **Security Warning:** Disabling SSL validation can expose you to man-in-the-middle attacks. Only disable for trusted development environments.

**Example:**
```json
{
  "rest-client": {
    "validateSSL": false  // ⚠️ Use with caution!
  }
}
```

### UI Settings

#### `responsePane`
- **Type:** String
- **Default:** `"right"`
- **Options:** `"right"`, `"below"`, `"tab"`
- **Description:** Controls where HTTP responses are displayed

**Example:**
```json
{
  "rest-client": {
    "responsePane": "below"  // Show response below the request
  }
}
```

**Options:**
- `"right"`: Split pane to the right of the request file
- `"below"`: Split pane below the request file
- `"tab"`: Open response in a new editor tab

#### `previewResponseInTab`
- **Type:** Boolean
- **Default:** `false`
- **Description:** When `true`, responses always open in a new tab regardless of `responsePane` setting

**Example:**
```json
{
  "rest-client": {
    "previewResponseInTab": true
  }
}
```

### History Settings

#### `historyLimit`
- **Type:** Integer
- **Default:** `1000`
- **Description:** Maximum number of request/response pairs to keep in history. Older entries are automatically removed.
- **Validation:** Must be greater than 0

**Example:**
```json
{
  "rest-client": {
    "historyLimit": 500  // Keep only 500 most recent requests
  }
}
```

### Environment Settings

#### `environmentFile`
- **Type:** String
- **Default:** `".http-client-env.json"`
- **Description:** Filename to search for when loading environment variables. The extension searches in the workspace root and up to 3 parent directories.

**Example:**
```json
{
  "rest-client": {
    "environmentFile": "custom-env.json"
  }
}
```

**Supported filenames:**
- `.http-client-env.json` (default)
- `http-client.env.json` (also searched automatically)
- Custom filename (configure via this setting)

### Proxy Settings

#### `excludeHostsFromProxy`
- **Type:** Array of strings
- **Default:** `[]`
- **Description:** List of hostnames or patterns to exclude from system proxy settings. Requests to these hosts will bypass the proxy.

**Example:**
```json
{
  "rest-client": {
    "excludeHostsFromProxy": [
      "localhost",
      "127.0.0.1",
      "*.internal.example.com",
      "192.168.1.*"
    ]
  }
}
```

**Pattern matching:**
- Exact hostnames: `"localhost"`, `"api.example.com"`
- Wildcards: `"*.internal.com"`, `"192.168.*"`
- IP addresses: `"127.0.0.1"`, `"10.0.0.*"`

### Default Headers

#### `defaultHeaders`
- **Type:** Object (key-value pairs)
- **Default:** `{ "User-Agent": "Zed-REST-Client/1.0" }`
- **Description:** Headers to include in every request automatically. Request-specific headers override these defaults.

**Example:**
```json
{
  "rest-client": {
    "defaultHeaders": {
      "User-Agent": "MyApp/2.0",
      "Accept": "application/json",
      "X-Client-ID": "zed-rest-client"
    }
  }
}
```

**Notes:**
- Headers in `.http` files override default headers
- Case-insensitive header matching (follows HTTP spec)
- Useful for API keys, client identifiers, or standard accept headers

## Complete Configuration Example

```json
{
  "rest-client": {
    // Network settings
    "timeout": 45000,
    "followRedirects": true,
    "maxRedirects": 5,
    "validateSSL": true,
    
    // UI settings
    "responsePane": "right",
    "previewResponseInTab": false,
    
    // History settings
    "historyLimit": 2000,
    
    // Environment settings
    "environmentFile": ".http-client-env.json",
    
    // Proxy settings
    "excludeHostsFromProxy": [
      "localhost",
      "127.0.0.1",
      "*.local"
    ],
    
    // Default headers
    "defaultHeaders": {
      "User-Agent": "Zed-REST-Client/1.0",
      "Accept": "application/json",
      "Accept-Encoding": "gzip, deflate"
    }
  }
}
```

## Configuration Presets

### Development Environment
```json
{
  "rest-client": {
    "timeout": 10000,
    "validateSSL": false,
    "historyLimit": 500,
    "responsePane": "below",
    "excludeHostsFromProxy": ["localhost", "127.0.0.1", "*.local"]
  }
}
```

### Production Testing
```json
{
  "rest-client": {
    "timeout": 60000,
    "validateSSL": true,
    "followRedirects": true,
    "historyLimit": 2000,
    "defaultHeaders": {
      "User-Agent": "Zed-REST-Client/1.0",
      "Accept": "application/json"
    }
  }
}
```

### API Development
```json
{
  "rest-client": {
    "timeout": 30000,
    "responsePane": "right",
    "historyLimit": 1000,
    "defaultHeaders": {
      "Content-Type": "application/json",
      "Accept": "application/json"
    }
  }
}
```

## Configuration Changes

Configuration changes apply:
- **Immediately** for most settings (timeout, headers, SSL validation)
- **On next request** for execution settings
- **No restart required** - settings are reloaded automatically

## Validation and Error Handling

The extension validates all configuration settings:

- **Invalid values:** Fall back to defaults and show a warning
- **Out-of-range values:** Rejected with an error message
- **Parse errors:** Use default configuration

**Common validation errors:**
- `timeout must be greater than 0`
- `historyLimit must be greater than 0`
- `maxRedirects must be >= 0`

## Troubleshooting

### Settings Not Applied

1. Check JSON syntax in `settings.json`
2. Ensure settings are under `"rest-client"` key
3. Check for validation errors in Zed's console
4. Try resetting to defaults and reapplying

### SSL Certificate Errors

If you're getting SSL errors in development:

```json
{
  "rest-client": {
    "validateSSL": false  // Only for development!
  }
}
```

### Slow Requests

Increase timeout for slow APIs:

```json
{
  "rest-client": {
    "timeout": 120000  // 2 minutes
  }
}
```

### Too Many Redirects

Limit redirect following:

```json
{
  "rest-client": {
    "followRedirects": true,
    "maxRedirects": 3
  }
}
```

## Example Files

The extension provides two example settings files:

- **[example-settings.json](../examples/example-settings.json)** - Valid JSON format for direct use
- **[example-settings.jsonc](../examples/example-settings.jsonc)** - Annotated version with detailed comments

Copy the JSON file contents directly to your Zed `settings.json`, or refer to the JSONC file for detailed explanations of each setting.

## See Also

- [Usage Guide](../USAGE.md) - How to use the REST Client extension
- [Request Variables](REQUEST_VARIABLES.md) - Variable substitution and environments
- [Examples](../examples/) - Sample `.http` files with various configurations