# CodeLens Feature Documentation

## Overview

The CodeLens feature provides clickable "Send Request" lenses above each HTTP request in `.http` and `.rest` files. This allows users to execute requests directly from the editor without manually selecting text or using keyboard shortcuts.

## Features

### Automatic Request Detection

CodeLens automatically detects all valid HTTP requests in your file and displays a clickable lens above each one:

```http
GET https://api.example.com/users
```

You'll see: **▶ Send Request** above the GET line.

### Named Requests

Use `@name` comments to give your requests descriptive names that appear in the CodeLens:

```http
# @name GetAllUsers
GET https://api.example.com/users
```

You'll see: **▶ Send Request: GetAllUsers**

### Multiple Requests

CodeLens works seamlessly with multiple requests in the same file:

```http
# @name CreateUser
POST https://api.example.com/users
Content-Type: application/json

{"name": "Alice"}

###

# @name DeleteUser
DELETE https://api.example.com/users/123
```

Each request gets its own CodeLens, even without delimiters between them.

### Section Organization

Organize your requests with section headers (comments starting with `###`):

```http
### Authentication

# @name Login
POST https://api.example.com/auth/login
Content-Type: application/json

{"username": "admin", "password": "secret"}

### User Management

# @name GetCurrentUser
GET https://api.example.com/users/me
Authorization: Bearer {{token}}
```

CodeLens recognizes section headers as comments and creates lenses for actual requests only.

## Supported HTTP Methods

CodeLens detects all standard HTTP methods (case-sensitive, uppercase only):

- `GET`
- `POST`
- `PUT`
- `PATCH`
- `DELETE`
- `HEAD`
- `OPTIONS`
- `CONNECT`
- `TRACE`

**Note:** Lowercase methods (e.g., `get`) are not recognized and will not show a CodeLens.

## CodeLens Behavior

### Placement

CodeLens appears on the same line as the HTTP method:

```http
# Comment above
GET https://api.example.com/users  ← CodeLens appears here
Accept: application/json
```

### Request Delimiters

Use `###` alone on a line to separate requests:

```http
GET https://api.example.com/users/1

###

GET https://api.example.com/users/2
```

**Note:** `### Section Name` is treated as a comment, not a delimiter.

### Name Scope

`@name` annotations apply to the next HTTP request encountered:

```http
# @name GetUser
GET https://api.example.com/users/1  ← This request is named "GetUser"

###

POST https://api.example.com/users   ← This request has no name
```

After a delimiter (`###`), the name is reset.

### Variables

CodeLens works with variable substitution:

```http
@baseUrl = https://api.example.com

###

GET {{baseUrl}}/users
```

The `@baseUrl = ...` line is recognized as a variable assignment (not a request) and doesn't get a CodeLens.

## Usage

### Clicking a CodeLens

1. Click the **▶ Send Request** lens above any request
2. The request executes immediately
3. Response appears in a new buffer/panel

### Keyboard Alternative

While CodeLens provides a convenient click-to-send interface, you can still:
- Use the command palette: "REST Client: Send Request"
- Use keyboard shortcuts (if configured)

## Examples

### Simple API Testing

```http
GET https://api.github.com/users/octocat
```

### Authenticated Requests

```http
# @name GetProfile
GET https://api.example.com/profile
Authorization: Bearer eyJhbGc...
Accept: application/json
```

### Complete Workflow

```http
### Setup

@baseUrl = https://api.example.com
@userId = 123

### Authentication

# @name Login
POST {{baseUrl}}/auth/login
Content-Type: application/json

{
  "username": "testuser",
  "password": "testpass"
}

### User Operations

# @name GetUser
GET {{baseUrl}}/users/{{userId}}
Authorization: Bearer {{authToken}}

###

# @name UpdateUser
PUT {{baseUrl}}/users/{{userId}}
Authorization: Bearer {{authToken}}
Content-Type: application/json

{
  "name": "Updated Name",
  "email": "newemail@example.com"
}

###

# @name DeleteUser
DELETE {{baseUrl}}/users/{{userId}}
Authorization: Bearer {{authToken}}
```

## Integration with Other Features

### With Diagnostics

CodeLens works alongside the diagnostics feature:
- Red squigglies show errors
- Yellow squigglies show warnings
- CodeLens remains clickable even with diagnostics present

### With Variable Resolution

CodeLens respects variable resolution:
- Variables are substituted before sending the request
- Undefined variables are caught by diagnostics
- Environment switching affects all requests

### With History

Requests sent via CodeLens are saved to history:
- View past requests
- Rerun previous requests
- Track API usage

## Technical Details

### Implementation

- **Module:** `src/language_server/codelens.rs`
- **Function:** `provide_code_lens(document: &str) -> Vec<CodeLens>`
- **Integration:** Language server provides CodeLens to Zed editor

### Performance

- CodeLens updates automatically when the document changes
- Scanning is optimized for large files (O(n) where n = number of lines)
- No performance impact on typing or editing

### Validation

CodeLens only appears for valid requests:
- Must start with a recognized HTTP method (uppercase)
- Comments and variable assignments are skipped
- Empty blocks are ignored

## Troubleshooting

### CodeLens Not Appearing

**Problem:** No CodeLens shows above my request.

**Solutions:**
1. Ensure the HTTP method is uppercase (`GET`, not `get`)
2. Check that the line starts with the method (no leading spaces beyond normal indentation)
3. Verify the file extension is `.http` or `.rest`

### Wrong Request Executing

**Problem:** Clicking a CodeLens executes a different request.

**Solution:** This shouldn't happen with the current implementation. Each CodeLens is tied to its specific line. If you encounter this, it's a bug—please report it.

### Name Not Showing

**Problem:** My `@name` annotation doesn't appear in the CodeLens.

**Solutions:**
1. Ensure the `@name` is directly before the request (with only comments/whitespace between)
2. Check the format: `# @name YourNameHere` or `// @name YourNameHere`
3. Make sure there's no `###` delimiter between the `@name` and the request

### Multiple CodeLens Per Request

**Problem:** I see multiple CodeLens for the same request.

**Solution:** This could happen if you have multiple HTTP methods on consecutive lines without separators. Each method gets its own CodeLens. Use `###` to separate distinct requests.

## Future Enhancements

Potential future features (not yet implemented):

- **Send & Copy as cURL**: Second CodeLens to send request and copy cURL command
- **Quick Actions**: Additional lenses for common operations (copy URL, edit variables, etc.)
- **Request Metrics**: Show last execution time or status in the lens
- **Conditional Lenses**: Show/hide based on request type or configuration

## See Also

- [Usage Guide](../USAGE.md) - General REST Client usage
- [Diagnostics](DIAGNOSTICS.md) - Real-time validation
- [Variables](../README.md#variables) - Variable substitution
- [Environments](../README.md#environments) - Environment management