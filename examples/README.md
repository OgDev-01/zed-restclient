# REST Client Examples

This directory contains comprehensive examples demonstrating all features of the REST Client extension for Zed.

## 📁 Example Files

### Core Examples

#### `basic-requests.http`
Basic HTTP request syntax and features.
- Simple GET/POST/PUT/DELETE requests
- Headers and request bodies
- Multiple requests per file (using `###` delimiter)
- JSON and form data examples

#### `json-api.http`
Working with JSON APIs.
- JSON request bodies
- Automatic Content-Type handling
- JSON response formatting
- Nested JSON structures

### Variable Examples

#### `with-variables.http` ⭐ **START HERE**
**Comprehensive reference** demonstrating ALL variable types in one place:
- System variables: `{{$guid}}`, `{{$timestamp}}`, `{{$datetime}}`, `{{$randomInt}}`
- Environment variables: `{{baseUrl}}`, `{{apiKey}}`, etc.
- Process environment: `{{$processEnv}}` and `{{$dotenv}}`
- Nested variable resolution
- Real-world examples combining all variable types
- Security best practices
- Debugging tips

#### `system-variables.http`
Deep dive into system variables:
- `{{$guid}}` - UUID generation
- `{{$timestamp}}` - Unix timestamps with offsets
- `{{$datetime}}` - Formatted dates (RFC1123, ISO8601)
- `{{$randomInt}}` - Random number generation
- `{{$processEnv}}` - Process environment variables
- `{{$dotenv}}` - .env file variables

#### `environment-variables.http`
Environment-specific configurations:
- Using `.http-client-env.json` variables
- Switching between dev/staging/production
- Shared variables across environments
- Variable precedence demonstration
- Multi-environment best practices

#### `variable-substitution.http`
Advanced variable substitution patterns:
- Nested variable resolution
- Variables in URLs, headers, and bodies
- Escaped braces for literal `{{`
- Variable resolution order demonstration

### Configuration Files

#### `.http-client-env.json`
Multi-environment configuration example:
```json
{
  "$shared": { ... },        // Variables for all environments
  "dev": { ... },           // Development environment
  "staging": { ... },       // Staging environment  
  "production": { ... },    // Production environment (uses $processEnv)
  "active": "dev"           // Default environment
}
```

Demonstrates:
- Shared variables
- Environment-specific overrides
- Security with `{{$processEnv}}` for production secrets
- Common patterns for API configurations

#### `.gitignore.example`
Security best practices for version control:
- What to commit vs ignore
- Protecting secrets in environment files
- Local override patterns
- Recommended `.gitignore` entries

Copy relevant sections to your project's `.gitignore`.

## 🚀 Quick Start

1. **Open any `.http` file** in Zed
2. **Place cursor** in a request block
3. **Send request** using keyboard shortcut or command
4. **View response** in the output pane

### Try Variables

1. Open `with-variables.http`
2. Review the `.http-client-env.json` file
3. Switch environments: `/switch-environment staging`
4. Execute requests to see variables in action

## 🔄 cURL Import/Export

### `curl-import-export.http`
Converting between cURL commands and HTTP requests.

**Features:**
- Import cURL commands from documentation
- Export requests as cURL for CLI usage
- Support for common cURL flags
- Authentication conversion (Basic, Bearer)
- Multi-line cURL with backslash continuations

**Supported cURL Flags:**
- `-X, --request` - HTTP method
- `-H, --header` - Headers
- `-d, --data` - Request body
- `-u, --user` - Basic authentication
- `--compressed, -k, -L, -s, -v` - Ignored flags

**Examples Included:**
- GitHub API requests
- Stripe payment processing
- Slack webhooks
- SendGrid email API
- Docker Hub and NPM registry

**Usage:**
1. **Import**: Copy cURL from docs → Use "Paste cURL" command
2. **Export**: Place cursor in request → Use "Copy as cURL" command
3. **Share**: Generated cURL works in any terminal

**Real-world scenarios:**
```bash
# Copy from browser DevTools
curl -X POST https://api.example.com/data \
  -H "Authorization: Bearer token" \
  -d '{"key":"value"}'

# Paste in Zed, converts to:
POST https://api.example.com/data
Authorization: Bearer token

{"key":"value"}
```

## 📚 Usage Patterns

### Pattern 1: Basic API Testing

```http
GET https://api.example.com/users
Content-Type: application/json
```

See: `basic-requests.http`

### Pattern 2: Environment-Aware Requests

```http
GET {{baseUrl}}/api/{{apiVersion}}/users
Authorization: Bearer {{apiKey}}
```

Setup:
1. Create `.http-client-env.json` with environments
2. Switch environment: `/switch-environment dev`
3. Execute request (uses dev values)

See: `environment-variables.http`, `.http-client-env.json`

### Pattern 3: Dynamic Values

```http
POST {{baseUrl}}/api/users
Content-Type: application/json
X-Request-ID: {{$guid}}

{
  "id": "{{$guid}}",
  "createdAt": "{{$datetime iso8601}}",
  "priority": {{$randomInt 1 10}}
}
```

See: `system-variables.http`, `with-variables.http`

### Pattern 4: Secure Credentials

```http
GET {{baseUrl}}/api/secure
Authorization: Bearer {{$processEnv API_TOKEN}}
```

