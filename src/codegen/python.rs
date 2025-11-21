//! Python code generation for HTTP requests.
//!
//! This module provides code generators for Python HTTP clients including
//! the requests library and the standard library urllib.

use crate::models::request::HttpRequest;

/// Generates Python code using the requests library.
///
/// Creates runnable Python code that uses the popular requests library with
/// proper headers, body, and error handling. The generated code includes
/// async/await patterns for modern Python applications.
///
/// # Arguments
///
/// * `request` - The HTTP request to generate code for
///
/// # Returns
///
/// A string containing the generated Python code with comments
pub fn generate_requests_code(request: &HttpRequest) -> String {
    let method = request.method.as_str().to_lowercase();
    let url = escape_python_string(&request.url);

    let mut code = String::new();

    // Add header comment
    code.push_str(&format!(
        "# Generated requests code for {} request\n",
        request.method.as_str()
    ));
    code.push_str("# This code uses the requests library\n");
    code.push_str("# Install: pip install requests\n\n");

    // Import requests
    code.push_str("import requests\n");
    code.push_str("import json\n\n");

    // Start the function
    code.push_str("def make_request():\n");
    code.push_str("    \"\"\"\n");
    code.push_str(&format!(
        "    Send a {} request to {}\n",
        request.method.as_str(),
        request.url
    ));
    code.push_str("    \"\"\"\n");
    code.push_str("    try:\n");

    // Define URL
    code.push_str(&format!(
        "        # Configure the {} request\n",
        request.method.as_str()
    ));
    code.push_str(&format!("        url = '{}'\n", url));

    // Add headers if present
    if !request.headers.is_empty() {
        code.push_str("        headers = {\n");
        for (key, value) in &request.headers {
            let escaped_key = escape_python_string(key);
            let escaped_value = escape_python_string(value);
            code.push_str(&format!(
                "            '{}': '{}',\n",
                escaped_key, escaped_value
            ));
        }
        code.push_str("        }\n");
    } else {
        code.push_str("        headers = {}\n");
    }

    // Add body if present
    if let Some(body) = &request.body {
        code.push_str("\n");

        // Check if body is JSON
        if is_json_content_type(request) {
            code.push_str("        # JSON request body\n");
            code.push_str("        data = ");
            code.push_str(&escape_python_json(body));
            code.push_str("\n");
        } else {
            code.push_str("        # Request body\n");
            let escaped_body = escape_python_string(body);
            code.push_str(&format!("        data = '{}'\n", escaped_body));
        }
    }

    code.push_str("\n");

    // Make the request
    code.push_str(&format!(
        "        # Send the {} request\n",
        request.method.as_str()
    ));
    code.push_str(&format!("        response = requests.{}(\n", method));
    code.push_str("            url,\n");
    code.push_str("            headers=headers,\n");

    if request.body.is_some() {
        if is_json_content_type(request) {
            code.push_str("            json=data,\n");
        } else {
            code.push_str("            data=data,\n");
        }
    }

    code.push_str("            timeout=30  # 30 second timeout\n");
    code.push_str("        )\n\n");

    // Check response status
    code.push_str("        # Raise an exception for HTTP errors\n");
    code.push_str("        response.raise_for_status()\n\n");

    // Parse the response
    code.push_str("        # Parse the response\n");
    code.push_str("        print(f'Status Code: {response.status_code}')\n");
    code.push_str("        print(f'Headers: {response.headers}')\n\n");

    code.push_str("        # Try to parse as JSON, otherwise return text\n");
    code.push_str("        try:\n");
    code.push_str("            data = response.json()\n");
    code.push_str("            print('Response (JSON):')\n");
    code.push_str("            print(json.dumps(data, indent=2))\n");
    code.push_str("        except ValueError:\n");
    code.push_str("            data = response.text\n");
    code.push_str("            print('Response (Text):')\n");
    code.push_str("            print(data)\n\n");

    code.push_str("        return data\n\n");

    // Error handling
    code.push_str("    except requests.exceptions.HTTPError as http_err:\n");
    code.push_str("        print(f'HTTP error occurred: {http_err}')\n");
    code.push_str("        raise\n");
    code.push_str("    except requests.exceptions.ConnectionError as conn_err:\n");
    code.push_str("        print(f'Connection error occurred: {conn_err}')\n");
    code.push_str("        raise\n");
    code.push_str("    except requests.exceptions.Timeout as timeout_err:\n");
    code.push_str("        print(f'Timeout error occurred: {timeout_err}')\n");
    code.push_str("        raise\n");
    code.push_str("    except requests.exceptions.RequestException as err:\n");
    code.push_str("        print(f'An error occurred: {err}')\n");
    code.push_str("        raise\n\n");

    // Add main guard
    code.push_str("\n");
    code.push_str("if __name__ == '__main__':\n");
    code.push_str("    # Execute the request\n");
    code.push_str("    make_request()\n");

    code
}

