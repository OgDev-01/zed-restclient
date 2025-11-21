# Code Generation

The REST Client extension supports generating executable HTTP client code in multiple programming languages from your `.http` files. This allows you to quickly convert your HTTP requests into runnable code snippets.

## Supported Languages and Libraries

### JavaScript
- **fetch** - Modern browser fetch() API (no dependencies)
- **axios** - Popular promise-based HTTP client (requires: `npm install axios`)

### Python
- **requests** - Simple and elegant HTTP library (requires: `pip install requests`)
- **urllib** - Python standard library (no dependencies)

### Coming Soon
- **Rust** - reqwest library support

## Usage

### Using the Module Programmatically

```rust
use rest_client::codegen::{generate_code, Language, Library};
use rest_client::models::request::{HttpMethod, HttpRequest};

// Create an HTTP request
let mut request = HttpRequest::new(
    "example".to_string(),
    HttpMethod::POST,
    "https://api.example.com/users".to_string(),
);
request.add_header("Content-Type".to_string(), "application/json".to_string());
request.set_body(r#"{"name": "John Doe", "email": "john@example.com"}"#.to_string());

// Generate JavaScript fetch code (using default library)
let js_code = generate_code(&request, Language::JavaScript, None).unwrap();

// Generate Python requests code (specifying library)
let py_code = generate_code(&request, Language::Python, Some(Library::Requests)).unwrap();

// Generate axios code
let axios_code = generate_code(&request, Language::JavaScript, Some(Library::Axios)).unwrap();
```

## Generated Code Examples

### JavaScript fetch()

**Input Request:**
```http
GET https://api.example.com/users/123
Authorization: Bearer token123
```

**Generated Code:**
```javascript
// Generated fetch() code for GET request
// This code uses the modern fetch API (browser/Node.js 18+)

async function makeRequest() {
  try {
    // Configure the GET request
    const options = {
      method: 'GET',
      headers: {
        'Authorization': 'Bearer token123',
      },
    };

    // Send the request to https://api.example.com/users/123
    const response = await fetch('https://api.example.com/users/123', options);

    // Check if the request was successful
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    // Parse the response based on content type
    const contentType = response.headers.get('content-type');
    let data;
    if (contentType && contentType.includes('application/json')) {
      data = await response.json();
    } else {
      data = await response.text();
    }

    // Log the response
    console.log('Status:', response.status);
    console.log('Response:', data);
    return data;
  } catch (error) {
    console.error('Request failed:', error.message);
    throw error;
  }
}

// Execute the request
makeRequest();
```

### JavaScript axios

**Input Request:**
```http
POST https://api.example.com/posts
Content-Type: application/json

{"title": "New Post", "body": "Content here"}
```

**Generated Code:**
```javascript
// Generated axios code for POST request
// This code uses the axios library
// Install: npm install axios

const axios = require('axios');
// Or for ES modules: import axios from 'axios';

async function makeRequest() {
  try {
    // Configure the POST request
    const config = {
      method: 'post',
      url: 'https://api.example.com/posts',
      headers: {
        'Content-Type': 'application/json',
      },
      data: {
        "title": "New Post",
        "body": "Content here"
      },
      timeout: 30000, // 30 second timeout
    };

    // Send the request to https://api.example.com/posts
    const response = await axios(config);

    // Log the response
    console.log('Status:', response.status);
    console.log('Headers:', response.headers);
    console.log('Data:', response.data);
    return response.data;
  } catch (error) {
    if (error.response) {
      // Server responded with error status
      console.error('Error status:', error.response.status);
      console.error('Error data:', error.response.data);
    } else if (error.request) {
      // Request was made but no response received
      console.error('No response received:', error.request);
    } else {
      // Error setting up the request
      console.error('Request setup error:', error.message);
    }
    throw error;
  }
}

// Execute the request
makeRequest();
```

### Python requests

**Input Request:**
```http
POST https://api.example.com/login
Content-Type: application/json

{"username": "user@example.com", "password": "secret123"}
```

**Generated Code:**
```python
# Generated requests code for POST request
# This code uses the requests library
# Install: pip install requests

import requests
import json

def make_request():
    """
    Send a POST request to https://api.example.com/login
    """
    try:
        # Configure the POST request
        url = 'https://api.example.com/login'
        headers = {
            'Content-Type': 'application/json',
        }

        # JSON request body
        data = {
            "username": "user@example.com",
            "password": "secret123"
        }

        # Send the POST request
        response = requests.post(
            url,
            headers=headers,
            json=data,
            timeout=30  # 30 second timeout
        )

        # Raise an exception for HTTP errors
        response.raise_for_status()

        # Parse the response
        print(f'Status Code: {response.status_code}')
        print(f'Headers: {response.headers}')

        # Try to parse as JSON, otherwise return text
        try:
            data = response.json()
            print('Response (JSON):')
            print(json.dumps(data, indent=2))
        except ValueError:
            data = response.text
            print('Response (Text):')
            print(data)

        return data

    except requests.exceptions.HTTPError as http_err:
        print(f'HTTP error occurred: {http_err}')
        raise
    except requests.exceptions.ConnectionError as conn_err:
        print(f'Connection error occurred: {conn_err}')
        raise
    except requests.exceptions.Timeout as timeout_err:
        print(f'Timeout error occurred: {timeout_err}')
        raise
    except requests.exceptions.RequestException as err:
        print(f'An error occurred: {err}')
        raise


if __name__ == '__main__':
    # Execute the request
    make_request()
```

### Python urllib

**Input Request:**
```http
GET https://api.example.com/data
Accept: application/json
X-API-Key: abc123
```

