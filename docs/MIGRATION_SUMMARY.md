# Migration Summary: From reqwest/tokio to Zed HTTP Client

## Overview

This document summarizes the migration from `reqwest`/`tokio` to the Zed extension API's built-in HTTP client to achieve WASM compatibility.

## Problem

The original implementation used:
- `reqwest` v0.11 for HTTP requests
- `tokio` v1.35 with `rt-multi-thread` feature for async runtime

**Build Error**:
```
error: Only features sync,macros,io-util,rt,time are supported on wasm.
   --> tokio-1.48.0/src/lib.rs:481:1
481 | compile_error!("Only features sync,macros,io-util,rt,time are supported on wasm.");
```

**Root Cause**: Zed extensions run in a `wasm32-wasip1` environment where:
1. `tokio`'s `rt-multi-thread` feature is not supported
2. `reqwest` depends on `tokio` and is not WASM-compatible
3. Standard async runtimes cannot be used in WASM

## Solution

Migrated to **Zed's built-in HTTP client API** (`zed_extension_api::http_client`), which is:
- ✅ WASM-compatible
- ✅ Synchronous (no async/await needed)
- ✅ Built into the Zed extension runtime

## Changes Made

### 1. Updated Dependencies

**Before** (`Cargo.toml`):
```toml
[dependencies]
zed_extension_api = "0.2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
reqwest = { version = "0.11", default-features = false, features = ["rustls-tls", "json"] }
tokio = { version = "1.35", features = ["time", "macros", "rt-multi-thread"] }
url = "2.5"
```

**After** (`Cargo.toml`):
```toml
[dependencies]
zed_extension_api = "0.7.0"  # Updated to latest
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
url = "2.5"
# Removed: reqwest and tokio
```

### 2. Refactored HTTP Executor

**Before** (`src/executor/mod.rs`):
```rust
pub async fn execute_request(
    request: &HttpRequest,
    config: &ExecutionConfig,
) -> Result<HttpResponse, RequestError> {
    let client = reqwest::Client::builder()
        .timeout(config.timeout_duration())
        .build()?;
    
    let mut req_builder = match request.method {
        HttpMethod::GET => client.get(&request.url),
        // ... more methods
    };
    
    let response = tokio::time::timeout(
        config.timeout_duration(), 
        client.execute(req)
    ).await??;
    
    let status_code = response.status().as_u16();
    let body = response.bytes().await?.to_vec();
    
    // ... process response
}
```

**After** (`src/executor/mod.rs`):
```rust
pub fn execute_request(  // No longer async
    request: &HttpRequest,
    _config: &ExecutionConfig,  // Timeout not supported
) -> Result<HttpResponse, RequestError> {
    let method = match request.method {
        HttpMethod::GET => ZedHttpMethod::Get,
        // ... convert to Zed HTTP methods
    };
    
    let request = http_client::HttpRequest::builder()
        .method(method)
        .url(&request.url)
        .headers(request.headers.clone())
        .body(request.body.clone())
        .build()?;
    
    let response = request.fetch()?;  // Synchronous
    
    // LIMITATION: No status code available
    let status_code = 200u16;  // Assume success
    let body = response.body.clone();
    
    // ... process response
}
```

### 3. Updated Command Handler

**Before** (`src/commands.rs`):
```rust
let response = execute_request(&request, &config)
    .await
    .map_err(|e| CommandError::ExecutionError(e.to_string()))?;
```

**After** (`src/commands.rs`):
```rust
let response = execute_request(&request, &config)  // No .await
    .map_err(|e| CommandError::ExecutionError(e.to_string()))?;
```

### 4. Updated Error Types

**Added** (`src/executor/error.rs`):
```rust
pub enum RequestError {
    // ... existing variants
    
    /// Unsupported HTTP method.
    UnsupportedMethod(String),  // New variant for TRACE/CONNECT
}
```

**Removed**:
- `From<reqwest::Error>` implementation
- All reqwest-specific error conversions

## API Differences

### Zed HTTP Client vs reqwest

| Feature | reqwest | Zed HTTP Client |
|---------|---------|----------------|
| **Async/Await** | ✅ Required | ❌ Synchronous only |
| **Status Codes** | ✅ Full support | ❌ Not available |
| **Timeout Config** | ✅ Configurable | ❌ Not available |
| **HTTP Methods** | ✅ All methods | ⚠️ No TRACE/CONNECT |
| **Response Body** | ✅ Streaming/bytes | ✅ Vec<u8> |
| **Headers** | ✅ Full access | ✅ Vec<(String, String)> |
| **WASM Support** | ❌ No | ✅ Yes |

### Request Building

**reqwest**:
```rust
let client = reqwest::Client::new();
let response = client
    .get("https://api.example.com")
    .header("Authorization", "Bearer token")
    .send()
    .await?;
```

