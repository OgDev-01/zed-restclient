//! JavaScript code generation for HTTP requests.
//!
//! This module provides code generators for JavaScript HTTP clients including
//! browser fetch() API and the axios library.

use crate::models::request::HttpRequest;

/// Generates JavaScript code using the browser fetch() API.
///
/// Creates runnable JavaScript code that uses the modern fetch() API with
/// proper headers, body, and error handling. The generated code includes
/// async/await patterns and is compatible with both browsers and Node.js.
///
/// # Arguments
///
/// * `request` - The HTTP request to generate code for
///
/// # Returns
///
/// A string containing the generated JavaScript code with comments
pub fn generate_fetch_code(request: &HttpRequest) -> String {
    let method = request.method.as_str();
    let url = escape_js_string(&request.url);

    let mut code = String::new();

    // Add header comment
    code.push_str(&format!(
        "// Generated fetch() code for {} request\n",
        method
    ));
    code.push_str("// This code uses the modern fetch API (browser/Node.js 18+)\n\n");

    // Start the async function
    code.push_str("async function makeRequest() {\n");
    code.push_str("  try {\n");

    // Build the fetch options
    code.push_str(&format!("    // Configure the {} request\n", method));
    code.push_str("    const options = {\n");
    code.push_str(&format!("      method: '{}',\n", method));

    // Add headers if present
    if !request.headers.is_empty() {
        code.push_str("      headers: {\n");
        for (key, value) in &request.headers {
            let escaped_key = escape_js_string(key);
            let escaped_value = escape_js_string(value);
            code.push_str(&format!(
                "        '{}': '{}',\n",
                escaped_key, escaped_value
            ));
        }
        code.push_str("      },\n");
    }

    // Add body if present
    if let Some(body) = &request.body {
        code.push_str("      body: ");

        // Check if body is JSON
        if is_json_content_type(request) {
            code.push_str("JSON.stringify(");
            code.push_str(&escape_js_json(body));
            code.push_str("),\n");
        } else {
            let escaped_body = escape_js_string(body);
            code.push_str(&format!("'{}',\n", escaped_body));
        }
    }

    code.push_str("    };\n\n");

    // Make the fetch call
    code.push_str(&format!("    // Send the request to {}\n", request.url));
    code.push_str(&format!(
        "    const response = await fetch('{}', options);\n\n",
        url
    ));

    // Check response status
    code.push_str("    // Check if the request was successful\n");
    code.push_str("    if (!response.ok) {\n");
    code.push_str("      throw new Error(`HTTP error! status: ${response.status}`);\n");
    code.push_str("    }\n\n");

    // Parse the response
    code.push_str("    // Parse the response based on content type\n");
    code.push_str("    const contentType = response.headers.get('content-type');\n");
    code.push_str("    let data;\n");
    code.push_str("    if (contentType && contentType.includes('application/json')) {\n");
    code.push_str("      data = await response.json();\n");
    code.push_str("    } else {\n");
    code.push_str("      data = await response.text();\n");
    code.push_str("    }\n\n");

    // Log the result
    code.push_str("    // Log the response\n");
    code.push_str("    console.log('Status:', response.status);\n");
    code.push_str("    console.log('Response:', data);\n");
    code.push_str("    return data;\n");

    // Error handling
    code.push_str("  } catch (error) {\n");
    code.push_str("    console.error('Request failed:', error.message);\n");
    code.push_str("    throw error;\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");

    // Add call to the function
    code.push_str("// Execute the request\n");
    code.push_str("makeRequest();\n");

    code
}

/// Generates JavaScript code using the axios library.
///
/// Creates runnable JavaScript code that uses axios with proper configuration,
/// headers, body, and error handling. The generated code includes async/await
/// patterns and comprehensive error handling.
///
/// # Arguments
///
/// * `request` - The HTTP request to generate code for
///
/// # Returns
///
/// A string containing the generated JavaScript code with comments
pub fn generate_axios_code(request: &HttpRequest) -> String {
    let method = request.method.as_str().to_lowercase();
    let url = escape_js_string(&request.url);

    let mut code = String::new();

    // Add header comment
    code.push_str(&format!(
        "// Generated axios code for {} request\n",
        request.method.as_str()
    ));
    code.push_str("// This code uses the axios library\n");
    code.push_str("// Install: npm install axios\n\n");

    // Import axios
    code.push_str("const axios = require('axios');\n");
    code.push_str("// Or for ES modules: import axios from 'axios';\n\n");

    // Start the async function
    code.push_str("async function makeRequest() {\n");
    code.push_str("  try {\n");

    // Build the axios config
    code.push_str(&format!(
        "    // Configure the {} request\n",
        request.method.as_str()
    ));
    code.push_str("    const config = {\n");
    code.push_str(&format!("      method: '{}',\n", method));
    code.push_str(&format!("      url: '{}',\n", url));

    // Add headers if present
    if !request.headers.is_empty() {
        code.push_str("      headers: {\n");
        for (key, value) in &request.headers {
            let escaped_key = escape_js_string(key);
            let escaped_value = escape_js_string(value);
            code.push_str(&format!(
                "        '{}': '{}',\n",
                escaped_key, escaped_value
            ));
        }
        code.push_str("      },\n");
    }

    // Add body if present
    if let Some(body) = &request.body {
        code.push_str("      data: ");

        // Check if body is JSON
        if is_json_content_type(request) {
            code.push_str(&escape_js_json(body));
            code.push_str(",\n");
        } else {
            let escaped_body = escape_js_string(body);
            code.push_str(&format!("'{}',\n", escaped_body));
        }
    }

    // Add timeout
    code.push_str("      timeout: 30000, // 30 second timeout\n");

    code.push_str("    };\n\n");

    // Make the axios call
    code.push_str(&format!("    // Send the request to {}\n", request.url));
    code.push_str("    const response = await axios(config);\n\n");

    // Log the result
    code.push_str("    // Log the response\n");
    code.push_str("    console.log('Status:', response.status);\n");
    code.push_str("    console.log('Headers:', response.headers);\n");
    code.push_str("    console.log('Data:', response.data);\n");
    code.push_str("    return response.data;\n");

    // Error handling with axios error details
    code.push_str("  } catch (error) {\n");
    code.push_str("    if (error.response) {\n");
    code.push_str("      // Server responded with error status\n");
    code.push_str("      console.error('Error status:', error.response.status);\n");
    code.push_str("      console.error('Error data:', error.response.data);\n");
    code.push_str("    } else if (error.request) {\n");
    code.push_str("      // Request was made but no response received\n");
    code.push_str("      console.error('No response received:', error.request);\n");
    code.push_str("    } else {\n");
    code.push_str("      // Error setting up the request\n");
    code.push_str("      console.error('Request setup error:', error.message);\n");
    code.push_str("    }\n");
    code.push_str("    throw error;\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");

    // Add call to the function
    code.push_str("// Execute the request\n");
    code.push_str("makeRequest();\n");

    code
}

