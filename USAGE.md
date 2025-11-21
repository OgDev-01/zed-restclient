# REST Client Extension - Usage Guide

This document explains how to use the REST Client extension in Zed Editor.

## Overview

The REST Client extension allows you to send HTTP requests directly from `.http` or `.rest` files within Zed, similar to tools like REST Client for VS Code or the HTTP Client in IntelliJ IDEA.

## File Format

Create files with `.http` or `.rest` extensions and write HTTP requests using the following syntax:

### Basic Request

```http
GET https://api.example.com/users
```

### Request with Headers

```http
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer your-token-here
```

### Request with Body

```http
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com",
  "age": 30
}
```

### Multiple Requests in One File

Use `###` as a delimiter between requests:

```http
# Get all users
GET https://api.example.com/users

###

# Create a new user
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Jane Smith",
  "email": "jane@example.com"
}

###

# Update a user
PUT https://api.example.com/users/123
Content-Type: application/json

{
  "name": "Jane Smith Updated"
}

###

# Delete a user
DELETE https://api.example.com/users/123
```

## Sending Requests

### Command Palette

1. Open a `.http` or `.rest` file
2. Place your cursor anywhere within the request you want to send
3. Open the command palette (Cmd+Shift+P on macOS, Ctrl+Shift+P on Linux/Windows)
4. Search for "REST Client: Send Request" or similar command
5. Execute the command

### Cursor Position

The extension automatically detects which request block contains your cursor:
- If you have multiple requests separated by `###`, only the request containing the cursor will be executed
- The cursor can be anywhere within the request block (method line, headers, body)

## Response Display

After sending a request, a new editor buffer will open showing the formatted response:

### Response Format

```
HTTP/1.1 200 OK

Headers:
  Content-Type: application/json
  Content-Length: 123
  Date: Thu, 21 Nov 2024 14:00:00 GMT

Duration: 234ms | Size: 1.23 KB | Type: JSON

---

{
  "id": 1,
  "name": "John Doe",
  "email": "john@example.com",
  "created_at": "2024-11-21T14:00:00Z"
}
```

### Content Types

The extension intelligently formats different content types:

- **JSON**: Pretty-printed with proper indentation
- **XML**: Basic indentation (enhanced formatting coming in future updates)
- **HTML**: Displayed as-is
- **Plain Text**: Displayed as-is
- **Binary**: Hex dump preview (first 1KB)
- **Images**: Image metadata and type information

## Supported HTTP Methods

- `GET` - Retrieve resources
- `POST` - Create resources
- `PUT` - Update/replace resources
- `PATCH` - Partial updates
- `DELETE` - Remove resources
- `HEAD` - Retrieve headers only
- `OPTIONS` - Get communication options
- `TRACE` - Debug/diagnostic requests
- `CONNECT` - Establish tunnels

## Request Syntax

### Request Line

```
METHOD URL [HTTP/VERSION]
```

Examples:
```http
GET https://api.example.com/users
GET https://api.example.com/users HTTP/1.1
POST https://api.example.com/users
```

### Headers

One header per line in `Name: Value` format:

```http
Content-Type: application/json
Authorization: Bearer token123
Accept: application/json
User-Agent: REST-Client/1.0
```

### Body

Add a blank line after headers, then include the request body:

```http
POST https://api.example.com/users
Content-Type: application/json

{
  "key": "value"
}
```

### Comments

Use `#` or `//` for comments:

```http
# This is a comment
// This is also a comment

GET https://api.example.com/users
```

## Configuration

### Timeout

The default request timeout is 30 seconds. This can be configured in the extension settings (feature coming in future updates).

### Response Size Limit

Responses larger than 1MB will be truncated with a warning message to prevent performance issues.

## Response Actions

The REST Client provides several actions to interact with response data:

### Save Response

Save response data to a file with automatically suggested filenames:

- **Full Response**: Saves status line, headers, and body
- **Body Only**: Saves only the response body (most common)
- **Headers Only**: Saves status line and headers

**Filename Suggestions:**
- GET request to `/users` → `get-users-response.json`
- POST request to `/posts` → `post-posts-response.json`
- Automatically uses appropriate extension based on content type (json, xml, html, txt, bin)

**Usage in Code:**
```rust
use rest_client::commands::save_response_command;
use rest_client::ui::response_actions::SaveOption;

let result = save_response_command(&response, &request, SaveOption::BodyOnly);
println!("Save to: {:?}", result.suggested_path);
```

### Copy to Clipboard

Copy response data to clipboard for use in other tools:

- **Full Response**: Copies complete response with status and headers
- **Body**: Copies only the response body
- **Headers**: Copies only headers
- **Status Line**: Copies only the HTTP status line

