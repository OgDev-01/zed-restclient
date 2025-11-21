# REST Client for Zed

A powerful HTTP client extension for the Zed editor that allows you to send HTTP requests and view responses directly within your editor. Inspired by the popular VS Code REST Client extension.

## Features

- 📝 Send HTTP requests from `.http` and `.rest` files
- 🎨 Syntax highlighting for HTTP request files
- 📊 View formatted responses with JSON/XML pretty-printing
- 🔄 Support for all HTTP methods (GET, POST, PUT, DELETE, PATCH, etc.)
- 📦 Variable substitution and environment support (coming soon)
- 📜 Request history tracking (coming soon)
- 🔐 Authentication support (Basic, Bearer, etc.) (coming soon)
- 🌐 GraphQL support (coming soon)

## Installation

### From Zed Extensions

1. Open Zed editor
2. Open the command palette (`Cmd+Shift+P` on macOS, `Ctrl+Shift+P` on Linux/Windows)
3. Search for "zed: extensions"
4. Search for "REST Client"
5. Click "Install"

### Manual Installation

1. Clone this repository
2. Navigate to the project directory
3. Build the extension:
   ```bash
   cargo build --target wasm32-wasip1 --release
   ```
4. Install the extension in Zed by copying it to your extensions directory

## Usage

### Creating an HTTP Request File

1. Create a new file with `.http` or `.rest` extension
2. Write your HTTP request following the format below

### Basic Request Format

#### Simple GET Request

```http
GET https://api.github.com/users/octocat
```

#### GET Request with Headers

```http
GET https://api.example.com/data
Accept: application/json
Authorization: Bearer YOUR_TOKEN_HERE
```

#### POST Request with JSON Body

```http
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "My Post",
  "body": "This is the post content",
  "userId": 1
}
```

#### Multiple Requests in One File

Separate multiple requests with `###`:

```http
### Get all posts
GET https://jsonplaceholder.typicode.com/posts

### Create a new post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "New Post",
  "body": "Content here",
  "userId": 1
}

### Get a specific post
GET https://jsonplaceholder.typicode.com/posts/1
```

### Sending Requests

1. Place your cursor anywhere within the request you want to send
2. Open the command palette (`Cmd+Shift+P` on macOS, `Ctrl+Shift+P` on Linux/Windows)
3. Search for "rest-client: send request"
4. Press Enter

The response will appear in a new editor pane showing:
- HTTP status code and reason
- Response headers
- Formatted response body
- Request duration and size

### Comments

Add comments to your request files using `#` or `//`:

```http
# This is a comment
// This is also a comment

### Get user data
GET https://api.example.com/users/123
```

## Request Format Specification

### Request Line

```
METHOD URL [HTTP_VERSION]
```

- `METHOD`: HTTP method (GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD, TRACE, CONNECT)
- `URL`: Complete URL including protocol (http:// or https://)
- `HTTP_VERSION`: Optional, defaults to HTTP/1.1

### Headers

Headers follow the request line, one per line:

```
Header-Name: Header-Value
```

### Request Body

Add a blank line after headers, then include your request body:

```http
POST https://api.example.com/data
Content-Type: application/json

{
  "key": "value"
}
```

## Examples

### REST API Testing

```http
### List all users
GET https://jsonplaceholder.typicode.com/users

### Get specific user
GET https://jsonplaceholder.typicode.com/users/1

### Create user
POST https://jsonplaceholder.typicode.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

### Update user
PUT https://jsonplaceholder.typicode.com/users/1
Content-Type: application/json

{
  "name": "Jane Doe",
  "email": "jane@example.com"
}

### Delete user
DELETE https://jsonplaceholder.typicode.com/users/1
```

### Testing with Custom Headers

```http
### Request with custom headers
GET https://httpbin.org/headers
User-Agent: REST-Client-Zed/1.0
X-Custom-Header: custom-value
Accept: application/json
```

### Form Data

```http
POST https://httpbin.org/post
Content-Type: application/x-www-form-urlencoded

username=johndoe&password=secret123
```

## Keyboard Shortcuts

Currently, there are no default keyboard shortcuts. You can add custom shortcuts in Zed's keymap settings:

```json
{
  "context": "Editor && (extension == 'http' || extension == 'rest')",
  "bindings": {
    "ctrl-alt-r": "rest-client: send request"
  }
}
```

## Troubleshooting

### Extension not loading

1. Ensure you have the latest version of Zed installed
2. Check the Zed logs for any error messages
3. Try reinstalling the extension

### Requests failing

1. Verify your URL is correct and includes the protocol (`http://` or `https://`)
2. Check your internet connection
3. Verify any authentication tokens or API keys are valid
4. Check if the API endpoint requires specific headers

### Syntax highlighting not working

1. Ensure your file has the `.http` or `.rest` extension
2. Reload Zed or restart the editor
3. Check if the language mode is set to "HTTP" in the status bar

## Roadmap

- [x] Basic HTTP request execution
- [x] Syntax highlighting for .http files
- [x] Response formatting (JSON, XML, HTML)
- [ ] Variable substitution (`{{variable}}`)
- [ ] Environment files support
- [ ] Request history
- [ ] Authentication helpers (Basic, Bearer, Digest)
- [ ] GraphQL support
- [ ] Code generation (cURL, JavaScript, Python, etc.)
- [ ] Advanced Tree-sitter grammar
- [ ] Response time graphs
- [ ] Certificate management

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Development

### Prerequisites

- Rust toolchain (latest stable)
- `wasm32-wasip1` target installed:
  ```bash
  rustup target add wasm32-wasip1
  ```

### Building

```bash
cargo build --target wasm32-wasip1 --release
```

### Testing

```bash
cargo test
```

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

- Inspired by [VS Code REST Client](https://github.com/Huachao/vscode-restclient)
- Built for [Zed](https://zed.dev/) editor
- Thanks to all contributors and the Zed community

## Support

For issues, questions, or suggestions:
- Open an issue on GitHub
- Check existing issues for similar problems
- Consult the Zed documentation for extension-related questions

---

**Happy API Testing! 🚀**