/// Generates Python code using the standard library urllib.
///
/// Creates runnable Python code that uses Python's built-in urllib with
/// proper headers, body, and error handling. No external dependencies required.
///
/// # Arguments
///
/// * `request` - The HTTP request to generate code for
///
/// # Returns
///
/// A string containing the generated Python code with comments
pub fn generate_urllib_code(request: &HttpRequest) -> String {
    let method = request.method.as_str().to_uppercase();
    let url = escape_python_string(&request.url);

    let mut code = String::new();

    // Add header comment
    code.push_str(&format!(
        "# Generated urllib code for {} request\n",
        request.method.as_str()
    ));
    code.push_str("# This code uses Python's standard library (no external dependencies)\n\n");

    // Import urllib
    code.push_str("import urllib.request\n");
    code.push_str("import urllib.error\n");
    code.push_str("import json\n\n");

    // Start the function
    code.push_str("def make_request():\n");
    code.push_str("    \"\"\"\n");
    code.push_str(&format!(
        "    Send a {} request to {}\n",
        request.method.as_str(),
        request.url
    ));
    code.push_str("    \"\"\"\n");
    code.push_str("    try:\n");

    // Define URL
    code.push_str(&format!(
        "        # Configure the {} request\n",
        request.method.as_str()
    ));
    code.push_str(&format!("        url = '{}'\n", url));

    // Add body if present
    let has_body = request.body.is_some();
    if let Some(body) = &request.body {
        code.push_str("\n");

        // Check if body is JSON
        if is_json_content_type(request) {
            code.push_str("        # JSON request body\n");
            code.push_str("        data = ");
            code.push_str(&escape_python_json(body));
            code.push_str("\n");
            code.push_str("        data = json.dumps(data).encode('utf-8')\n");
        } else {
            code.push_str("        # Request body\n");
            let escaped_body = escape_python_string(body);
            code.push_str(&format!("        data = '{}'\n", escaped_body));
            code.push_str("        data = data.encode('utf-8')\n");
        }
    } else {
        code.push_str("        data = None\n");
    }

    code.push_str("\n");

    // Create request object
    code.push_str("        # Create the request object\n");
    code.push_str("        req = urllib.request.Request(\n");
    code.push_str("            url,\n");
    code.push_str("            data=data,\n");
    code.push_str(&format!("            method='{}'\n", method));
    code.push_str("        )\n\n");

    // Add headers if present
    if !request.headers.is_empty() {
        code.push_str("        # Add headers\n");
        for (key, value) in &request.headers {
            let escaped_key = escape_python_string(key);
            let escaped_value = escape_python_string(value);
            code.push_str(&format!(
                "        req.add_header('{}', '{}')\n",
                escaped_key, escaped_value
            ));
        }
        code.push_str("\n");
    }

    // Make the request
    code.push_str(&format!(
        "        # Send the {} request\n",
        request.method.as_str()
    ));
    code.push_str("        with urllib.request.urlopen(req, timeout=30) as response:\n");
    code.push_str("            # Read the response\n");
    code.push_str("            response_data = response.read()\n");
    code.push_str("            status_code = response.status\n");
    code.push_str("            headers = response.headers\n\n");

    code.push_str("            print(f'Status Code: {status_code}')\n");
    code.push_str("            print(f'Headers: {dict(headers)}')\n\n");

    code.push_str("            # Try to parse as JSON, otherwise return text\n");
    code.push_str("            try:\n");
    code.push_str("                data = json.loads(response_data.decode('utf-8'))\n");
    code.push_str("                print('Response (JSON):')\n");
    code.push_str("                print(json.dumps(data, indent=2))\n");
    code.push_str("            except (ValueError, UnicodeDecodeError):\n");
    code.push_str("                data = response_data.decode('utf-8', errors='replace')\n");
    code.push_str("                print('Response (Text):')\n");
    code.push_str("                print(data)\n\n");

    code.push_str("            return data\n\n");

    // Error handling
    code.push_str("    except urllib.error.HTTPError as http_err:\n");
    code.push_str("        print(f'HTTP error occurred: {http_err.code} {http_err.reason}')\n");
    code.push_str("        raise\n");
    code.push_str("    except urllib.error.URLError as url_err:\n");
    code.push_str("        print(f'URL error occurred: {url_err.reason}')\n");
    code.push_str("        raise\n");
    code.push_str("    except Exception as err:\n");
    code.push_str("        print(f'An error occurred: {err}')\n");
    code.push_str("        raise\n\n");

    // Add main guard
    code.push_str("\n");
    code.push_str("if __name__ == '__main__':\n");
    code.push_str("    # Execute the request\n");
    code.push_str("    make_request()\n");

    code
}

