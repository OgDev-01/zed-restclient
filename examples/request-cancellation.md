# Request Cancellation

This document demonstrates how to use the request cancellation feature in the REST Client extension.

## Overview

The REST Client now supports canceling in-flight HTTP requests. This is useful for:
- Long-running requests that you want to abort
- Accidentally triggered requests
- Requests that are taking too long to respond

## Architecture

The cancellation system uses the following components:

- **RequestHandle**: Tracks individual requests with a unique ID and cancellation flag
- **RequestTracker**: Manages all active requests in a thread-safe HashMap
- **Global Tracker**: A singleton that tracks all requests across the extension
- **Cancellation API**: Functions to cancel specific requests or the most recent one

## Usage

### Canceling the Most Recent Request

The easiest way to cancel a request is to use the "Cancel Request" command:

```
Command Palette → REST Client: Cancel Request
```

This will cancel the most recently started request that is still in-flight.

### Programmatic Cancellation

You can also cancel requests programmatically in Rust code:

```rust
use rest_client::executor::{
    execute_request_with_cancellation,
    cancel_request,
    cancel_most_recent_request,
    get_active_request_count,
};
use rest_client::commands::cancel_request_command;

// Execute a request with cancellation support
let (response, request_id) = execute_request_with_cancellation(&request, &config)?;

// Cancel a specific request by ID
cancel_request(&request_id)?;

// Cancel the most recent request
let cancelled_id = cancel_most_recent_request()?;

// Get count of active requests
let count = get_active_request_count();

// Use the command wrapper (user-friendly)
let result = cancel_request_command();
if result.success {
    println!("Cancelled: {}", result.message);
}
```

## How It Works

### Request Registration

When a request is executed with `execute_request_with_cancellation()`:

1. A unique request ID is generated using UUID v4
2. A `RequestHandle` is created with a cancellation flag
3. The handle is registered in the global `RequestTracker`
4. The request executes and checks the cancellation flag periodically
5. When complete (or cancelled), the request is unregistered from the tracker

### Cancellation Process

When `cancel_request()` or `cancel_most_recent_request()` is called:

1. The tracker looks up the request by ID (or finds the most recent)
2. The cancellation flag is set to `true`
3. The request handle is removed from the tracker
4. The executing request detects the flag and returns an error

### Cancellation Check Points

The executor checks for cancellation at these points:

- Before starting the request
- After URL validation
- Before building the HTTP request
- Before executing the network call
- After receiving the response

This ensures that cancellation is detected quickly at multiple stages.

## Race Condition Handling

The cancellation system handles several race conditions gracefully:

### Request Completes While Cancelling

If a request finishes naturally while a cancellation is being processed:

```rust
// Thread 1: Request completes
let _ = tracker.unregister(&request_id);

// Thread 2: Tries to cancel
let result = tracker.cancel_request(&request_id);
// Returns Err(CancelError::NotFound)
```

The cancellation attempt fails with `NotFound`, which is expected behavior.

### Multiple Cancellation Attempts

If the same request is cancelled multiple times:

```rust
cancel_request("req-123")?; // Success
cancel_request("req-123")?; // Err(CancelError::NotFound)
```

The first cancellation succeeds, subsequent attempts return `NotFound`.

### Concurrent Request Execution

The `SharedRequestTracker` uses `Arc<Mutex<>>` to ensure thread-safety:

```rust
// Multiple threads can safely register/cancel requests
let tracker = SharedRequestTracker::new();
tracker.register(handle1)?; // Thread 1
tracker.register(handle2)?; // Thread 2
tracker.cancel_request("req-1")?; // Thread 3
```

## Error Handling

Cancellation operations can return these errors:

```rust
pub enum CancelError {
    /// Request with the given ID was not found
    NotFound(String),
    
    /// Request has already completed
    AlreadyCompleted(String),
    
    /// Failed to acquire lock on tracker
    LockError(String),
}
```

### Handling Cancellation Errors

```rust
match cancel_request(&request_id) {
    Ok(()) => println!("Request cancelled successfully"),
    Err(CancelError::NotFound(id)) => {
        println!("Request {} not found (may have already completed)", id);
    }
    Err(CancelError::AlreadyCompleted(id)) => {
        println!("Request {} already completed", id);
    }
    Err(CancelError::LockError(msg)) => {
        eprintln!("Lock error: {}", msg);
    }
}
```

## Status Messages

When a request is cancelled, the user sees:

```
✗ Request cancelled (ID: 550e8400-e29b-41d4-a716-446655440000)
```

If there are other active requests:

```
✗ Request cancelled successfully (ID: 550e8400-e29b-41d4-a716-446655440000)
  2 request(s) still active
```

## Memory Management

The cancellation system prevents memory leaks through:

### Automatic Cleanup

Requests are automatically unregistered when they:
- Complete successfully
- Fail with an error
- Are cancelled

### Manual Cleanup

You can manually clean up completed requests:

```rust
use rest_client::executor::SharedRequestTracker;

let tracker = SharedRequestTracker::new();

// Removes all requests marked as cancelled
let cleaned_count = tracker.cleanup_completed()?;
println!("Cleaned up {} completed requests", cleaned_count);
```

### Best Practices

1. **Always use `execute_request_with_cancellation`** for requests that may need to be cancelled
2. **Don't hold request IDs indefinitely** - they're only valid while the request is active
3. **Check active count** before assuming requests can be cancelled
4. **Handle `NotFound` errors gracefully** - they're expected when requests complete naturally

## Limitations

### WASM Environment

In the current Zed extension WASM environment:

- Full tokio task cancellation is not available (tokio is only in dev-dependencies)
- Cancellation uses a flag-based approach instead of `JoinHandle::abort()`
- Network calls cannot be interrupted mid-flight (checked between operations)

### Future Enhancements

When Zed's extension API supports async/tokio:

- True task-based cancellation with `tokio::task::JoinHandle`
- Ability to abort network calls in progress
- More granular cancellation control

## Testing

The cancellation system includes comprehensive tests:

```bash
# Run all cancellation tests
cargo test --lib cancellation

# Run specific test
cargo test --lib test_tracker_cancel_most_recent
```

Test coverage includes:
- Request handle creation and lifecycle
- Tracker registration/unregistration
- Cancellation of specific requests
- Cancellation of most recent request
- Error handling for non-existent requests
- Thread safety with `SharedRequestTracker`
- Cleanup of completed requests
- Race condition scenarios

## Example: Long-Running Request

```http
### Long-running request (can be cancelled)
GET https://httpbin.org/delay/30
```

1. Position cursor in the request block
2. Execute: `REST Client: Send Request`
3. While waiting, execute: `REST Client: Cancel Request`
4. The request will be aborted and show "Request cancelled"

## Summary

The request cancellation feature provides:

✓ Unique request IDs for tracking
✓ Thread-safe request management
✓ Graceful race condition handling
✓ Automatic memory cleanup
✓ User-friendly command interface
✓ Comprehensive error handling
✓ Detailed status messages

Use it to improve user experience and give users control over their HTTP requests!