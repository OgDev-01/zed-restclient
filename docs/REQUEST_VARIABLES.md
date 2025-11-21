# Request Variables and Response Capture

This document explains how to capture values from HTTP responses and use them in subsequent requests, enabling powerful request chaining workflows.

## Overview

Request variables allow you to extract data from one HTTP response and use it in later requests within the same `.http` file. This is essential for:

- **Authentication flows**: Login to get a token, then use it in authenticated requests
- **Resource creation**: Create a resource, capture its ID, then fetch or update it
- **Data pipelines**: Chain multiple API calls where each depends on the previous response
- **Testing workflows**: Automate complex multi-step API testing scenarios

## Syntax

Capture directives are written as comments immediately after a request:

```http
POST https://api.example.com/auth/login
Content-Type: application/json

{"username": "admin", "password": "secret"}

# @capture variableName = extractionPath
```

The captured variable can then be used in subsequent requests:

```http
GET https://api.example.com/protected/resource
Authorization: Bearer {{variableName}}
```

## Extraction Methods

### 1. JSONPath (for JSON responses)

Use JSONPath syntax to extract values from JSON responses.

**Basic field access:**
```http
# @capture token = $.access_token
# @capture userId = $.user.id
# @capture userName = $.user.name
```

**Nested object access:**
```http
# @capture city = $.user.address.city
# @capture companyName = $.user.company.name
```

**Array index access:**
```http
# @capture firstId = $.items[0].id
# @capture secondName = $.items[1].name
```

**Root access:**
```http
# @capture fullResponse = $
```

### 2. Header Extraction

Extract header values using the `headers.` prefix:

```http
# @capture authToken = headers.Authorization
# @capture sessionId = headers.X-Session-Id
# @capture contentType = headers.Content-Type
```

Header names are case-insensitive.

### 3. XPath (for XML responses)

**Note:** XPath support is planned but not yet implemented. Use JSONPath for JSON responses.

## Data Type Handling

Captured values are always stored as strings:

- **Strings**: Extracted without quotes
- **Numbers**: Converted to string representation
- **Booleans**: `"true"` or `"false"`
- **Null**: `"null"`
- **Objects/Arrays**: Serialized as JSON strings

## Examples

### Example 1: Authentication Flow

```http
### Step 1: Login and capture token
POST https://api.example.com/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "secret"
}

# @capture accessToken = $.access_token
# @capture refreshToken = $.refresh_token

### Step 2: Use token to access protected resource
GET https://api.example.com/users/me
Authorization: Bearer {{accessToken}}

### Step 3: Refresh token when needed
POST https://api.example.com/auth/refresh
Content-Type: application/json

{
  "refresh_token": "{{refreshToken}}"
}
```

### Example 2: CRUD Operations

```http
### Create a new user
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Alice Johnson",
  "email": "alice@example.com"
}

# @capture userId = $.id
# @capture userEmail = $.email

### Fetch the created user
GET https://api.example.com/users/{{userId}}

### Update the user
PUT https://api.example.com/users/{{userId}}
Content-Type: application/json

{
  "name": "Alice Smith",
  "email": "{{userEmail}}"
}

### Delete the user
DELETE https://api.example.com/users/{{userId}}
```

### Example 3: Nested Data Extraction

```http
### Get user with nested data
GET https://api.example.com/users/1

# Response:
# {
#   "id": 1,
#   "name": "John Doe",
#   "address": {
#     "city": "New York",
#     "geo": {
#       "lat": "40.7128",
#       "lng": "-74.0060"
#     }
#   },
#   "company": {
#     "name": "Acme Corp"
#   }
# }

# @capture userId = $.id
# @capture userName = $.name
# @capture userCity = $.address.city
# @capture userLat = $.address.geo.lat
# @capture userLng = $.address.geo.lng
# @capture companyName = $.company.name

### Use the captured data
POST https://api.example.com/locations
Content-Type: application/json

{
  "user_id": {{userId}},
  "name": "{{userName}}'s Location",
  "city": "{{userCity}}",
  "coordinates": {
    "lat": {{userLat}},
    "lng": {{userLng}}
  },
  "company": "{{companyName}}"
}
```

### Example 4: Array Processing

```http
### Get a list of items
GET https://api.example.com/products

# Response:
# [
#   {"id": 101, "name": "Widget", "price": 29.99},
#   {"id": 102, "name": "Gadget", "price": 49.99},
#   {"id": 103, "name": "Doohickey", "price": 19.99}
# ]

# @capture firstProductId = $[0].id
# @capture firstProductName = $[0].name
# @capture secondProductPrice = $[1].price

### Get details of the first product
GET https://api.example.com/products/{{firstProductId}}

### Add to cart
POST https://api.example.com/cart
Content-Type: application/json

{
  "product_id": {{firstProductId}},
  "product_name": "{{firstProductName}}",
  "quantity": 1
}
```

### Example 5: Multi-Step Workflow

```http
### Step 1: Create a project
POST https://api.example.com/projects
Content-Type: application/json

{
  "name": "New Project",
  "description": "A test project"
}

# @capture projectId = $.id

### Step 2: Add a task to the project
POST https://api.example.com/projects/{{projectId}}/tasks
Content-Type: application/json

{
  "title": "First Task",
  "description": "Initial task for the project"
}

# @capture taskId = $.id

### Step 3: Assign the task
POST https://api.example.com/tasks/{{taskId}}/assign
Content-Type: application/json

{
  "user_id": 123
}

### Step 4: Update task status
PATCH https://api.example.com/tasks/{{taskId}}
Content-Type: application/json

{
  "status": "in_progress"
}

### Step 5: Get project summary
GET https://api.example.com/projects/{{projectId}}/summary
```