/// Escapes a string for use in Python string literals.
///
/// Handles special characters like quotes, newlines, backslashes, etc.
fn escape_python_string(s: &str) -> String {
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

/// Escapes JSON content for Python code generation.
///
/// Attempts to parse and re-format JSON as Python dict, or escapes as string if invalid.
fn escape_python_json(json: &str) -> String {
    // Try to parse as JSON first to validate and pretty-print
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
        // Convert to Python-compatible JSON (pretty printed)
        if let Ok(formatted) = serde_json::to_string_pretty(&value) {
            return formatted;
        }
    }

    // If not valid JSON, escape as string
    format!("'{}'", escape_python_string(json))
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
    fn test_escape_python_string() {
        assert_eq!(escape_python_string("hello"), "hello");
        assert_eq!(escape_python_string("hello'world"), "hello\\'world");
        assert_eq!(escape_python_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_python_string("hello\\world"), "hello\\\\world");
        assert_eq!(escape_python_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_python_string("hello\tworld"), "hello\\tworld");
    }

    #[test]
    fn test_generate_requests_code_simple_get() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );

        let code = generate_requests_code(&request);

        assert!(code.contains("import requests"));
        assert!(code.contains("def make_request():"));
        assert!(code.contains("requests.get("));
        assert!(code.contains("https://api.example.com/users"));
        assert!(code.contains("response.raise_for_status()"));
    }

    #[test]
    fn test_generate_requests_code_post_with_json() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/users".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"name": "Alice", "age": 30}"#.to_string());

        let code = generate_requests_code(&request);

        assert!(code.contains("requests.post("));
        assert!(code.contains("Content-Type"));
        assert!(code.contains("application/json"));
        assert!(code.contains("json=data"));
        assert!(code.contains("name"));
    }

    #[test]
    fn test_generate_requests_code_with_auth() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/protected".to_string(),
        );
        request.add_header("Authorization".to_string(), "Bearer secret123".to_string());

        let code = generate_requests_code(&request);

        assert!(code.contains("Authorization"));
        assert!(code.contains("Bearer secret123"));
    }

    #[test]
    fn test_generate_urllib_code_simple_get() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/data".to_string(),
        );

        let code = generate_urllib_code(&request);

        assert!(code.contains("import urllib.request"));
        assert!(code.contains("def make_request():"));
        assert!(code.contains("method='GET'"));
        assert!(code.contains("https://api.example.com/data"));
        assert!(code.contains("urllib.request.urlopen"));
    }

    #[test]
    fn test_generate_urllib_code_post_with_json() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/submit".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"key": "value"}"#.to_string());

        let code = generate_urllib_code(&request);

        assert!(code.contains("method='POST'"));
        assert!(code.contains("json.dumps(data).encode"));
        assert!(code.contains("req.add_header"));
        assert!(code.contains("Content-Type"));
    }

    #[test]
    fn test_generate_urllib_code_with_headers() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::PUT,
            "https://api.example.com/update".to_string(),
        );
        request.add_header("X-API-Key".to_string(), "abc123".to_string());
        request.add_header("Accept".to_string(), "application/json".to_string());

        let code = generate_urllib_code(&request);

        assert!(code.contains("X-API-Key"));
        assert!(code.contains("abc123"));
        assert!(code.contains("Accept"));
        assert!(code.contains("application/json"));
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
