# Getting Started with REST Client for Zed

Welcome to the REST Client extension for Zed! This guide will help you get up and running quickly with testing HTTP APIs directly from your editor.

## Table of Contents

- [What is REST Client?](#what-is-rest-client)
- [Installation](#installation)
- [Creating Your First Request](#creating-your-first-request)
- [Sending Requests](#sending-requests)
- [Understanding Responses](#understanding-responses)
- [Request Formats](#request-formats)
- [Advanced Features](#advanced-features)
- [Troubleshooting](#troubleshooting)
- [Examples](#examples)

## What is REST Client?

REST Client is a Zed extension that allows you to send HTTP requests and view formatted responses without leaving your editor. It's perfect for:

- Testing REST APIs during development
- Debugging API endpoints
- Learning about HTTP and APIs
- Creating shareable API documentation
- Version controlling your API tests

## Installation

### Prerequisites

- [Zed Editor](https://zed.dev/) installed on your system
- Internet connection for downloading the extension

### Installing the Extension

1. **Open Zed Editor**

2. **Open the Extensions Panel**
   - Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux/Windows)
   - Type "extensions" and select "Extensions: Install Extensions"
   - Or use the menu: `Zed` → `Extensions`

3. **Search for REST Client**
   - In the extensions panel, search for "REST Client"
   - Click the "Install" button next to the REST Client extension

4. **Wait for Installation**
   - The extension will download and install automatically
   - You may need to restart Zed after installation

5. **Verify Installation**
   - Create a new file with the `.http` extension
   - You should see syntax highlighting for HTTP methods and URLs

## Creating Your First Request

Let's create a simple GET request to test a public API.

### Step 1: Create a New .http File

1. In Zed, create a new file: `File` → `New File` or press `Cmd+N` (macOS) / `Ctrl+N` (Linux/Windows)
2. Save the file with a `.http` or `.rest` extension (e.g., `my-requests.http`)

### Step 2: Write Your First Request

Type the following into your new file:

```http
# My first request
GET https://httpbin.org/get
```

That's it! You've written your first HTTP request. Let's break down what this means:

- `# My first request` - A comment (lines starting with `#` or `//` are comments)
- `GET` - The HTTP method
- `https://httpbin.org/get` - The URL to send the request to

### Step 3: Add Headers (Optional)

You can add headers to your request by adding them on new lines after the request line:

```http
GET https://httpbin.org/get
Accept: application/json
User-Agent: MyApp/1.0
```

### Step 4: Add a Request Body (for POST, PUT, PATCH)

For requests that need a body, add a blank line after the headers, then add your body content:

```http
POST https://httpbin.org/post
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

## Sending Requests

There are two ways to send a request:

### Method 1: Command Palette

1. Place your cursor anywhere within the request you want to send
2. Open the command palette: `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux/Windows)
3. Type "Send Request" and select `rest-client: send request`
4. The request will execute and the response will appear in a new editor pane

### Method 2: Keyboard Shortcut

1. Place your cursor anywhere within the request
2. Press `Cmd+Enter` (macOS) or `Ctrl+Enter` (Linux/Windows)
3. The response will appear in a new pane

> **Tip**: You don't need to select the entire request. Just place your cursor anywhere in the request block, and the extension will automatically detect which request to send.

## Understanding Responses

When you send a request, the response appears in a new editor buffer with the following information:

### Response Format

```
HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: 315
Date: Mon, 01 Jan 2024 12:00:00 GMT

{
  "status": "success",
  "data": {
    "message": "Hello World"
  }
}
```

### Response Sections

1. **Status Line**: `HTTP/1.1 200 OK`
   - Shows the HTTP version, status code, and status text
   - Common codes: 200 (OK), 201 (Created), 404 (Not Found), 500 (Server Error)

2. **Headers**: The lines between the status and the blank line
   - Contains metadata about the response
   - Important headers: `Content-Type`, `Content-Length`, `Date`, etc.

3. **Body**: Everything after the blank line
   - The actual response data
   - Automatically formatted based on content type (JSON, XML, HTML, etc.)

### Response Metadata

At the bottom of the response, you'll see metadata:

```
# Response Metadata
# Duration: 245ms
# Size: 1.2 KB
# Status: 200 OK
# Content-Type: application/json
```

## Request Formats

### Basic Request Syntax

```http
METHOD URL [HTTP_VERSION]
[Header-Name: Header-Value]
[...]

[Request Body]
```

- **METHOD**: Required. One of GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD
- **URL**: Required. The full URL including protocol (http:// or https://)
- **HTTP_VERSION**: Optional. Defaults to HTTP/1.1
- **Headers**: Optional. One per line in `Name: Value` format
- **Body**: Optional. Separated from headers by a blank line

### Multiple Requests in One File

Separate multiple requests using three hash marks (`###`):

```http
# First request
GET https://api.example.com/users

###

# Second request
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Jane Doe"
}

###

# Third request
GET https://api.example.com/users/123
```

### Supported HTTP Methods

- **GET**: Retrieve a resource
- **POST**: Create a new resource
- **PUT**: Update/replace a resource completely
- **PATCH**: Update a resource partially
- **DELETE**: Remove a resource
- **HEAD**: Get headers only (no body)
- **OPTIONS**: Get supported methods for a resource

### Content Types

The extension automatically detects and formats responses:

- **JSON**: Pretty-printed with syntax highlighting
- **XML**: Formatted with indentation
- **HTML**: Displayed as-is
- **Plain Text**: Displayed as-is
- **Binary**: Shows hex preview for images and binary data

## Advanced Features

### Custom Headers

Add any custom headers you need:

```http
POST https://api.example.com/data
Content-Type: application/json
Authorization: Bearer your-token-here
X-Request-ID: unique-id-123
X-Custom-Header: custom-value

{
  "data": "your data here"
}
```

### Query Parameters

Include query parameters directly in the URL:

```http
GET https://api.example.com/search?q=test&limit=10&offset=0
```

### Different Content Types

#### JSON Body
```http
POST https://api.example.com/json
Content-Type: application/json

{
  "key": "value"
}
```

#### XML Body
```http
POST https://api.example.com/xml
Content-Type: application/xml

<?xml version="1.0"?>
<root>
  <item>value</item>
</root>
```

#### Form Data
```http
POST https://api.example.com/form
Content-Type: application/x-www-form-urlencoded

name=John&email=john@example.com&age=30
```

### Comments

Add comments to document your requests:

```http
# This is a single-line comment
// This is also a comment

# Get user profile
# This endpoint requires authentication
GET https://api.example.com/profile
Authorization: Bearer token123
```

### Authentication Examples

#### Bearer Token
```http
GET https://api.example.com/protected
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### Basic Authentication
```http
GET https://api.example.com/auth
Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
```

#### API Key
```http
GET https://api.example.com/data
X-API-Key: your-api-key-here
```

## Troubleshooting

### Issue: Extension Not Working

**Solution:**
1. Verify the extension is installed: `Cmd+Shift+P` → "Extensions"
2. Check that your file has `.http` or `.rest` extension
3. Restart Zed Editor
4. Try reinstalling the extension

### Issue: "Send Request" Command Not Found

**Solution:**
1. Make sure your file is saved with `.http` or `.rest` extension
2. The command only appears when editing HTTP files
3. Check that the extension is enabled in the Extensions panel

### Issue: Request Timeout

**Solution:**
1. Check your internet connection
2. Verify the URL is correct and the server is accessible
3. The default timeout is 30 seconds; slow servers may time out
4. Try using a different endpoint to test

### Issue: Invalid Request Format Error

**Solution:**
1. Ensure the request line has the format: `METHOD URL`
2. Check that headers are in `Name: Value` format
3. Verify there's a blank line between headers and body
4. Make sure you're using a supported HTTP method

### Issue: Response Not Formatted

**Solution:**
1. The extension auto-detects content type from the `Content-Type` header
2. If the server doesn't set this header, the response shows as plain text
3. JSON responses are automatically pretty-printed
4. XML responses get basic formatting

### Issue: Can't Send Multiple Requests

**Solution:**
1. Separate requests with `###` on its own line
2. Make sure there's at least one blank line before and after `###`
3. Place your cursor in the specific request block you want to send

### Issue: Special Characters in Response

**Solution:**
1. The extension handles UTF-8 encoding automatically
2. Binary responses show a hex preview
3. Very large responses (>1MB) are truncated with a warning

### Issue: CORS Errors

**Solution:**
1. CORS (Cross-Origin Resource Sharing) is a browser restriction
2. This extension runs outside the browser, so CORS doesn't apply
3. If you see CORS-related errors, they're likely from the server itself
4. Contact the API provider to resolve server-side CORS issues

### Getting Help

If you encounter issues not covered here:

1. Check the [examples](../examples/) directory for working request samples
2. Review the main [README](../README.md) for additional documentation
3. Report bugs on the [GitHub repository](https://github.com/yourusername/rest-client)
4. Consult the [Zed documentation](https://zed.dev/docs) for editor-specific questions

## Examples

### Quick Start Example

Try this complete example to test the extension:

```http
# Quick Start Test - httpbin.org provides free HTTP testing endpoints

# 1. Simple GET request
GET https://httpbin.org/get

###

# 2. POST with JSON data
POST https://httpbin.org/post
Content-Type: application/json

{
  "message": "Hello from REST Client!",
  "timestamp": "2024-01-01T12:00:00Z"
}

###

# 3. Request with custom headers
GET https://httpbin.org/headers
Accept: application/json
X-Custom-Header: MyValue
User-Agent: REST-Client/1.0

###

# 4. Testing query parameters
GET https://httpbin.org/get?name=Test&version=1.0
```

### More Examples

Check out the comprehensive examples in the `examples/` directory:

- [`basic-requests.http`](../examples/basic-requests.http) - Basic HTTP operations and features
- [`json-api.http`](../examples/json-api.http) - RESTful API testing with JSONPlaceholder

## Next Steps

Now that you're familiar with the basics:

1. **Explore the Examples**: Open the example files and try sending the requests
2. **Test Your Own APIs**: Create `.http` files for your projects
3. **Share with Your Team**: Commit `.http` files to version control for team collaboration
4. **Learn More**: Check the [USAGE.md](../USAGE.md) file for detailed feature documentation

Happy API testing! 🚀