# Code Generation Usage Guide

## Overview

The REST Client extension includes a powerful code generation feature that converts your HTTP requests into executable code in various programming languages. This allows you to quickly prototype API calls in `.http` files and then generate production-ready code for your applications.

## Supported Languages and Libraries

### JavaScript
- **fetch** (default) - Native browser Fetch API, no dependencies required
- **axios** - Popular HTTP client library

### Python
- **requests** (default) - The most popular HTTP library for Python
- **urllib** - Standard library, no dependencies required

## Using Code Generation

### Method 1: Slash Command

The easiest way to generate code is using the `/generate-code` slash command:

1. Write your HTTP request in a `.http` file:
```http
GET https://api.example.com/users
Authorization: Bearer your-token-here
Accept: application/json
```

2. Select the request text

3. Type `/generate-code` followed by the language and optionally the library:
```
/generate-code javascript
/generate-code javascript axios
/generate-code python
/generate-code python urllib
```

4. The generated code will appear in a new buffer with:
   - Usage instructions in comments
   - Complete, runnable code
   - The code automatically copied to your clipboard

### Method 2: Command Functions

You can also use the code generation programmatically:

```rust
use rest_client::commands::generate_code_from_cursor;
use std::path::PathBuf;

let editor_text = "GET https://api.example.com/users\nAuthorization: Bearer token";
let cursor_pos = 10;
let file_path = PathBuf::from("test.http");

let result = generate_code_from_cursor(
    editor_text,
    cursor_pos,
    &file_path,
    "javascript",
    Some("fetch")
).unwrap();

println!("{}", result.to_display_string());
```

## Examples

### Example 1: Simple GET Request

**HTTP Request:**
```http
GET https://api.github.com/users/octocat
Accept: application/vnd.github.v3+json
```

**Generated JavaScript (fetch):**
```javascript
// Fetch API - GET request
async function makeRequest() {
  try {
    const response = await fetch('https://api.github.com/users/octocat', {
      method: 'GET',
      headers: {
        'Accept': 'application/vnd.github.v3+json'
      }
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();
    console.log('Response:', data);
    return data;
  } catch (error) {
    console.error('Request failed:', error);
    throw error;
  }
}

makeRequest();
```

**Generated Python (requests):**
```python
import requests

def make_request():
    """
    Send GET request to https://api.github.com/users/octocat
    """
    try:
        response = requests.get(
            'https://api.github.com/users/octocat',
            headers={
                'Accept': 'application/vnd.github.v3+json'
            }
        )
        
        response.raise_for_status()
        data = response.json()
        print('Response:', data)
        return data
        
    except requests.exceptions.RequestException as e:
        print(f'Request failed: {e}')
        raise

if __name__ == '__main__':
    make_request()
```

### Example 2: POST Request with JSON Body

**HTTP Request:**
```http
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9

{
  "name": "John Doe",
  "email": "john@example.com",
  "role": "developer"
}
```

**Generated JavaScript (axios):**
```javascript
const axios = require('axios');

async function makeRequest() {
  try {
    const response = await axios({
      method: 'POST',
      url: 'https://api.example.com/users',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9'
      },
      data: {
        "name": "John Doe",
        "email": "john@example.com",
        "role": "developer"
      },
      timeout: 30000
    });

    console.log('Response:', response.data);
    return response.data;
  } catch (error) {
    if (error.response) {
      console.error('Server error:', error.response.status, error.response.data);
    } else if (error.request) {
      console.error('No response received:', error.request);
    } else {
      console.error('Request failed:', error.message);
    }
    throw error;
  }
}

makeRequest();
```

### Example 3: Request with Multiple Headers

**HTTP Request:**
```http
GET https://api.example.com/data
Authorization: Bearer token123
Accept: application/json
User-Agent: MyApp/1.0
X-Custom-Header: custom-value
```