**Usage in Code:**
```rust
use rest_client::commands::copy_response_command;
use rest_client::ui::response_actions::CopyOption;

let result = copy_response_command(&response, CopyOption::Body);
// Content is ready to be copied: result.content
```

### Fold/Unfold Large Responses

For large JSON or XML responses, you can fold sections to make them more manageable:

- **JSON**: Folds large arrays and objects (configurable threshold)
- **XML**: Folds large XML nodes
- **Default Threshold**: 10 lines

**Example:**
```json
// Before folding:
{
  "users": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"},
    {"id": 3, "name": "Charlie"},
    // ... 100 more items
  ]
}

// After folding:
{
  "users": [ ... 103 items folded ... ],
  "total": 103
}
```

**Usage in Code:**
```rust
use rest_client::commands::fold_response_command;

let result = fold_response_command(&response, 10);
println!("Folded {} sections", result.sections_folded);
```

### Toggle Raw View

Switch between formatted (pretty-printed) and raw (exact bytes) view:

- **Formatted View**: Pretty-printed with syntax highlighting
- **Raw View**: Exact bytes received from server without formatting

**Usage in Code:**
```rust
use rest_client::commands::toggle_raw_view_command;

let toggled = toggle_raw_view_command(&response);
// toggled.is_formatted is now opposite of response.is_formatted
```

### Action Menu

When viewing responses, an action menu is automatically displayed showing available actions:

```
╭─────────────────────────────────────────────────────────╮
│               Response Actions Available               │
├─────────────────────────────────────────────────────────┤
│ 💾 Save Response:                                       │
│    • Full Response (status + headers + body)           │
│    • Body Only                                          │
│    • Headers Only                                       │
├─────────────────────────────────────────────────────────┤
│ 📋 Copy to Clipboard:                                   │
│    • Full Response                                      │
│    • Body Only                                          │
│    • Headers Only                                       │
│    • Status Line Only                                   │
├─────────────────────────────────────────────────────────┤
│ 🔄 View Mode: Formatted (toggle to raw)                │
│ 📁 Fold/Unfold: Available for large sections           │
╰─────────────────────────────────────────────────────────╯
```

## Error Handling

The extension provides clear error messages for common issues:

- **No Request Found**: Cursor is not within a valid request block
- **Parse Error**: Invalid request syntax
- **Network Error**: Connection issues, DNS failures
- **Timeout**: Request took longer than the configured timeout
- **Invalid URL**: Malformed URL
- **TLS Error**: SSL/certificate issues

## Examples

### RESTful API Testing

```http
# List all posts
GET https://jsonplaceholder.typicode.com/posts

###

# Get a specific post
GET https://jsonplaceholder.typicode.com/posts/1

###

# Create a new post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "My New Post",
  "body": "This is the post content",
  "userId": 1
}

###

# Update a post
PUT https://jsonplaceholder.typicode.com/posts/1
Content-Type: application/json

{
  "id": 1,
  "title": "Updated Title",
  "body": "Updated content",
  "userId": 1
}

###

# Delete a post
DELETE https://jsonplaceholder.typicode.com/posts/1
```

### Testing with Headers

```http
# Request with custom headers
GET https://httpbin.org/headers
User-Agent: REST-Client/1.0
X-Custom-Header: custom-value
Accept: application/json

###

# Request with authentication
GET https://api.example.com/protected
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

###

# Request with multiple accept types
GET https://api.example.com/data
Accept: application/json, text/plain
Accept-Language: en-US,en;q=0.9
```

## Tips and Best Practices

1. **Organize Requests**: Use comments and the `###` delimiter to organize related requests
2. **Test Incrementally**: Start with simple GET requests, then add complexity
3. **Use Comments**: Document what each request does for future reference
4. **Check Responses**: Always review the response status, headers, and body
5. **File Organization**: Create separate `.http` files for different APIs or features

## Troubleshooting

### Request Not Executing

- Ensure your cursor is within a valid request block
- Check that the file has a `.http` or `.rest` extension
- Verify the request syntax is correct (method, URL on first line)

### Connection Errors

- Check your internet connection
- Verify the URL is correct and accessible
- Check for firewall or proxy issues

### Timeout Issues

- Try increasing the timeout in settings
- Check if the server is responding slowly
- Consider breaking large requests into smaller chunks

## Future Features

The following features are planned for future releases:

- Environment variables and variable substitution (`{{variable}}`)
- Request history and reuse
- Response caching
- Authentication helpers (OAuth, Basic Auth)
- File uploads and multipart requests
- WebSocket support
- GraphQL support
- Advanced XML formatting
- Syntax highlighting for responses
- Split pane view for request and response
- Request collections and organization
- Export/import functionality

## Support

For issues, feature requests, or contributions, please visit the project repository.