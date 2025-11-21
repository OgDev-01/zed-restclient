# REST Client Troubleshooting Guide

This guide covers common issues and their solutions when using the Zed REST Client extension.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Request Execution Issues](#request-execution-issues)
- [Variable Resolution Issues](#variable-resolution-issues)
- [Environment Issues](#environment-issues)
- [SSL/TLS Issues](#ssl-tls-issues)
- [Authentication Issues](#authentication-issues)
- [Response Issues](#response-issues)
- [Syntax and Parsing Issues](#syntax-and-parsing-issues)
- [Performance Issues](#performance-issues)
- [Configuration Issues](#configuration-issues)

---

## Installation Issues

### Extension Not Loading

**Symptom:** Extension doesn't appear in Zed or `.http` files aren't recognized.

**Solutions:**

1. **Verify Installation:**
   - Open Zed
   - Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux/Windows)
   - Type "zed: extensions"
   - Check if "REST Client" is listed and enabled

2. **Reinstall Extension:**
   ```bash
   # Remove and reinstall
   rm -rf ~/.config/zed/extensions/rest-client
   # Then reinstall from Zed extensions panel
   ```

3. **Check Zed Version:**
   - Ensure you have the latest version of Zed
   - The extension requires Zed version X.X.X or higher

4. **Check Logs:**
   - Open Zed logs: Help → View Logs
   - Look for errors related to "rest-client"

### Syntax Highlighting Not Working

**Symptom:** `.http` files show as plain text without colors.

**Solutions:**

1. **Check File Extension:**
   - Ensure file ends with `.http` or `.rest`
   - Rename if needed: `mv requests.txt requests.http`

2. **Reload Zed:**
   - Close and reopen Zed
   - Or reload window: `Cmd+R` (macOS) or `Ctrl+R` (Linux/Windows)

3. **Verify Language Mode:**
   - Check status bar shows "HTTP" as language
   - If not, click language selector and choose "HTTP"

4. **Clear Cache:**
   ```bash
   rm -rf ~/.config/zed/cache
   # Then restart Zed
   ```

---

## Request Execution Issues

### Request Not Sending

**Symptom:** Clicking "Send Request" does nothing or shows no response.

**Solutions:**

1. **Check Request Format:**
   ```http
   # ✅ Correct
   GET https://api.example.com/users
   
   # ❌ Incorrect - missing protocol
   GET api.example.com/users
   
   # ❌ Incorrect - missing URL
   GET
   ```

2. **Verify URL is Complete:**
   - Must include `http://` or `https://`
   - Must be a valid URL format

3. **Check for Syntax Errors:**
   - Look for red squiggly lines
   - Fix any diagnostics before sending

4. **Check Network Connection:**
   ```bash
   # Test if URL is reachable
   curl https://api.example.com/users
   ```

5. **Review Zed Logs:**
   - Help → View Logs
   - Look for error messages when sending request

### Timeout Errors

**Symptom:** Request times out with "Request timed out" error.

**Solutions:**

1. **Increase Timeout:**
   ```json
   {
     "rest-client": {
       "timeout": 60000  // 60 seconds instead of default 30
     }
   }
   ```

2. **Check Server Response Time:**
   ```bash
   # Test with curl to see actual response time
   curl -w "@-" -o /dev/null -s https://api.example.com/slow-endpoint <<'EOF'
   time_total: %{time_total}
   EOF
   ```

3. **Verify Endpoint is Working:**
   - Try in browser or Postman
   - May be a server-side issue

4. **Check for Network Latency:**
   - VPN may slow requests
   - Try without VPN if possible

### Connection Refused

**Symptom:** Error message: "Connection refused" or "Failed to connect"

**Solutions:**

1. **Verify Server is Running:**
   ```bash
   # For localhost
   curl http://localhost:3000
   
   # Check if port is listening
   lsof -i :3000
   ```

2. **Check Port Number:**
   ```http
   # Make sure port is correct
   GET http://localhost:3000/api/users  # ✅
   GET http://localhost:8080/api/users  # Different port
   ```

3. **Check Firewall:**
   - Firewall may be blocking the connection
   - Temporarily disable to test

4. **Use Correct Protocol:**
   ```http
   # Server may require HTTPS
   GET https://localhost:3000/api/users
   ```

### DNS Resolution Failed

**Symptom:** Error: "DNS lookup failed" or "Host not found"

**Solutions:**

1. **Check Domain Name:**
   ```bash
   # Test DNS resolution
   nslookup api.example.com
   dig api.example.com
   ```

2. **Try IP Address:**
   ```http
   # If DNS fails, try IP directly
   GET http://192.168.1.100:3000/api
   ```

3. **Check Hosts File:**
   ```bash
   # View hosts file
   cat /etc/hosts
   
   # Add entry if needed
   echo "127.0.0.1 api.local" | sudo tee -a /etc/hosts
   ```

4. **Verify VPN/Proxy Settings:**
   - VPN may block certain domains
   - Try without VPN

---

## Variable Resolution Issues

### Variables Not Substituting

**Symptom:** Variables show as `{{variableName}}` in requests instead of being replaced.

**Solutions:**

1. **Check Variable Definition:**
   ```http
   # ✅ Correct - defined before use
   @baseUrl = https://api.example.com
   
   GET {{baseUrl}}/users
   
   # ❌ Incorrect - used before definition
   GET {{baseUrl}}/users
   
   @baseUrl = https://api.example.com
   ```

2. **Verify Variable Name:**
   ```http
   # Variable names are case-sensitive
   @BaseUrl = https://api.example.com
   
   GET {{baseUrl}}/users  # ❌ Won't work - case mismatch
   GET {{BaseUrl}}/users  # ✅ Correct
   ```

3. **Check for Typos:**
   ```http
   @apiKey = secret123
   
   Authorization: Bearer {{apikey}}  # ❌ Wrong case
   Authorization: Bearer {{apiKey}}  # ✅ Correct
   ```

4. **Verify Environment is Active:**
   - Use `/switch-environment` to check
   - Variable may be in different environment

### Undefined Variable Warnings

**Symptom:** Yellow warning: "Variable 'xxx' is undefined"

**Solutions:**

1. **Define Missing Variable:**
   ```http
   # Add variable definition
   @userId = 123
   
   GET {{baseUrl}}/users/{{userId}}
   ```

2. **Check Environment File:**
   ```json
   {
     "development": {
       "baseUrl": "http://localhost:3000",
       "apiKey": "dev-key"  // Add missing variable
     }
   }
   ```

3. **Switch to Correct Environment:**
   ```
   /switch-environment development
   ```

4. **Use System Variable:**
   ```http
   # If it should be a system variable
   GET {{baseUrl}}/users?id={{$guid}}
   ```

### Circular Reference Detected

**Symptom:** Error: "Circular variable reference detected"

**Solutions:**

1. **Fix Circular Reference:**
   ```http
   # ❌ Circular reference
   @varA = {{varB}}
   @varB = {{varA}}
   
   # ✅ Fixed
   @varA = value1
   @varB = {{varA}}-suffix
   ```

2. **Check Nested Variables:**
   ```http
   # ❌ Indirect circular reference
   @base = {{protocol}}://{{domain}}
   @protocol = https
   @domain = {{base}}/api  # References base which references domain
   
   # ✅ Fixed
   @protocol = https
   @domain = api.example.com
   @base = {{protocol}}://{{domain}}
   ```

### Environment Variables Not Loading

**Symptom:** `{{$processEnv VAR}}` or `{{$dotenv VAR}}` returns empty or error.

**Solutions:**

1. **Check Environment Variable is Set:**
   ```bash
   # Verify process environment variable
   echo $API_KEY
   
   # Set if missing
   export API_KEY="your-key-here"
   ```

2. **Verify .env File Exists:**
   ```bash
   # Check .env file location
   ls -la .env
   
   # View contents
   cat .env
   ```

3. **Check .env Format:**
   ```bash
   # ✅ Correct format
   API_KEY=secret123
   BASE_URL=https://api.example.com
   
   # ❌ Incorrect - no spaces around =
   API_KEY = secret123
   ```

4. **Use Optional Syntax:**
   ```http
   # Returns empty if not set (no error)
   Authorization: Bearer {{$processEnv %OPTIONAL_KEY}}
   ```

---

## Environment Issues

### Environment File Not Found

**Symptom:** Warning: "Environment file not found"

**Solutions:**

1. **Create Environment File:**
   ```bash
   # Create in workspace root
   touch .http-client-env.json
   ```

2. **Add Basic Configuration:**
   ```json
   {
     "development": {
       "baseUrl": "http://localhost:3000"
     },
     "production": {
       "baseUrl": "https://api.example.com"
     }
   }
   ```

3. **Check File Location:**
   - Must be in workspace root or parent directories
   - Extension searches up to 3 parent directories

4. **Verify File Name:**
   ```bash
   # Supported names:
   .http-client-env.json  ✅
   http-client.env.json   ✅
   .http-env.json         ❌ Not supported
   ```

### Environment Variables Not Working

**Symptom:** Variables from environment file aren't resolved.

**Solutions:**

1. **Validate JSON:**
   ```bash
   # Check JSON is valid
   jq . .http-client-env.json
   
   # Or use online validator
   ```

2. **Check Environment Structure:**
   ```json
   {
     "development": {           // ✅ Correct
       "baseUrl": "localhost"
     }
   }
   ```
   
   ```json
   {
     "development": [           // ❌ Wrong - should be object
       "baseUrl": "localhost"
     ]
   }
   ```

3. **Ensure Environment is Active:**
   ```
   /switch-environment development
   ```

4. **Check Shared Variables:**
   ```json
   {
     "$shared": {
       "apiVersion": "v1"
     },
     "development": {
       "baseUrl": "http://localhost:3000"
     }
   }
   ```

### Cannot Switch Environment

**Symptom:** `/switch-environment` command doesn't work or shows no environments.

**Solutions:**

1. **Check Environment File Exists:**
   ```bash
   ls -la .http-client-env.json
   ```

2. **Verify JSON is Valid:**
   - Use JSON validator
   - Check for trailing commas

3. **Reload Zed:**
   - Close and reopen Zed
   - Or: `Cmd+R` / `Ctrl+R`

4. **Check Command Syntax:**
   ```
   /switch-environment              # List all
   /switch-environment production   # Switch to specific
   ```

---

## SSL/TLS Issues

### SSL Certificate Verification Failed

**Symptom:** Error: "SSL certificate verification failed"

**Solutions:**

1. **For Development (Self-Signed Certs):**
   ```json
   {
     "rest-client": {
       "validateSSL": false  // ⚠️ Only for development!
     }
   }
   ```

2. **For Production:**
   - Fix the certificate issue on the server
   - Don't disable SSL validation in production

3. **Check Certificate:**
   ```bash
   # View certificate details
   openssl s_client -connect api.example.com:443 -showcerts
   ```

4. **Update CA Certificates:**
   ```bash
   # macOS
   sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain certificate.crt
   
   # Linux
   sudo cp certificate.crt /usr/local/share/ca-certificates/
   sudo update-ca-certificates
   ```

### TLS Handshake Failed

**Symptom:** Error: "TLS handshake failed" or "SSL protocol error"

**Solutions:**

1. **Check TLS Version:**
   - Server may require specific TLS version
   - Try different endpoint to isolate issue

2. **Test with curl:**
   ```bash
   curl -v https://api.example.com
   ```

3. **Check for MITM Proxy:**
   - Corporate proxy may interfere
   - Configure proxy exclusions:
   ```json
   {
     "rest-client": {
       "excludeHostsFromProxy": ["api.example.com"]
     }
   }
   ```

---

## Authentication Issues

### 401 Unauthorized

**Symptom:** Response: "401 Unauthorized"

**Solutions:**

1. **Check Authorization Header:**
   ```http
   # ✅ Correct format
   Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
   
   # ❌ Common mistakes
   Authorization: eyJhbGc...  # Missing "Bearer"
   Authorization: Bearer: token  # Extra colon
   Authorisation: Bearer token  # Wrong spelling
   ```

2. **Verify Token is Valid:**
   ```bash
   # Decode JWT to check expiration
   echo "eyJhbGc..." | cut -d. -f2 | base64 -d | jq
   ```

3. **Check Variable Resolution:**
   ```http
   @token = {{$processEnv API_TOKEN}}
   
   # Verify token is not empty
   # Add debug request:
   POST https://httpbin.org/post
   Content-Type: application/json
   
   {
     "token": "{{token}}"
   }
   ```

4. **Try Hardcoded Token:**
   - Replace variable with actual token to isolate issue

### Basic Auth Not Working

**Symptom:** Basic authentication fails with 401.

**Solutions:**

1. **Check Encoding:**
   ```http
   # Manual base64 encoding
   # "username:password" → base64
   Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
   ```

2. **Verify Credentials:**
   ```bash
   # Test with curl
   curl -u username:password https://api.example.com/protected
   ```

3. **Check for Special Characters:**
   - Special characters in password may need encoding
   - Use `{{$processEnv}}` to avoid issues

### API Key Not Accepted

**Symptom:** API key header present but request fails.

**Solutions:**

1. **Check Header Name:**
   ```http
   # Different APIs use different header names
   X-API-Key: {{apiKey}}       # Common
   API-Key: {{apiKey}}         # Also common
   X-Auth-Token: {{apiKey}}    # Some APIs
   Authorization: ApiKey {{apiKey}}  # Others
   ```

2. **Check Query Parameter:**
   ```http
   # Some APIs use query params instead
   GET https://api.example.com/data?api_key={{apiKey}}
   ```

3. **Verify Key Format:**
   - Check API documentation for exact format
   - May require prefix like "sk_" or "Bearer"

---

## Response Issues

### Response Not Formatted

**Symptom:** JSON/XML shown as plain text without formatting.

**Solutions:**

1. **Check Content-Type Header:**
   - Server must return proper `Content-Type`
   ```http
   # Response should include:
   Content-Type: application/json
   ```

2. **Verify Response is Valid JSON:**
   ```bash
   # Test response with jq
   curl https://api.example.com/data | jq
   ```

3. **Check Response Size:**
   - Very large responses (>10MB) may not format automatically
   - Save to file and format externally

4. **Toggle Raw View:**
   - Command: "rest-client: toggle raw"
   - See if raw content is valid

### Response Body Empty

**Symptom:** Status 200 OK but no response body shown.

**Solutions:**

1. **Check HTTP Method:**
   ```http
   # HEAD requests don't return body
   HEAD https://api.example.com/users  # No body expected
   GET https://api.example.com/users   # Body expected
   ```

2. **Check Status Code:**
   - 204 No Content: No body expected
   - 304 Not Modified: No body expected

3. **Verify Server Response:**
   ```bash
   curl -v https://api.example.com/endpoint
   ```

4. **Check Content-Length:**
   - Response header should show content length
   - May be server issue if 0

### Large Response Performance

**Symptom:** Zed freezes or slows down with large responses.

**Solutions:**

1. **Save to File Instead:**
   ```http
   # Use save response action
   # Command: "rest-client: save response"
   ```

2. **Limit Response Size:**
   - Use pagination in API
   - Request smaller datasets

3. **Increase Response Limit:**
   - Check configuration for size limits

4. **Use Streaming:**
   - For very large downloads, use curl or wget

### Binary Responses Not Displaying

**Symptom:** Images or PDFs don't show in response pane.

**Solutions:**

1. **Save Binary Responses:**
   - Command: "rest-client: save response"
   - Save as appropriate file type

2. **Check Content-Type:**
   ```http
   # Response should indicate binary
   Content-Type: image/png
   Content-Type: application/pdf
   ```

3. **Expected Behavior:**
   - Binary content shows hex preview
   - Full binary saved to file

---

## Syntax and Parsing Issues

### Request Not Parsed Correctly

**Symptom:** Request format errors or incorrect parsing.

**Solutions:**

1. **Check Request Separator:**
   ```http
   # ✅ Correct - three or more #
   ###
   GET https://api.example.com
   
   # ❌ Incorrect - only two #
   ##
   GET https://api.example.com
   ```

2. **Verify Header Format:**
   ```http
   # ✅ Correct
   Content-Type: application/json
   
   # ❌ Incorrect - missing colon
   Content-Type application/json
   
   # ❌ Incorrect - multiple colons
   Content-Type:: application/json
   ```

3. **Check Body Separator:**
   ```http
   POST https://api.example.com
   Content-Type: application/json
                              # ← Blank line required
   {
     "data": "value"
   }
   ```

### JSON Body Validation Errors

**Symptom:** Red squiggles in JSON body or validation errors.

**Solutions:**

1. **Validate JSON:**
   ```bash
   # Use jq to validate
   echo '{"key": "value"}' | jq
   ```

2. **Common JSON Errors:**
   ```json
   {
     "key": "value",  // ❌ Trailing comma
   }
   
   {
     "key": "value"   // ✅ No trailing comma
   }
   ```

3. **Check Quotes:**
   ```json
   {
     "key": 'value'   // ❌ Single quotes
   }
   
   {
     "key": "value"   // ✅ Double quotes
   }
   ```

### Comments Breaking Requests

**Symptom:** Request doesn't work when comments are added.

**Solutions:**

1. **Use Correct Comment Syntax:**
   ```http
   # ✅ Correct - comment before request
   GET https://api.example.com
   
   GET https://api.example.com  # ❌ Inline comments not supported
   ```

2. **Separate Comments from Headers:**
   ```http
   # ✅ Correct
   # This is a comment
   GET https://api.example.com
   Content-Type: application/json
   
   # ❌ Incorrect - comment between method and headers
   GET https://api.example.com
   # This breaks parsing
   Content-Type: application/json
   ```

---

## Performance Issues

### Slow Request Execution

**Symptom:** Requests take longer than expected.

**Solutions:**

1. **Check Network Latency:**
   ```bash
   ping api.example.com
   ```

2. **Review Timing Breakdown:**
   - Hover over response time
   - Identify slow phase (DNS, TCP, TLS, etc.)

3. **Optimize DNS:**
   - Use IP address if DNS is slow
   - Configure local DNS cache

4. **Disable SSL Validation (Dev Only):**
   ```json
   {
     "rest-client": {
       "validateSSL": false  // May speed up TLS
     }
   }
   ```

### Zed Freezing on Large Files

**Symptom:** Editor freezes when opening large `.http` files.

**Solutions:**

1. **Split Large Files:**
   ```bash
   # Split into smaller files
   auth.http
   users.http
   products.http
   ```

2. **Reduce File Size:**
   - Remove old/unused requests
   - Archive to separate directory

3. **Increase Zed Memory:**
   - Check Zed resource usage
   - Close other applications

---

## Configuration Issues

### Settings Not Applied

**Symptom:** Configuration changes in `settings.json` don't take effect.

**Solutions:**

1. **Check JSON Syntax:**
   ```json
   {
     "rest-client": {
       "timeout": 30000,    // ← Comma required
       "validateSSL": true  // ← No comma on last item
     }
   }
   ```

2. **Verify Settings Location:**
   - Global: `~/.config/zed/settings.json`
   - Project: `.zed/settings.json`

3. **Reload Zed:**
   - Settings may require restart
   - `Cmd+R` or `Ctrl+R`

4. **Check Setting Names:**
   ```json
   {
     "rest-client": {
       "timeout": 30000,        // ✅ Correct
       "timeoutMs": 30000       // ❌ Wrong name
     }
   }
   ```

### Invalid Configuration Values

**Symptom:** Error: "Invalid configuration value"

**Solutions:**

1. **Check Value Types:**
   ```json
   {
     "rest-client": {
       "timeout": 30000,        // ✅ Number
       "timeout": "30000",      // ❌ String
       "validateSSL": true,     // ✅ Boolean
       "validateSSL": "true"    // ❌ String
     }
   }
   ```

2. **Verify Value Ranges:**
   ```json
   {
     "rest-client": {
       "timeout": 30000,        // ✅ Valid
       "timeout": -1000,        // ❌ Must be positive
       "maxRedirects": 5,       // ✅ Valid
       "maxRedirects": -1       // ❌ Must be >= 0
     }
   }
   ```

3. **Check Enum Values:**
   ```json
   {
     "rest-client": {
       "responsePane": "right",   // ✅ Valid
       "responsePane": "left"     // ❌ Not a valid option
     }
   }
   ```

---

## Getting More Help

### Enable Debug Logging

```json
{
  "rest-client": {
    "debug": true  // If available
  }
}
```

### Check Zed Logs

1. Help → View Logs
2. Look for "rest-client" entries
3. Note any error messages

### Reproduce with Minimal Example

Create a simple test file:

```http
### Minimal test
GET https://httpbin.org/get
```

If this works, gradually add complexity to isolate the issue.

### Report an Issue

When reporting issues, include:

1. **Zed Version:** Help → About Zed
2. **Extension Version:** Check extensions panel
3. **Operating System:** macOS, Linux, Windows
4. **Minimal Reproduction:** Simplest `.http` file that shows the problem
5. **Error Messages:** From logs or UI
6. **Expected vs Actual:** What should happen vs what does happen

### Community Resources

- GitHub Issues: Report bugs and request features
- Documentation: Check all docs in `docs/` directory
- Examples: Review working examples in `examples/` directory

---

## Quick Reference

### Common Error Messages

| Error | Likely Cause | Solution |
|-------|--------------|----------|
| "Connection refused" | Server not running | Start server or check URL |
| "Timeout" | Slow server/network | Increase timeout setting |
| "SSL verification failed" | Invalid certificate | Disable SSL validation (dev only) |
| "Variable undefined" | Typo or wrong environment | Check variable name and environment |
| "Invalid JSON" | Malformed JSON body | Validate JSON syntax |
| "401 Unauthorized" | Invalid/missing auth | Check Authorization header |
| "DNS lookup failed" | Invalid domain | Check domain name spelling |

### Diagnostic Checklist

When a request fails:

- [ ] Is the URL complete with protocol?
- [ ] Are all variables defined?
- [ ] Is the correct environment active?
- [ ] Is the request syntax valid?
- [ ] Are there any diagnostic warnings?
- [ ] Does the server work with curl?
- [ ] Are authentication headers correct?
- [ ] Is the network connection working?

---

For more information, see:
- [Getting Started Guide](./GETTING_STARTED.md)
- [Features Guide](./FEATURES.md)
- [Configuration Reference](./CONFIGURATION.md)
- [Variables Guide](./VARIABLES.md)
- [Migration Guide](./MIGRATION.md)