# Request History Module

This module provides persistent storage and management of HTTP request/response history for the REST Client extension.

## Overview

The history module allows users to:
- Save request/response pairs after each execution
- Load and review past requests
- Re-execute previous requests
- Search and filter history entries
- Manage storage limits and cleanup

## Architecture

The module is organized into two main components:

### `models.rs` - Data Models

Defines the core data structures:

- **`HistoryEntry`**: Represents a single request/response pair with metadata
  - `id`: Unique identifier (UUID v4)
  - `timestamp`: When the request was executed (UTC)
  - `request`: The HTTP request that was sent
  - `response`: The HTTP response received
  - `tags`: User-defined tags for organization

- **`HistoryError`**: Error types for history operations
  - `StorageError`: File I/O errors
  - `SerializationError`: JSON parsing errors
  - `QuotaExceeded`: History limit exceeded

### `storage.rs` - Persistence Layer

Handles reading and writing history to disk:

- **File Format**: JSONL (JSON Lines) - one JSON object per line
- **Location**: `~/.config/zed/extensions/rest-client/history.json`
- **Default Limit**: 1000 entries (configurable)

## Features

### 1. Automatic Sanitization

Sensitive headers are automatically removed before storage (can be disabled):

```rust
// Sanitized headers:
- Authorization
- Cookie
- Set-Cookie
- X-Api-Key
- Bearer tokens
- Access tokens
```

### 2. Large Response Handling

Response bodies larger than 1MB are automatically excluded to save storage:

```rust
const MAX_RESPONSE_BODY_SIZE: usize = 1_048_576; // 1MB
```

### 3. Selective Storage

By default, only successful requests (2xx, 3xx) are saved. Failed requests can be optionally stored:

```rust
let config = HistoryConfig {
    save_failed_requests: true,
    ..Default::default()
};
```

### 4. Graceful Corruption Handling

If the history file becomes corrupted:
- Invalid lines are skipped
- Valid entries are still loaded
- Use `rebuild_history()` to clean up

## Usage

### Basic Operations

```rust
use history::{HistoryEntry, save_entry, load_history};

// Save a history entry
let entry = HistoryEntry::new(request, response);
save_entry(&entry)?;

// Load all history
let entries = load_history()?;

// Maintain history limit
maintain_history_limit(1000)?;

// Clear all history
clear_history()?;
```

### Configuration

```rust
use history::{HistoryConfig, save_entry_with_config};

let config = HistoryConfig {
    max_entries: 500,
    sanitize_sensitive_headers: false, // Keep sensitive data
    save_failed_requests: true,        // Save 4xx/5xx responses
};

save_entry_with_config(&entry, &config)?;
```

### Working with Tags

```rust
let mut entry = HistoryEntry::new(request, response);

// Add tags
entry.add_tag("production".to_string());
entry.add_tag("user-api".to_string());

// Check tags
if entry.has_tag("production") {
    println!("Production request");
}

// Remove tags
entry.remove_tag("production");
```

### Custom Preparation

```rust
// Prepare entry for storage manually
let sanitized_entry = entry.sanitize_headers(true);
let truncated_entry = entry.truncate_large_response();

// Or use the all-in-one method
let prepared_entry = entry.prepare_for_storage(true);
```

## File Format

The history file uses JSONL format (JSON Lines):

```json
{"id":"uuid-1","timestamp":"2025-01-01T10:00:00Z","request":{...},"response":{...},"tags":[]}
{"id":"uuid-2","timestamp":"2025-01-01T10:05:00Z","request":{...},"response":{...},"tags":["api"]}
{"id":"uuid-3","timestamp":"2025-01-01T10:10:00Z","request":{...},"response":{...},"tags":[]}
```

**Advantages of JSONL:**
- Efficient append operations (no need to rewrite entire file)
- Easy recovery from corruption (skip invalid lines)
- Simple streaming reads for large files
- Human-readable and debuggable

## Storage Location

| Platform | Default Path |
|----------|-------------|
| Linux/macOS | `~/.config/zed/extensions/rest-client/history.json` |
| Windows | `%USERPROFILE%\AppData\Roaming\zed\extensions\rest-client\history.json` |

The directory is created automatically if it doesn't exist.

## Performance Considerations

1. **Append-Only Writes**: New entries are appended without reading the entire file
2. **Lazy Loading**: History is loaded on demand, not at startup
3. **Automatic Cleanup**: `maintain_history_limit()` keeps file size manageable
4. **Streaming Reads**: Large history files are read line-by-line

## Error Handling

All operations return `Result<T, HistoryError>`:

```rust
match save_entry(&entry) {
    Ok(_) => println!("Entry saved successfully"),
    Err(HistoryError::StorageError(e)) => {
        eprintln!("Failed to write to disk: {}", e);
    }
    Err(HistoryError::SerializationError(e)) => {
        eprintln!("Failed to serialize entry: {}", e);
    }
    Err(HistoryError::QuotaExceeded { current, max }) => {
        eprintln!("History full: {}/{} entries", current, max);
    }
}
```

## Testing

The module includes comprehensive tests for:

- ✅ Save and load operations
- ✅ Multiple entries
- ✅ History limits and maintenance
- ✅ Sensitive header sanitization
- ✅ Large response truncation
- ✅ Tag management
- ✅ Configuration options
- ✅ Error handling

Run tests with:

```bash
cargo test history
```

## Future Enhancements

Potential improvements for future versions:

- [ ] Search and filter API
- [ ] Export/import functionality
- [ ] Compression for old entries
- [ ] Indexing for faster searches
- [ ] Encryption option for sensitive data
- [ ] History statistics and analytics
- [ ] Automatic backup/sync

## Integration

This module is designed to integrate with:

1. **Request Executor**: Automatically save entries after execution
2. **Command Layer**: Provide history viewing and management commands
3. **UI Layer**: Display history in panels or quick-pick lists

See `Task 17: Request History UI and Commands` for user-facing features.

## Security Notes

⚠️ **Important Security Considerations:**

1. **Sensitive Data**: Even with sanitization enabled, request/response bodies may contain sensitive data
2. **File Permissions**: The history file is stored with default user permissions
3. **Plaintext Storage**: All data is stored unencrypted
4. **Disable for Production**: Consider disabling history for production environments

To disable history:

```rust
// Don't save the entry
if !is_production_env() {
    save_entry(&entry)?;
}
```

## License

This module is part of the REST Client extension and follows the same license (MIT OR Apache-2.0).