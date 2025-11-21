# WASM Build Guide for Zed REST Client Extension

This document explains how to build the REST Client extension for Zed, including WASM compatibility requirements and known limitations.

## Prerequisites

1. **Rust toolchain** with WASM target support:
   ```bash
   rustup target add wasm32-wasip1
   ```

2. **Zed Editor** (to test the extension)

## Building the Extension

To build the extension for Zed:

```bash
cargo build --target wasm32-wasip1 --release
```

The compiled WASM binary will be located at:
```
target/wasm32-wasip1/release/rest_client.wasm
```

## WASM Compatibility Requirements

Zed extensions run in a WebAssembly (WASM) environment with strict limitations. The following must be observed:

### ✅ Allowed Dependencies

- `zed_extension_api` (v0.7.0 or later) - Required
- `serde` and `serde_json` - For serialization
- `regex` - For pattern matching
- `url` - For URL parsing
- Any other WASM-compatible crates without async runtime dependencies

### ❌ Forbidden Dependencies

The following crates **CANNOT** be used in Zed extensions:

- **`tokio`** - Async runtime with features beyond WASM support
  - ❌ `rt-multi-thread` feature is not supported
  - ❌ `rt` feature is not supported  
  - Only `sync`, `macros`, `io-util`, `rt`, and `time` are theoretically supported, but even these are problematic in practice

- **`reqwest`** - HTTP client library
  - Depends on `tokio` with unsupported features
  - Not compatible with `wasm32-wasip1` target

- **`async-std`** - Alternative async runtime
  - Not compatible with WASM targets used by Zed

### ✅ HTTP Requests in WASM

Instead of `reqwest` or `tokio`, use **Zed's built-in HTTP client**:

```rust
use zed_extension_api::http_client;

// Build a request
let request = http_client::HttpRequest::builder()
    .method(http_client::HttpMethod::Get)
    .url("https://api.example.com/data")
    .header("Content-Type", "application/json")
    .body(b"request body".to_vec())
    .build()?;

// Execute the request (synchronous)
let response = request.fetch()?;

// Access response data
let headers: Vec<(String, String)> = response.headers;
let body: Vec<u8> = response.body;
```

## Known Limitations

### 1. No HTTP Status Codes

**CRITICAL LIMITATION**: The Zed HTTP client API (as of v0.7.0) does **NOT** provide HTTP status codes in responses.

- `HttpResponse` only contains `headers` and `body` fields
- No `status`, `status_code`, or similar field exists
- Cannot distinguish between 200 OK, 404 Not Found, 500 Internal Server Error, etc.
- Success is determined solely by whether the request completes without error

**Impact on REST Client**:
- The extension assumes all successful requests return `200 OK`
- Cannot display actual HTTP status codes to users
- Error responses that return 4xx or 5xx may not be distinguishable from success

**Workaround**:
- Check response body content for error messages
- Look for error indicators in response headers

### 2. No Request Timeouts

The Zed HTTP client API does not support configurable timeouts:

- The `ExecutionConfig` timeout parameter is currently ignored
- Requests will use whatever default timeout the Zed runtime provides
- Long-running requests may hang indefinitely

### 3. No Detailed Timing Information

Cannot measure individual phases of HTTP requests:

- No DNS lookup time
- No TCP connection time
- No TLS handshake time
- No time-to-first-byte
- Only total request duration can be measured

### 4. Limited HTTP Methods

The following HTTP methods are **NOT supported** by Zed's HTTP client:

- `TRACE` - Will return `UnsupportedMethod` error
- `CONNECT` - Will return `UnsupportedMethod` error

Supported methods:
- `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`

### 5. No Async/Await

All operations must be **synchronous**:

- No `async fn`
- No `.await` calls
- The Zed HTTP client is synchronous by design

## Troubleshooting Build Errors

### Error: "Only features sync,macros,io-util,rt,time are supported on wasm"

**Cause**: You have `tokio` with unsupported features in your `Cargo.toml`

**Solution**: Remove `tokio` entirely and use Zed's HTTP client:

```toml
# ❌ This will fail:
[dependencies]
tokio = { version = "1.35", features = ["rt-multi-thread", "macros"] }

# ✅ Use this instead:
[dependencies]
zed_extension_api = "0.7.0"
```

### Error: "could not compile `reqwest`"

**Cause**: `reqwest` is not WASM-compatible for `wasm32-wasip1`

**Solution**: Remove `reqwest` and use `zed_extension_api::http_client`

### Error: "cannot find function `execute_request` in this scope"

**Cause**: The function signature may have changed (no longer `async`)

**Solution**: Remove `.await` from function calls:

```rust
// ❌ Old (async):
let response = execute_request(&request, &config).await?;

// ✅ New (sync):
let response = execute_request(&request, &config)?;
```

## Testing

### Unit Tests

Standard unit tests can be run with:

```bash
cargo test
```

**Note**: Tests that use `zed_extension_api::http_client` will **NOT** work in standard test environments because the HTTP client is only available in the Zed WASM runtime.

### Integration Tests

HTTP request functionality must be tested manually within Zed:

1. Build the extension: `cargo build --target wasm32-wasip1 --release`
2. Install the extension in Zed
3. Create a `.http` file with test requests
4. Execute requests and verify behavior

## Dependencies Reference

Current `Cargo.toml` dependencies (all WASM-compatible):

```toml
[dependencies]
zed_extension_api = "0.7.0"  # Required for Zed integration
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
url = "2.5"
```

## Version History

- **v0.7.0** (Current): Uses `zed_extension_api` 0.7.0 with synchronous HTTP client
- **v0.2.0** (Deprecated): Used `reqwest` and `tokio` - **NOT WASM-compatible**

## Additional Resources

- [Zed Extension API Documentation](https://docs.rs/zed_extension_api/latest/zed_extension_api/)
- [Zed Extensions Repository](https://github.com/zed-industries/extensions)
- [WASM Target Documentation](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1.html)

## Future Improvements

Potential enhancements that would require Zed API updates:

1. **Status Code Support**: Add `status_code: u16` field to `HttpResponse`
2. **Timeout Configuration**: Support configurable request timeouts
3. **Detailed Timing**: Expose DNS, TCP, TLS, and download timing metrics
4. **Streaming Responses**: Better support for large response bodies
5. **Request Cancellation**: Ability to abort in-flight requests

If you need these features, consider requesting them from the Zed team or contributing to the `zed_extension_api` crate.