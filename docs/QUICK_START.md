# Quick Start Guide - Testing the REST Client Extension

This guide will help you quickly build and test the REST Client extension in Zed.

## Prerequisites

Ensure you have:
- Rust toolchain installed
- Zed editor installed
- WASM target added: `rustup target add wasm32-wasip1`

## Build the Extension

```bash
cd rest-client
cargo build --target wasm32-wasip1 --release
```

Expected output:
```
Finished `release` profile [optimized] target(s) in XX.XXs
```

The compiled extension will be at: `target/wasm32-wasip1/release/rest_client.wasm`

## Test File Examples

Create a file called `test.http` with the following examples:

### Example 1: Simple GET Request

```http
### Get a user from GitHub API
GET https://api.github.com/users/octocat
```

### Example 2: JSON POST Request

```http
### Create a test post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "Test Post",
  "body": "This is a test from Zed REST Client",
  "userId": 1
}
```

### Example 3: Request with Headers

```http
### Get with custom headers
GET https://httpbin.org/headers
User-Agent: Zed-REST-Client/1.0
Accept: application/json
X-Custom-Header: test-value
```

### Example 4: PUT Request

```http
### Update a resource
PUT https://jsonplaceholder.typicode.com/posts/1
Content-Type: application/json

{
  "id": 1,
  "title": "Updated Title",
  "body": "Updated content",
  "userId": 1
}
```

### Example 5: DELETE Request

```http
### Delete a resource
DELETE https://jsonplaceholder.typicode.com/posts/1
```

### Example 6: Query Parameters

```http
### Search with query parameters
GET https://api.github.com/search/repositories?q=rust&sort=stars&order=desc
```

## How to Test

### In Zed Editor (Once Extension is Installed)

1. Open the `test.http` file in Zed
2. Place your cursor inside a request block
3. Open the command palette (`Cmd+Shift+P` or `Ctrl+Shift+P`)
4. Search for "Send Request" or the REST Client command
5. Execute the command
6. View the response in a new buffer/panel

### Expected Response Format

```
HTTP/1.1 200 OK
Date: Mon, 01 Jan 2024 12:00:00 GMT
Content-Type: application/json; charset=utf-8
Content-Length: 123

{
  "id": 1,
  "name": "octocat",
  "bio": "GitHub mascot"
}

---
Duration: 234ms
Size: 123 bytes
```

## Verify Build Success

Check that the build completed without errors:

```bash
# Should show no errors
cargo build --target wasm32-wasip1 --release 2>&1 | grep -i error

# Check the output file exists
ls -lh target/wasm32-wasip1/release/rest_client.wasm
```

Expected: A WASM file around 2-4 MB in size.

## Common Testing Scenarios

### 1. Test Success Response (200 OK)

```http
GET https://httpbin.org/status/200
```

**Expected**: Response body from httpbin showing request details.

### 2. Test JSON Formatting

```http
GET https://jsonplaceholder.typicode.com/posts/1
```

**Expected**: Pretty-printed JSON in the response.

### 3. Test Headers

```http
GET https://httpbin.org/headers
Accept: application/json
User-Agent: TestClient/1.0
```

**Expected**: Response showing your custom headers were sent.

### 4. Test POST with Body

```http
POST https://httpbin.org/post
Content-Type: application/json

{
  "test": "data",
  "nested": {
    "value": 123
  }
}
```

**Expected**: Response echoing back your request body.

### 5. Test Different Content Types

```http
### Test XML response
GET https://httpbin.org/xml
```

**Expected**: XML content in response body.

## Known Limitations to Verify

When testing, be aware of these limitations:

### ❌ No Status Codes
- All successful responses show "200 OK" regardless of actual status
- Cannot distinguish between 200, 201, 204, etc.
- **Test**: Try `GET https://httpbin.org/status/404` - will show as success even though it's a 404

### ❌ No Timeout Control
- Cannot configure request timeout
- **Test**: Try a slow endpoint (may hang indefinitely)

### ❌ No TRACE/CONNECT Methods
- **Test**: `TRACE https://example.com` - should show error message

## Troubleshooting

### Build Fails

**Error**: `tokio` features not supported
```bash
# Solution: Make sure Cargo.toml doesn't have tokio or reqwest
grep -E "(tokio|reqwest)" Cargo.toml
# Should return nothing
```

**Error**: Cannot find `wasm32-wasip1` target
```bash
# Solution: Add the target
rustup target add wasm32-wasip1
```

### Extension Doesn't Load in Zed

1. Check WASM file exists: `ls target/wasm32-wasip1/release/rest_client.wasm`
2. Verify file size is reasonable (2-4 MB)
3. Check Zed extension logs for errors
4. Rebuild with `--release` flag

### Requests Don't Execute

1. Verify cursor is inside a request block (between `###` markers)
2. Check the URL is valid (starts with `http://` or `https://`)
3. Check network connectivity
4. Try a known-good endpoint like `https://httpbin.org/get`

## Quick Validation Checklist

Before considering the extension ready:

- [ ] Builds successfully without errors
- [ ] WASM file is generated
- [ ] GET requests work
- [ ] POST requests with body work
- [ ] Headers are sent correctly
- [ ] Response body is displayed
- [ ] Response headers are shown
- [ ] Multiple requests separated by `###` work
- [ ] Error messages are clear for invalid requests

## Development Workflow

For rapid testing during development:

```bash
# 1. Make code changes
# 2. Rebuild
cargo build --target wasm32-wasip1 --release

# 3. If installed in Zed, reload the extension
# (Check Zed documentation for extension reload)

# 4. Test in Zed with test.http file
```

## Example Output

A successful request should produce output similar to:

```
=== Request ===
GET https://api.github.com/users/octocat

=== Response ===
Status: 200 OK
Duration: 245ms
Size: 1234 bytes

Headers:
  content-type: application/json; charset=utf-8
  cache-control: public, max-age=60
  x-github-request-id: ABC123

Body:
{
  "login": "octocat",
  "id": 1,
  "node_id": "MDQ6VXNlcjE=",
  "avatar_url": "https://github.com/images/error/octocat_happy.gif",
  "type": "User",
  "name": "The Octocat",
  "company": "@github",
  "blog": "https://github.blog"
}
```

## Next Steps

Once basic testing is complete:

1. Read [WASM_BUILD.md](WASM_BUILD.md) for detailed build information
2. Review [MIGRATION_SUMMARY.md](MIGRATION_SUMMARY.md) for API details
3. Contribute improvements or report issues
4. Test with your real-world API endpoints

## Support

For issues or questions:
- Check documentation in `docs/` directory
- Review known limitations in `WASM_BUILD.md`
- Verify your request syntax matches the examples above
- Test with public APIs (httpbin.org, jsonplaceholder.typicode.com) first

---

**Happy Testing! 🚀**