## Variable Scoping

### Scope Rules

1. **File-scoped**: Variables are available only within the same `.http` file
2. **Sequential**: Variables must be defined before they can be used
3. **Below-only**: Variables are only available to requests that appear after the capture directive
4. **Persistence**: Variables persist until the file is edited or reloaded

### Example of Variable Scope

```http
### This will fail - token not yet defined
GET https://api.example.com/data
Authorization: Bearer {{token}}

### Define the token
POST https://api.example.com/auth/login
Content-Type: application/json

{"username": "user", "password": "pass"}

# @capture token = $.access_token

### This works - token is now defined
GET https://api.example.com/data
Authorization: Bearer {{token}}

### This also works - token is still available
GET https://api.example.com/more-data
Authorization: Bearer {{token}}
```

## Variable Priority

When a variable name conflicts with other variable sources, the resolution order is:

1. **System variables** (e.g., `{{$guid}}`, `{{$timestamp}}`)
2. **Request variables** (captured from responses) ← This feature
3. **File variables** (defined with `@name = value`)
4. **Environment variables** (from `.http-client-env.json`)
5. **Shared variables** (from environment file)

This means request variables take precedence over environment variables, allowing you to override environment settings dynamically.

## Best Practices

### 1. Use Descriptive Variable Names

```http
# Good
# @capture authToken = $.access_token
# @capture userId = $.user.id

# Avoid
# @capture t = $.access_token
# @capture x = $.user.id
```

### 2. Group Related Captures

```http
### Login and capture all auth data
POST https://api.example.com/auth/login
Content-Type: application/json

{"username": "admin", "password": "secret"}

# @capture accessToken = $.access_token
# @capture refreshToken = $.refresh_token
# @capture expiresIn = $.expires_in
# @capture tokenType = $.token_type
```

### 3. Document Complex Extractions

```http
### Get user profile
GET https://api.example.com/users/1

# Extract nested company information
# @capture companyId = $.company.id
# @capture companyName = $.company.name
# @capture companyDomain = $.company.catchPhrase
```

### 4. Handle Optional Fields Gracefully

If a JSONPath doesn't find a match, an error will occur. Make sure the response structure matches your expectations, or handle errors appropriately.

### 5. Combine with Environment Variables

```http
### Login to the environment-specific endpoint
POST {{baseUrl}}/auth/login
Content-Type: application/json

{
  "username": "{{username}}",
  "password": "{{password}}"
}

# @capture token = $.access_token

### Use both environment and captured variables
GET {{baseUrl}}/protected/data
Authorization: Bearer {{token}}
```

## Limitations

### Current Limitations

1. **XPath not yet implemented**: Only JSONPath and header extraction are currently supported
2. **No filtering**: Advanced JSONPath filtering (e.g., `$.users[?(@.age > 18)]`) is not yet supported
3. **No wildcards**: JSONPath wildcards (e.g., `$.users[*].name`) are not yet supported
4. **Single values**: Each capture extracts a single value; bulk captures are not supported

### Planned Features

- Full JSONPath spec compliance
- XPath support for XML responses
- Advanced filtering and querying
- Conditional captures
- Variable transformations

## Troubleshooting

### Variable Not Found

**Problem**: `Error: Variable 'myVar' not found`

**Solutions**:
- Ensure the capture directive is placed immediately after the request
- Check that the variable is defined before it's used
- Verify the variable name spelling

### JSONPath Extraction Failed

**Problem**: `Error: Field 'token' not found in JSON`

**Solutions**:
- Verify the response structure matches your JSONPath
- Check the response in the Response Pane to see the actual structure
- Use simpler paths first (e.g., `$.token` before `$.data.token`)
- Ensure the response content type is JSON

### Content Type Mismatch

**Problem**: `Error: JSONPath extraction requires JSON content type`

**Solutions**:
- Verify the server is returning `Content-Type: application/json`
- Check if the response is actually JSON
- Use header extraction instead if you need to extract from non-JSON responses

### Invalid JSONPath Syntax

**Problem**: `Error: Invalid JSONPath syntax`

**Solutions**:
- Ensure brackets are balanced: `$.items[0]` not `$.items[0`
- Start with `$` for root reference
- Use dot notation for fields: `$.user.name`
- Use bracket notation for arrays: `$.items[0]`

## Related Documentation

- [Variables Guide](VARIABLES.md) - Overview of all variable types
- [Environment Variables](VARIABLES.md#environment-variables) - Using `.http-client-env.json`
- [System Variables](VARIABLES.md#system-variables) - Dynamic variables like `{{$guid}}`
- [Examples](../examples/request-chaining.http) - Working examples of request chaining

## JSONPath Reference

### Supported Syntax

| Pattern | Description | Example |
|---------|-------------|---------|
| `$` | Root object | `$` |
| `$.field` | Object field | `$.user` |
| `$.field1.field2` | Nested field | `$.user.name` |
| `$[n]` | Array element | `$[0]`, `$[5]` |
| `$.field[n]` | Array in object | `$.users[0]` |
| `$.field[n].field` | Field in array element | `$.users[0].name` |

### Examples

```json
{
  "status": "success",
  "data": {
    "user": {
      "id": 123,
      "name": "Alice",
      "emails": ["alice@example.com", "alice@work.com"]
    },
    "posts": [
      {"id": 1, "title": "First Post"},
      {"id": 2, "title": "Second Post"}
    ]
  }
}
```

Extraction paths:
- `$.status` → `"success"`
- `$.data.user.id` → `"123"`
- `$.data.user.name` → `"Alice"`
- `$.data.user.emails[0]` → `"alice@example.com"`
- `$.data.posts[0].id` → `"1"`
- `$.data.posts[1].title` → `"Second Post"`