Setup:
```bash
export API_TOKEN="your-secret-token"
```

See: `with-variables.http` (Section 7), VARIABLES.md

## 🔐 Security Best Practices

### ✅ DO

- **Use variables** for all configurable values
- **Use `{{$processEnv}}`** for production secrets
- **Commit** `.http-client-env.json` with `$processEnv` references
- **Ignore** `.env` and local overrides in `.gitignore`
- **Document** required environment variables in README

### ❌ DON'T

- **Hardcode** API keys or tokens in `.http` files
- **Commit** `.env` files (they contain secrets)
- **Share** personal API credentials
- **Store** production secrets in environment files

### Example: Secure Environment File

```json
{
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "{{$processEnv PROD_API_KEY}}"
  }
}
```

See: `.gitignore.example` for complete patterns

## 🛠️ Environment Management

### Create Environment File

1. Copy `.http-client-env.json` to your project root
2. Customize environments for your API
3. Use `{{$processEnv}}` for secrets

### Switch Environments

```
/switch-environment                  # List environments
/switch-environment dev              # Switch to dev
/switch-environment production       # Switch to production
```

### Environment Variables

The active environment persists across:
- ✅ Multiple requests in the same file
- ✅ Different `.http` files in the same session
- ✅ Until you switch to another environment
- ❌ Does NOT persist when Zed closes

Set default with `"active": "dev"` in environment file.

## 📖 Documentation

For detailed documentation, see the `/docs` directory:

- **[VARIABLES.md](../docs/VARIABLES.md)** - Complete variables reference
- **[ENVIRONMENTS.md](../docs/ENVIRONMENTS.md)** - Environment management guide
- **[GETTING_STARTED.md](../docs/GETTING_STARTED.md)** - Quick start tutorial
- **[LSP_FEATURES.md](../docs/LSP_FEATURES.md)** - Autocomplete and hover features

## 🎯 Learning Path

### Beginner

1. Start with `basic-requests.http` - understand syntax
2. Try `json-api.http` - work with JSON
3. Read `system-variables.http` - learn dynamic variables

### Intermediate

4. Read `.http-client-env.json` - understand environments
5. Try `environment-variables.http` - switch environments
6. Practice switching: `/switch-environment dev`

### Advanced

7. Study `with-variables.http` - comprehensive patterns
8. Review `variable-substitution.http` - advanced nesting
9. Read `.gitignore.example` - security practices
10. Read [VARIABLES.md](../docs/VARIABLES.md) - complete reference

## 💡 Tips

### Autocomplete Variables

Type `{{` to see all available variables with descriptions.

### Hover for Values

Hover over `{{variableName}}` to see its current value.

### Multiple Requests

Use `###` to separate multiple requests in one file:

```http
### First request
GET https://api.example.com/users

### Second request
POST https://api.example.com/users
Content-Type: application/json

{"name": "John"}
```

### Comments

Add comments with `#` or `//`:

```http
# This is a comment
GET https://api.example.com/users

// This is also a comment
```

## 🧪 Testing the Examples

### Test System Variables

```bash
# Open system-variables.http
# Execute any request
# Check response for generated UUIDs, timestamps, etc.
```

### Test Environment Variables

```bash
# Verify environment file exists
ls .http-client-env.json

# List environments
/switch-environment

# Switch to staging
/switch-environment staging

# Open environment-variables.http
# Execute requests (should use staging URLs)
```

### Test Process Environment Variables

```bash
# Set test variable
export API_TOKEN="test-token-12345"

# Open with-variables.http
# Execute Section 1.5 request
# Check that API_TOKEN is included
```

## 🔍 Troubleshooting

### Variables Not Resolving

1. Check spelling (case-sensitive)
2. Verify environment file exists: `.http-client-env.json`
3. Ensure environment is active: `/switch-environment`
4. Look for warning diagnostics in editor

### Environment Not Found

1. Create `.http-client-env.json` in workspace root
2. Verify valid JSON syntax
3. Ensure environment names are defined

### $processEnv Returns Empty

1. Check variable is set: `echo $VAR_NAME`
2. Restart Zed after setting variables
3. Use optional syntax: `{{$processEnv %VAR_NAME}}`

See [VARIABLES.md](../docs/VARIABLES.md) Troubleshooting section for more help.

## 📝 Contributing

To add new examples:

1. Create descriptive `.http` file
2. Add comments explaining the feature
3. Use realistic (but fake) API endpoints
4. Demonstrate best practices
5. Update this README

## 🔗 Related Resources

- [httpbin.org](https://httpbin.org) - HTTP testing service (used in examples)
- [VS Code REST Client](https://marketplace.visualstudio.com/items?itemName=humao.rest-client) - Original inspiration
- [RFC 2616](https://www.ietf.org/rfc/rfc2616.txt) - HTTP/1.1 specification

## ❓ Questions?

- Read [VARIABLES.md](../docs/VARIABLES.md) for complete variable reference
- Read [ENVIRONMENTS.md](../docs/ENVIRONMENTS.md) for environment management
- Check [GETTING_STARTED.md](../docs/GETTING_STARTED.md) for quick start guide
- Review examples in this directory for practical patterns

---

**Happy Testing! 🚀**