**Generated Code:**
```python
# Generated urllib code for GET request
# This code uses Python's standard library (no external dependencies)

import urllib.request
import urllib.error
import json

def make_request():
    """
    Send a GET request to https://api.example.com/data
    """
    try:
        # Configure the GET request
        url = 'https://api.example.com/data'
        data = None

        # Create the request object
        req = urllib.request.Request(
            url,
            data=data,
            method='GET'
        )

        # Add headers
        req.add_header('Accept', 'application/json')
        req.add_header('X-API-Key', 'abc123')

        # Send the GET request
        with urllib.request.urlopen(req, timeout=30) as response:
            # Read the response
            response_data = response.read()
            status_code = response.status
            headers = response.headers

            print(f'Status Code: {status_code}')
            print(f'Headers: {dict(headers)}')

            # Try to parse as JSON, otherwise return text
            try:
                data = json.loads(response_data.decode('utf-8'))
                print('Response (JSON):')
                print(json.dumps(data, indent=2))
            except (ValueError, UnicodeDecodeError):
                data = response_data.decode('utf-8', errors='replace')
                print('Response (Text):')
                print(data)

            return data

    except urllib.error.HTTPError as http_err:
        print(f'HTTP error occurred: {http_err.code} {http_err.reason}')
        raise
    except urllib.error.URLError as url_err:
        print(f'URL error occurred: {url_err.reason}')
        raise
    except Exception as err:
        print(f'An error occurred: {err}')
        raise


if __name__ == '__main__':
    # Execute the request
    make_request()
```

## Features

### Automatic String Escaping
The code generator properly escapes special characters in URLs, headers, and body content:
- Quotes (`'` and `"`)
- Newlines (`\n`)
- Tabs (`\t`)
- Backslashes (`\`)
- Control characters

### JSON Detection and Formatting
When `Content-Type: application/json` is present:
- JavaScript: Uses `JSON.stringify()` for the body
- Python requests: Uses `json=data` parameter
- Python urllib: Uses `json.dumps(data).encode('utf-8')`

The JSON is validated and pretty-printed in the generated code.

### Authentication Support
All authentication headers are preserved in the generated code:
- Bearer tokens
- Basic authentication
- API keys
- Custom authentication schemes

### Error Handling
Generated code includes comprehensive error handling:
- **JavaScript fetch**: Checks `response.ok` and throws on errors
- **JavaScript axios**: Handles `error.response`, `error.request`, and setup errors
- **Python requests**: Catches HTTPError, ConnectionError, Timeout, and RequestException
- **Python urllib**: Catches HTTPError, URLError, and general exceptions

### Response Parsing
Generated code intelligently parses responses:
- Detects JSON content type and parses accordingly
- Falls back to text for non-JSON responses
- Includes proper error handling for invalid JSON

## Testing Generated Code

After generating code, you can test it directly:

### JavaScript
```bash
# For fetch code (Node.js 18+)
node generated-code.js

# For axios code
npm install axios
node generated-code.js
```

### Python
```bash
# For requests code
pip install requests
python3 generated-code.py

# For urllib code (no installation needed)
python3 generated-code.py
```

## API Reference

### `generate_code`

```rust
pub fn generate_code(
    request: &HttpRequest,
    language: Language,
    library: Option<Library>,
) -> Result<String, CodeGenError>
```

Generates HTTP client code for the given request.

**Parameters:**
- `request` - The HTTP request to generate code for
- `language` - Target programming language (JavaScript, Python, Rust)
- `library` - Optional specific library (defaults to language's default)

**Returns:**
- `Ok(String)` - Generated code
- `Err(CodeGenError)` - Error if generation fails

**Errors:**
- `UnsupportedLanguage` - Language not yet implemented
- `UnsupportedLibrary` - Library not yet implemented
- `IncompatibleLibrary` - Library doesn't match the language
- `InvalidRequest` - Request is missing required fields

### `Language` Enum

```rust
pub enum Language {
    JavaScript,
    Python,
    Rust,  // Coming soon
}
```

**Methods:**
- `as_str()` - Returns string representation
- `all()` - Returns all available languages
- `default_library()` - Returns the default library for this language
- `available_libraries()` - Returns all libraries for this language

### `Library` Enum

```rust
pub enum Library {
    Fetch,      // JavaScript
    Axios,      // JavaScript
    Requests,   // Python
    Urllib,     // Python
    Reqwest,    // Rust (future)
}
```

**Methods:**
- `as_str()` - Returns string representation
- `language()` - Returns the language this library belongs to
- `description()` - Returns a human-readable description

## Best Practices

1. **Choose the right library:**
   - Use `fetch` for browser environments or modern Node.js
   - Use `axios` for more features and better error handling
   - Use `urllib` when you can't install external dependencies
   - Use `requests` for the most Pythonic and user-friendly code

2. **Review generated code:**
   - Always review the generated code before using in production
   - Adjust timeout values based on your needs
   - Add additional error handling as needed
   - Consider adding retry logic for production use

3. **Security:**
   - Never hardcode sensitive credentials in generated code
   - Use environment variables for API keys and tokens
   - Consider using secret management systems in production

4. **Testing:**
   - Test generated code with various response types
   - Verify error handling works as expected
   - Check timeout behavior with slow endpoints

## Future Enhancements

Planned features for future releases:
- Rust code generation with reqwest
- TypeScript type definitions
- Go net/http support
- More library options (node-fetch, aiohttp, etc.)
- Custom code templates
- Code formatting options
- Environment variable substitution in generated code