**Zed HTTP Client**:
```rust
let request = http_client::HttpRequest::builder()
    .method(http_client::HttpMethod::Get)
    .url("https://api.example.com")
    .header("Authorization", "Bearer token")
    .build()?;

let response = request.fetch()?;
```

## Known Limitations

### 1. No HTTP Status Codes ⚠️

**Impact**: Cannot distinguish between different HTTP responses
- All successful requests return `200 OK` in the UI
- Cannot show actual `404`, `500`, `201`, etc. status codes
- Error detection relies on request failure (network errors only)

**Workaround**: Check response body content for error indicators

### 2. No Timeout Configuration ⚠️

**Impact**: Cannot set request timeouts
- `ExecutionConfig::timeout` parameter is ignored
- Requests use Zed's default timeout
- Long-running requests may hang

**Workaround**: None currently available

### 3. No Detailed Timing Metrics ⚠️

**Impact**: Cannot measure request phases
- No DNS lookup time
- No TCP connection time
- No TLS handshake time
- Only total duration is measurable

**Workaround**: Only display total request duration

### 4. Limited HTTP Methods ⚠️

**Impact**: Some HTTP methods are not supported
- `TRACE` method returns `UnsupportedMethod` error
- `CONNECT` method returns `UnsupportedMethod` error
- All other standard methods work: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS

**Workaround**: Return clear error message for unsupported methods

### 5. Synchronous Execution Only

**Impact**: No async/await syntax
- All operations block until complete
- Cannot cancel in-flight requests
- No progress callbacks

**Benefit**: Simpler code without async complexity

## Build Instructions

### Before (Would Fail)

```bash
cargo build --target wasm32-wasip1 --release
# Error: tokio features not supported on wasm
```

### After (Success)

```bash
# Add WASM target (one-time setup)
rustup target add wasm32-wasip1

# Build for Zed
cargo build --target wasm32-wasip1 --release

# Output: target/wasm32-wasip1/release/rest_client.wasm
```

## Testing

### Unit Tests

```bash
# Run standard unit tests
cargo test
```

**Note**: Tests using `http_client` will not work in standard test environment (WASM runtime required)

### Integration Tests

Must be tested manually in Zed:
1. Build the extension
2. Install in Zed
3. Create `.http` file
4. Execute requests
5. Verify responses

## Files Modified

1. **`Cargo.toml`**
   - Removed `reqwest` and `tokio`
   - Updated `zed_extension_api` to 0.7.0

2. **`src/executor/mod.rs`**
   - Removed async/await
   - Replaced reqwest with Zed HTTP client
   - Added workarounds for missing status codes

3. **`src/executor/error.rs`**
   - Added `UnsupportedMethod` variant
   - Removed reqwest error conversions

4. **`src/commands.rs`**
   - Removed `.await` from execute_request call

## Files Created

1. **`docs/WASM_BUILD.md`**
   - Comprehensive build guide
   - WASM compatibility requirements
   - Troubleshooting guide

2. **`docs/MIGRATION_SUMMARY.md`** (this file)
   - Migration overview
   - API differences
   - Known limitations

## Success Metrics

- ✅ Builds successfully with `wasm32-wasip1` target
- ✅ No tokio/reqwest dependencies
- ✅ All existing functionality preserved (except limitations)
- ✅ Clean compilation with no errors
- ✅ Proper error handling for unsupported features

## Next Steps

### For Extension Users

1. Read [WASM_BUILD.md](WASM_BUILD.md) for build instructions
2. Be aware of limitations (especially no status codes)
3. Test the extension in Zed
4. Report issues or unexpected behavior

### For Contributors

1. Never add `tokio` or `reqwest` dependencies
2. Use `zed_extension_api::http_client` for all HTTP operations
3. Keep code synchronous (no async/await)
4. Document any new limitations discovered
5. Consider requesting new features from Zed team

### For Zed Team (Feature Requests)

To improve the HTTP client API, consider adding:

1. **Status code support**: Add `status: u16` field to `HttpResponse`
2. **Timeout configuration**: Allow extensions to set request timeouts
3. **Detailed timing**: Expose DNS, TCP, TLS timing metrics
4. **Progress callbacks**: For large uploads/downloads
5. **Request cancellation**: Ability to abort in-flight requests
6. **Additional methods**: Support for TRACE and CONNECT

## Conclusion

The migration to Zed's HTTP client API successfully enables WASM compatibility, allowing the REST Client extension to run in Zed. While there are limitations compared to reqwest, the core functionality is preserved and the extension is now properly integrated with Zed's extension ecosystem.

The main tradeoff is the loss of HTTP status code visibility, which is a significant limitation for a REST client. This should be clearly communicated to users and may warrant a feature request to the Zed team to enhance the HTTP client API.