/// Escapes a string for use in JavaScript string literals.
///
/// Handles special characters like quotes, newlines, backslashes, etc.
fn escape_js_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\'' => "\\'".to_string(),
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            c if c.is_control() => format!("\\u{:04x}", c as u32),
            c => c.to_string(),
        })
        .collect()
}

/// Escapes JSON content for JavaScript code generation.
///
/// Attempts to parse and re-format JSON, or escapes as string if invalid.
fn escape_js_json(json: &str) -> String {
    // Try to parse as JSON first to validate and pretty-print
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
        // Pretty print the JSON with 2-space indentation
        if let Ok(formatted) = serde_json::to_string_pretty(&value) {
            return formatted;
        }
    }

    // If not valid JSON, escape as string
    format!("'{}'", escape_js_string(json))
}

/// Checks if the request has a JSON content type.
fn is_json_content_type(request: &HttpRequest) -> bool {
    request
        .content_type()
        .map(|ct| ct.to_lowercase().contains("json"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::HttpMethod;

    #[test]
    fn test_escape_js_string() {
        assert_eq!(escape_js_string("hello"), "hello");
        assert_eq!(escape_js_string("hello'world"), "hello\\'world");
        assert_eq!(escape_js_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_js_string("hello\\world"), "hello\\\\world");
        assert_eq!(escape_js_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_js_string("hello\tworld"), "hello\\tworld");
    }

    #[test]
    fn test_generate_fetch_code_simple_get() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );

        let code = generate_fetch_code(&request);

        assert!(code.contains("async function makeRequest()"));
        assert!(code.contains("method: 'GET'"));
        assert!(code.contains("https://api.example.com/users"));
        assert!(code.contains("await fetch"));
        assert!(code.contains("console.log"));
    }

    #[test]
    fn test_generate_fetch_code_post_with_json() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/users".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"name": "John Doe"}"#.to_string());

        let code = generate_fetch_code(&request);

        assert!(code.contains("method: 'POST'"));
        assert!(code.contains("Content-Type"));
        assert!(code.contains("application/json"));
        assert!(code.contains("JSON.stringify"));
        assert!(code.contains("name"));
    }

    #[test]
    fn test_generate_fetch_code_with_auth() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/protected".to_string(),
        );
        request.add_header("Authorization".to_string(), "Bearer token123".to_string());

        let code = generate_fetch_code(&request);

        assert!(code.contains("Authorization"));
        assert!(code.contains("Bearer token123"));
    }

    #[test]
    fn test_generate_axios_code_simple_get() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );

        let code = generate_axios_code(&request);

        assert!(code.contains("const axios = require('axios')"));
        assert!(code.contains("async function makeRequest()"));
        assert!(code.contains("method: 'get'"));
        assert!(code.contains("url: 'https://api.example.com/users'"));
        assert!(code.contains("await axios(config)"));
    }

    #[test]
    fn test_generate_axios_code_post_with_json() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/users".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"email": "test@example.com"}"#.to_string());

        let code = generate_axios_code(&request);

        assert!(code.contains("method: 'post'"));
        assert!(code.contains("data:"));
        assert!(code.contains("email"));
        assert!(code.contains("error.response"));
    }

    #[test]
    fn test_generate_axios_code_with_headers() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::DELETE,
            "https://api.example.com/users/123".to_string(),
        );
        request.add_header("Authorization".to_string(), "Bearer secret".to_string());
        request.add_header("X-Custom-Header".to_string(), "custom-value".to_string());

        let code = generate_axios_code(&request);

        assert!(code.contains("Authorization"));
        assert!(code.contains("Bearer secret"));
        assert!(code.contains("X-Custom-Header"));
        assert!(code.contains("custom-value"));
    }

    #[test]
    fn test_is_json_content_type() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://example.com".to_string(),
        );

        assert!(!is_json_content_type(&request));

        request.add_header("Content-Type".to_string(), "application/json".to_string());
        assert!(is_json_content_type(&request));

        request.headers.clear();
        request.add_header(
            "content-type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        assert!(is_json_content_type(&request));
    }
}
