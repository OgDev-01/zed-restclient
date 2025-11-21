# cURL Import/Export Commands

This document describes how to use the cURL import and export functionality in the REST Client extension for Zed.

## Overview

The REST Client extension provides two slash commands for working with cURL commands:

- **`/paste-curl`** - Convert a cURL command to HTTP request format
- **`/copy-as-curl`** - Convert an HTTP request to a cURL command

These commands make it easy to:
- Import cURL commands from documentation, Stack Overflow, or API examples
- Export your HTTP requests as cURL commands to share with others or use in scripts
- Convert between formats seamlessly

## Commands

### `/paste-curl` - Import cURL Command

Converts a cURL command into a clean HTTP request format that can be saved in your `.http` file.

**Usage:**
```
/paste-curl <curl-command>
```

**Example:**

Input:
```
/paste-curl curl -X POST https://api.github.com/repos/owner/repo/issues -H "Authorization: Bearer token123" -H "Content-Type: application/json" -d '{"title":"Bug report"}'
```

Output:
```http
# Generated from cURL command
POST https://api.github.com/repos/owner/repo/issues
Authorization: Bearer token123
Content-Type: application/json

{"title":"Bug report"}
```

**Features:**
- Auto-detects cURL commands (must start with "curl")
- Parses all common cURL flags (`-X`, `-H`, `-d`, `-u`, etc.)
- Handles multi-line cURL commands with backslash continuations
- Formats output with proper spacing and comments
- Includes source comment indicating it came from cURL

**Supported cURL Flags:**
- `-X`, `--request` - HTTP method
- `-H`, `--header` - Headers
- `-d`, `--data`, `--data-raw`, `--data-binary` - Request body
- `-u`, `--user` - Basic authentication (converted to Authorization header)
- Multi-line commands with `\` line continuations

### `/copy-as-curl` - Export to cURL

Converts an HTTP request to a valid cURL command that can be copied and used in terminal or scripts.

**Usage:**

1. Select an HTTP request in your `.http` file
2. Type `/copy-as-curl`
3. The cURL command will be displayed in the output

**Example:**

Input (selected text):
```http
POST https://api.stripe.com/v1/charges
Authorization: Bearer sk_test_key
Content-Type: application/x-www-form-urlencoded

amount=2000&currency=usd
```

Output:
```bash
curl -X POST 'https://api.stripe.com/v1/charges' \
  -H 'Authorization: Bearer sk_test_key' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'amount=2000&currency=usd'
```

**Features:**
- Generates valid, runnable cURL commands
- Properly escapes shell special characters
- Multi-line formatting with backslashes for readability
- Shows preview (first ~50 characters) for confirmation
- Preserves header order
- Handles all HTTP methods

## Common Use Cases

### 1. Import API Examples from Documentation

Many API documentations provide examples as cURL commands. You can paste them directly:

```
/paste-curl curl https://api.openai.com/v1/chat/completions -H "Authorization: Bearer $OPENAI_API_KEY" -H "Content-Type: application/json" -d '{"model":"gpt-4","messages":[{"role":"user","content":"Hello"}]}'
```

This will convert it to a clean HTTP request you can edit and reuse.

### 2. Share Requests with Teammates

Convert your HTTP requests to cURL for sharing:

1. Write your request in `.http` format
2. Test it until it works
3. Use `/copy-as-curl` to generate a shareable cURL command
4. Share the cURL command in documentation or chat

### 3. Debug Network Requests

Convert browser network inspector cURL exports to HTTP format for easier editing and testing.

### 4. Convert between Formats

Round-trip between formats as needed:
- HTTP → cURL (for scripts)
- cURL → HTTP (for editing)
- HTTP → cURL → HTTP (to validate conversion)

## Tips and Best Practices

### Working with Multi-line cURL Commands

cURL commands with backslash continuations are fully supported:

```
/paste-curl curl -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer token" \
  -d '{"key": "value"}' \
  https://api.example.com/endpoint