**Generated Python (urllib):**
```python
import urllib.request
import urllib.error
import json

def make_request():
    """
    Send GET request to https://api.example.com/data
    """
    url = 'https://api.example.com/data'
    headers = {
        'Authorization': 'Bearer token123',
        'Accept': 'application/json',
        'User-Agent': 'MyApp/1.0',
        'X-Custom-Header': 'custom-value'
    }
    
    try:
        request = urllib.request.Request(url, headers=headers, method='GET')
        
        with urllib.request.urlopen(request, timeout=30) as response:
            response_data = response.read().decode('utf-8')
            data = json.loads(response_data)
            print('Response:', data)
            return data
            
    except urllib.error.HTTPError as e:
        print(f'HTTP error {e.code}: {e.reason}')
        raise
    except urllib.error.URLError as e:
        print(f'URL error: {e.reason}')
        raise
    except Exception as e:
        print(f'Request failed: {e}')
        raise

if __name__ == '__main__':
    make_request()
```

## Features

### What's Included in Generated Code

✅ **Proper Error Handling**
- Try-catch blocks with detailed error messages
- HTTP status code checking
- Timeout handling

✅ **Authentication**
- Authorization headers preserved
- Bearer tokens, API keys, etc.

✅ **Request Body**
- JSON bodies properly formatted and escaped
- Form data support
- Text bodies handled correctly

✅ **Headers**
- All custom headers included
- Content-Type, Accept, etc.

✅ **Modern Patterns**
- Async/await for JavaScript
- Type hints for Python (where applicable)
- Proper imports and dependencies listed

✅ **Usage Instructions**
- Comments explaining how to run the code
- Installation instructions for dependencies
- Example usage

### Automatic Clipboard Copy

When code is generated via the slash command, it's automatically copied to your clipboard with a confirmation message. You can immediately paste it into your project files.

## Command Reference

### Slash Commands

```
/generate-code javascript       # Generate JavaScript with fetch (default)
/generate-code javascript fetch # Generate JavaScript with fetch (explicit)
/generate-code javascript axios # Generate JavaScript with axios
/generate-code python           # Generate Python with requests (default)
/generate-code python requests  # Generate Python with requests (explicit)
/generate-code python urllib    # Generate Python with urllib
```

### Viewing Available Options

To see all available languages and libraries, use the command without arguments:

```
/generate-code
```

This will display:
- All supported languages
- Available libraries for each language
- Default library for each language
- Usage examples

## Tips and Best Practices

1. **Test Your Requests First**: Always test your HTTP requests in the REST Client before generating code. This ensures the request works as expected.

2. **Use Variables**: If you're using environment variables in your `.http` files, make sure to switch to the appropriate environment before generating code. The generated code will use the resolved values.

3. **Review Generated Code**: While the generated code is production-ready, always review it to ensure it meets your specific requirements and coding standards.

4. **Dependencies**: Note which libraries require installation:
   - **fetch**: No dependencies (built-in to modern browsers and Node.js 18+)
   - **axios**: Requires `npm install axios`
   - **requests**: Requires `pip install requests`
   - **urllib**: No dependencies (Python standard library)

5. **Security**: Be careful with sensitive data like API keys and tokens. The generated code includes them as-is from your `.http` file. Consider using environment variables in production.

## Troubleshooting

### "No request found at cursor position"
Make sure your cursor is within a valid HTTP request block. Requests should start with a method (GET, POST, etc.) and URL.

### "Failed to parse request"
Check that your HTTP request syntax is correct. Common issues:
- Missing URL
- Invalid HTTP method
- Malformed headers (should be `Header-Name: value`)

### "Unknown language"
Only JavaScript and Python are currently supported. Use one of these exactly:
- `javascript` or `js`
- `python` or `py`

### "Library not compatible with language"
Make sure you're using a library that's available for the selected language:
- JavaScript: `fetch`, `axios`
- Python: `requests`, `urllib`

## Future Enhancements

Planned features for future releases:
- Go code generation (net/http)
- Rust code generation (reqwest)
- TypeScript with proper type annotations
- cURL command generation
- More library options (node-fetch, aiohttp, etc.)
- Custom templates

## See Also

- [Code Generation Examples](examples/code-generation.http)
- [REST Client Documentation](README.md)
- [API Reference](docs/API.md)