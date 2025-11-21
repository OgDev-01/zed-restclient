# REST Client Features Guide

Complete guide to all features available in the Zed REST Client extension.

## Table of Contents

- [Basic HTTP Requests](#basic-http-requests)
- [Variables](#variables)
- [Environments](#environments)
- [Request Chaining](#request-chaining)
- [Authentication](#authentication)
- [Response Handling](#response-handling)
- [Code Generation](#code-generation)
- [GraphQL Support](#graphql-support)
- [cURL Integration](#curl-integration)
- [LSP Features](#lsp-features)
- [History](#history)
- [Advanced Features](#advanced-features)

## Basic HTTP Requests

### Supported HTTP Methods

The extension supports all standard HTTP methods:

- **GET** - Retrieve data
- **POST** - Create resources
- **PUT** - Update resources
- **PATCH** - Partial updates
- **DELETE** - Remove resources
- **OPTIONS** - Check available methods
- **HEAD** - Get headers only

### Request Format

#### Simple Format

For GET requests, you can use just the URL:

```http
https://api.github.com/users/octocat
```

This defaults to a GET request.

#### Full Format

Standard HTTP request format:

```http
METHOD URL [HTTP_VERSION]
Header-Name: Header-Value
Another-Header: Another-Value

Request Body (optional)
```

**Example:**

```http
POST https://api.example.com/users HTTP/1.1
Content-Type: application/json
Accept: application/json
User-Agent: Zed-REST-Client/1.0

{
  "name": "Jane Doe",
  "email": "jane@example.com",
  "role": "developer"
}
```

### Multiple Requests in One File

Separate requests with three or more `#` characters:

```http
### Get all users
GET https://api.example.com/users

### Get specific user
GET https://api.example.com/users/123

### Create new user
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### Comments

Use `#` or `//` for comments:

```http
# This is a comment
// This is also a comment

### User Management API
# TODO: Add pagination support
GET https://api.example.com/users
```

### Request Bodies

#### JSON Body

```http
POST https://api.example.com/data
Content-Type: application/json

{
  "name": "Test",
  "value": 123,
  "nested": {
    "key": "value"
  }
}
```

#### Form Data

```http
POST https://api.example.com/form
Content-Type: application/x-www-form-urlencoded

username=johndoe&password=secret&remember=true
```

#### Plain Text

```http
POST https://api.example.com/text
Content-Type: text/plain

This is plain text content
that can span multiple lines.
```

#### XML Body

```http
POST https://api.example.com/soap
Content-Type: application/xml

<?xml version="1.0"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
  <soap:Body>
    <m:GetUser xmlns:m="http://example.com">
      <m:UserId>123</m:UserId>
    </m:GetUser>
  </soap:Body>
</soap:Envelope>
```

## Variables

Variables allow you to reuse values and make requests dynamic.

### File Variables

Define variables at the top of your file:

```http
@baseUrl = https://api.example.com
@apiVersion = v1
@userId = 12345

### Use variables in requests
GET {{baseUrl}}/{{apiVersion}}/users/{{userId}}
```

### System Variables

#### UUID Generation

```http
POST https://api.example.com/requests
Content-Type: application/json

{
  "requestId": "{{$guid}}",
  "correlationId": "{{$guid}}"
}
```

Each `{{$guid}}` generates a unique UUID.

#### Timestamps

```http
### Current timestamp
POST https://api.example.com/events
Content-Type: application/json

{
  "timestamp": {{$timestamp}},
  "eventType": "user_login"
}

### Timestamp with offset
GET https://api.example.com/logs?from={{$timestamp -1 d}}&to={{$timestamp}}
```

**Offset syntax:**
- `s` - seconds
- `m` - minutes
- `h` - hours
- `d` - days

Examples:
- `{{$timestamp -1 d}}` - Yesterday
- `{{$timestamp +2 h}}` - Two hours from now
- `{{$timestamp -30 m}}` - 30 minutes ago

#### Datetime Formatting

```http
POST https://api.example.com/events
Content-Type: application/json

{
  "timestamp_iso": "{{$datetime iso8601}}",
  "timestamp_rfc": "{{$datetime rfc1123}}"
}
```

**Formats:**
- `iso8601` - ISO 8601 format (2025-01-15T10:30:00Z)
- `rfc1123` - RFC 1123 format (Wed, 15 Jan 2025 10:30:00 GMT)

**With offsets:**

```http
{
  "created_at": "{{$datetime iso8601}}",
  "expires_at": "{{$datetime iso8601 +1 d}}"
}
```

#### Random Values

```http
POST https://api.example.com/test-data
Content-Type: application/json

{
  "randomId": {{$randomInt 1000 9999}},
  "randomScore": {{$randomInt 0 100}}
}
```

#### Environment Variables

Access process environment variables:

```http
### Using process environment
GET https://api.example.com/data
Authorization: Bearer {{$processEnv API_TOKEN}}

### Optional environment variable (returns empty if not set)
X-Custom-Header: {{$processEnv %OPTIONAL_VAR}}
```

#### .env File Variables

Read from `.env` file in workspace:

```http
### Using .env variables
GET {{$dotenv BASE_URL}}/users
Authorization: Bearer {{$dotenv API_KEY}}
```

### Nested Variables

Variables can reference other variables:

```http
@protocol = https
@domain = api.example.com
@baseUrl = {{protocol}}://{{domain}}
@apiUrl = {{baseUrl}}/v1

### Use nested variable
GET {{apiUrl}}/users
```

## Environments

Manage different environments (development, staging, production) with environment files.

### Environment File Format

Create `.http-client-env.json` in your workspace root:

```json
{
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

### Shared Variables

Variables available in all environments:

```json
{
  "$shared": {
    "contentType": "application/json",
    "userAgent": "MyApp/1.0"
  },
  "development": {
    "baseUrl": "http://localhost:3000"
  },
  "production": {
    "baseUrl": "https://api.example.com"
  }
}
```

### Using Environment Variables

```http
### Uses current environment's baseUrl
GET {{baseUrl}}/users
Content-Type: {{contentType}}
User-Agent: {{userAgent}}
```

### Switching Environments

1. Open command palette: `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux/Windows)
2. Type `/switch-environment`
3. Select environment from the list

Or switch directly:
```
/switch-environment production
```

### Variable Resolution Order

1. **Request variables** (captured from responses)
2. **Active environment variables**
3. **Shared variables** (from `$shared`)
4. **File variables** (defined with `@`)
5. **System variables** (like `$guid`)

## Request Chaining

Capture values from responses and use them in subsequent requests.

### Basic Response Capture

```http
### Login to get token
POST {{baseUrl}}/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "{{password}}"
}

# @capture authToken = $.token

### Use captured token
GET {{baseUrl}}/protected-resource
Authorization: Bearer {{authToken}}
```

### JSONPath Syntax

```http
### Create user and capture ID
POST {{baseUrl}}/users
Content-Type: application/json

{
  "name": "Jane Doe",
  "email": "jane@example.com"
}

# @capture userId = $.data.id
# @capture userEmail = $.data.email

### Get the created user
GET {{baseUrl}}/users/{{userId}}
```

**Common JSONPath expressions:**
- `$.id` - Top-level id field
- `$.data.user.id` - Nested field
- `$.items[0].id` - First item in array
- `$.users[*].id` - All user IDs (returns first match)

### Capturing Headers

```http
### Request that returns location header
POST {{baseUrl}}/resources
Content-Type: application/json

{
  "name": "New Resource"
}

# @capture resourceUrl = headers.Location

### Follow the redirect
GET {{resourceUrl}}
```

### Multiple Captures

```http
### Login request
POST {{baseUrl}}/auth/login
Content-Type: application/json

{
  "email": "admin@example.com",
  "password": "{{adminPassword}}"
}

# @capture accessToken = $.access_token
# @capture refreshToken = $.refresh_token
# @capture userId = $.user.id
# @capture expiresIn = $.expires_in

### Use multiple captured values
GET {{baseUrl}}/users/{{userId}}/profile
Authorization: Bearer {{accessToken}}
```

### Complex Workflows

```http
### Step 1: Create a project
POST {{baseUrl}}/projects
Content-Type: application/json

{
  "name": "My Project",
  "description": "Test project"
}

# @capture projectId = $.id

### Step 2: Add team member to project
POST {{baseUrl}}/projects/{{projectId}}/members
Content-Type: application/json

{
  "userId": "{{userId}}",
  "role": "developer"
}

# @capture memberId = $.id

### Step 3: Assign task to member
POST {{baseUrl}}/projects/{{projectId}}/tasks
Content-Type: application/json

{
  "title": "Setup project",
  "assigneeId": "{{memberId}}"
}
```

## Authentication

### Bearer Token

```http
GET https://api.example.com/protected
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

With variables:

```http
GET https://api.example.com/protected
Authorization: Bearer {{token}}
```

### Basic Authentication

Manually encoded:

```http
GET https://api.example.com/protected
Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
```

Using variables:

```http
@username = myuser
@password = mypassword
@basicAuth = {{$base64 {{username}}:{{password}}}}

GET https://api.example.com/protected
Authorization: Basic {{basicAuth}}
```

### API Keys

Header-based:

```http
GET https://api.example.com/data
X-API-Key: {{apiKey}}
```

Query parameter:

```http
GET https://api.example.com/data?api_key={{apiKey}}
```

### Custom Headers

```http
GET https://api.example.com/data
Authorization: Custom {{customToken}}
X-Request-ID: {{$guid}}
X-Client-Version: 1.0.0
```

## Response Handling

### Response Display

Responses show:
- HTTP status code and message
- Response headers
- Formatted response body
- Request timing information
- Response size

### Response Formatting

#### JSON Responses

Automatically pretty-printed:

```json
{
  "id": 123,
  "name": "John Doe",
  "email": "john@example.com",
  "metadata": {
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:00Z"
  }
}
```

#### XML Responses

Formatted with proper indentation:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<user>
  <id>123</id>
  <name>John Doe</name>
  <email>john@example.com</email>
</user>
```

#### HTML Responses

Syntax highlighted for readability.

### Response Actions

#### Save Response

Save response to a file:
1. Send a request
2. Command palette: "rest-client: save response"
3. Choose location and filename

#### Copy Response

Copy parts of the response to clipboard:
1. Command palette: "rest-client: copy response"
2. Choose: Headers, Body, or Full Response

#### Toggle Raw View

Switch between formatted and raw response:
- Command palette: "rest-client: toggle raw"

### Response Timing

Hover over response timing to see breakdown:
- DNS lookup time
- TCP connection time
- TLS handshake time (HTTPS)
- Time to first byte
- Download time

## Code Generation

Generate HTTP client code in multiple languages.

### Supported Languages

- **JavaScript** (fetch, axios)
- **Python** (requests, urllib)
- More languages coming soon

### Generate Code

1. Position cursor in a request
2. Command palette: "rest-client: generate code"
3. Select language and library
4. Generated code is displayed and copied to clipboard

### JavaScript Examples

#### Fetch API

```javascript
// Generated from HTTP request
const url = 'https://api.example.com/users';
const options = {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': 'Bearer your-token'
  },
  body: JSON.stringify({
    name: 'John Doe',
    email: 'john@example.com'
  })
};

fetch(url, options)
  .then(response => response.json())
  .then(data => console.log(data))
  .catch(error => console.error('Error:', error));
```

#### Axios

```javascript
// Generated from HTTP request
const axios = require('axios');

const url = 'https://api.example.com/users';
const data = {
  name: 'John Doe',
  email: 'john@example.com'
};
const config = {
  headers: {
    'Content-Type': 'application/json',
    'Authorization': 'Bearer your-token'
  }
};

axios.post(url, data, config)
  .then(response => console.log(response.data))
  .catch(error => console.error('Error:', error));
```

### Python Examples

#### Requests Library

```python
# Generated from HTTP request
import requests
import json

url = 'https://api.example.com/users'
headers = {
    'Content-Type': 'application/json',
    'Authorization': 'Bearer your-token'
}
data = {
    'name': 'John Doe',
    'email': 'john@example.com'
}

response = requests.post(url, headers=headers, json=data)
print(response.json())
```

## GraphQL Support

Send GraphQL queries and mutations.

### Simple Query

```http
POST https://api.example.com/graphql
Content-Type: application/json

{
  "query": "{ users { id name email } }"
}
```

### Query with Variables

```http
POST https://api.example.com/graphql
Content-Type: application/json

{
  "query": "query GetUser($id: ID!) { user(id: $id) { id name email } }",
  "variables": {
    "id": "{{userId}}"
  }
}
```

### Mutation

```http
POST https://api.example.com/graphql
Content-Type: application/json

{
  "query": "mutation CreateUser($input: UserInput!) { createUser(input: $input) { id name email } }",
  "variables": {
    "input": {
      "name": "Jane Doe",
      "email": "jane@example.com"
    }
  }
}
```

### With Authentication

```http
POST https://api.example.com/graphql
Content-Type: application/json
Authorization: Bearer {{token}}

{
  "query": "{ me { id name email } }"
}
```

## cURL Integration

### Import cURL Commands

Copy a cURL command from documentation:

```bash
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer token123" \
  -d '{"name":"John","email":"john@example.com"}'
```

In Zed:
1. Command palette: "rest-client: paste cURL"
2. The cURL command is automatically converted to HTTP format:

```http
### Imported from cURL
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer token123

{
  "name": "John",
  "email": "john@example.com"
}
```

### Export as cURL

1. Position cursor in a request
2. Command palette: "rest-client: copy as cURL"
3. cURL command is copied to clipboard

Example output:

```bash
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer token123" \
  -d '{"name":"John","email":"john@example.com"}'
```

## LSP Features

Language Server Protocol features for enhanced editing.

### Auto-Completion

#### HTTP Methods

Type and get completions for:
- GET, POST, PUT, PATCH, DELETE
- OPTIONS, HEAD

#### Headers

Auto-complete common headers:
- Content-Type
- Authorization
- Accept
- User-Agent
- And many more...

#### Content Types

Auto-complete MIME types:
- application/json
- application/xml
- text/html
- multipart/form-data
- And more...

#### Variables

Type `{{` to see available variables:
- Environment variables
- File variables
- System variables
- Request variables

### Hover Information

Hover over variables to see:
- Current value
- Source (environment, file, system)
- Type information

### Diagnostics

Real-time error checking:

#### Syntax Errors

```http
POST https://api.example.com/users
Content-Type application/json  # ❌ Missing colon

{
  "name": "John"
```
# ❌ Unclosed JSON brace

#### Undefined Variables

```http
GET {{baseUrl}}/users  # ⚠️ Warning: baseUrl is undefined
```

#### URL Validation

```http
GET not-a-valid-url  # ❌ Invalid URL format
```

#### Header Validation

```http
GET https://api.example.com/data
Conten-Type: application/json  # ⚠️ Did you mean "Content-Type"?
```

### CodeLens

"Send Request" appears above each request:

```http
# Send Request
### Get users
GET https://api.example.com/users
```

Click to execute the request.

## History

Request history is automatically saved.

### What's Saved

- Request method and URL
- Request headers and body
- Response status and headers
- Response body (up to 1MB)
- Timestamp
- Duration

### History Location

History is stored in:
```
~/.config/zed/extensions/rest-client/history.json
```

### Viewing History

History UI is coming soon. Currently, you can view the JSON file directly.

### History Limit

Configure in settings:

```json
{
  "rest-client": {
    "historyLimit": 1000
  }
}
```

## Advanced Features

### Custom Headers for All Requests

Set default headers in configuration:

```json
{
  "rest-client": {
    "defaultHeaders": {
      "User-Agent": "MyApp/1.0",
      "X-Client-Version": "1.0.0"
    }
  }
}
```

### Timeout Configuration

```json
{
  "rest-client": {
    "timeout": 60000  // 60 seconds
  }
}
```

### SSL/TLS Validation

```json
{
  "rest-client": {
    "validateSSL": true
  }
}
```

Set to `false` for self-signed certificates (not recommended for production).

### Follow Redirects

```json
{
  "rest-client": {
    "followRedirects": true,
    "maxRedirects": 5
  }
}
```

### Response Pane Position

```json
{
  "rest-client": {
    "responsePane": "right"  // or "below" or "tab"
  }
}
```

### Proxy Configuration

Exclude specific hosts from proxy:

```json
{
  "rest-client": {
    "excludeHostsFromProxy": [
      "localhost",
      "127.0.0.1",
      "*.internal.company.com"
    ]
  }
}
```

## Best Practices

### 1. Use Environment Files

Keep environment-specific values in `.http-client-env.json`:

```json
{
  "development": {
    "baseUrl": "http://localhost:3000",
    "debugMode": "true"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "debugMode": "false"
  }
}
```

### 2. Secure Secrets

Never commit secrets. Use environment variables:

```json
{
  "production": {
    "apiKey": "{{$processEnv PROD_API_KEY}}"
  }
}
```

### 3. Document Your Requests

Add clear comments:

```http
### User Management API
# Creates a new user account
# 
# Required permissions: admin
# Rate limit: 100 requests/hour
#
# Returns: User object with generated ID

POST {{baseUrl}}/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### 4. Organize Requests

Group related requests in files:

```
api-tests/
├── auth.http          # Authentication endpoints
├── users.http         # User management
├── products.http      # Product endpoints
└── admin.http         # Admin operations
```

### 5. Use Request Chaining

Build workflows:

```http
### 1. Login
POST {{baseUrl}}/auth/login
# ... request details ...
# @capture token = $.token

### 2. Create Resource
POST {{baseUrl}}/resources
Authorization: Bearer {{token}}
# ... request details ...
# @capture resourceId = $.id

### 3. Update Resource
PUT {{baseUrl}}/resources/{{resourceId}}
Authorization: Bearer {{token}}
# ... request details ...
```

## Summary

The Zed REST Client provides a comprehensive set of features for API testing and development:

- ✅ Full HTTP method support
- ✅ Variable system with environments
- ✅ Request chaining with response capture
- ✅ GraphQL support
- ✅ Code generation
- ✅ cURL integration
- ✅ LSP features (auto-complete, diagnostics, hover)
- ✅ Response formatting and actions
- ✅ Authentication support
- ✅ History tracking
- ✅ Highly configurable

For more information, see:
- [Getting Started Guide](./GETTING_STARTED.md)
- [Configuration Reference](./CONFIGURATION.md)
- [Variables Guide](./VARIABLES.md)
- [Troubleshooting](./TROUBLESHOOTING.md)
- [Migration Guide](./MIGRATION.md)