```

### Authentication Handling

The `-u` flag is automatically converted to an Authorization header:

Input:
```
/paste-curl curl -u username:password https://api.example.com
```

Output:
```http
# Generated from cURL command
GET https://api.example.com
Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
```

### Error Handling

If the cURL command is invalid, you'll receive a helpful error message:

```
Error: Failed to parse cURL command: Missing URL
```

Common issues:
- Missing URL
- Unbalanced quotes
- Invalid method name
- Empty content

### Variables in URLs

You can use Zed's variable substitution after importing:

1. Import the cURL command
2. Replace hardcoded values with variables: `{{baseUrl}}`
3. Configure your environment

## Examples

### GitHub API Request

```
/paste-curl curl -L -X POST https://api.github.com/repos/octocat/hello-world/issues -H "Accept: application/vnd.github+json" -H "Authorization: Bearer ghp_token" -d '{"title":"Found a bug","body":"Issue description"}'
```

Result:
```http
# Generated from cURL command
POST https://api.github.com/repos/octocat/hello-world/issues
Accept: application/vnd.github+json
Authorization: Bearer ghp_token

{"title":"Found a bug","body":"Issue description"}
```

### Stripe API with Basic Auth

```
/paste-curl curl https://api.stripe.com/v1/charges -u sk_test_key: -d amount=2000 -d currency=usd -d source=tok_visa
```

Result:
```http
# Generated from cURL command
POST https://api.stripe.com/v1/charges
Authorization: Basic c2tfdGVzdF9rZXk6

amount=2000&currency=usd&source=tok_visa
```

### Docker Hub API

```
/paste-curl curl -X GET "https://hub.docker.com/v2/repositories/library/ubuntu/tags?page_size=10" -H "Accept: application/json"
```

Result:
```http
# Generated from cURL command
GET https://hub.docker.com/v2/repositories/library/ubuntu/tags?page_size=10
Accept: application/json
```

## Keyboard Shortcuts

While Zed doesn't currently support custom keyboard shortcuts for slash commands, you can:

1. Type `/paste` or `/copy` and use tab completion
2. Create editor snippets for common patterns
3. Use Zed's command palette to search for slash commands

## Limitations

### Unsupported cURL Flags

Some cURL flags don't translate to HTTP request properties and are ignored:
- `-o`, `--output` - File output
- `-w`, `--write-out` - Output formatting
- `--max-time`, `-m` - Timeout settings
- `--connect-timeout` - Connection timeout
- `-k`, `--insecure` - SSL verification (ignored but parsed)
- `-L`, `--location` - Follow redirects (ignored but parsed)

These flags are silently ignored during parsing.

### Format Differences

- cURL uses shell escaping; HTTP format doesn't need it
- Some automatic cURL behaviors (like Content-Type inference) are preserved
- Cookie handling differs between cURL and HTTP requests

## Troubleshooting

### "Not a cURL command" Error

Make sure your command starts with `curl`:
```
✗ /paste-curl GET https://example.com
✓ /paste-curl curl GET https://example.com
```

### Quote Mismatch Errors

Ensure all quotes are balanced:
```
✗ /paste-curl curl -H "Content-Type: application/json' https://example.com
✓ /paste-curl curl -H "Content-Type: application/json" https://example.com
```

### Empty Output

If `/copy-as-curl` produces unexpected output, verify:
- The HTTP request is valid
- The request has a URL
- Headers are properly formatted

## See Also

- [Code Generation Usage](CODE_GENERATION_USAGE.md) - Generate executable code from requests
- [Environment Variables](../examples/environment-variables.http) - Using variables in requests
- [cURL Examples](../examples/curl-import-export.http) - More cURL import/export examples
- [REST Client Documentation](../README.md) - General extension documentation

## Support

For issues or questions:
- Check the examples in `examples/curl-import-export.http`
- Review the parser/generator tests for supported syntax
- Report issues on the GitHub repository