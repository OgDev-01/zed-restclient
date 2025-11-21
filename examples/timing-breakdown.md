# Request Timing Breakdown

The REST Client extension provides detailed performance metrics for each HTTP request, breaking down the total response time into individual phases.

## Overview

When you execute a request, the extension measures and displays timing information for:

- **DNS Lookup**: Time spent resolving the domain name to an IP address
- **TCP Connection**: Time spent establishing the TCP connection to the server
- **TLS Handshake**: Time spent performing SSL/TLS negotiation (HTTPS only)
- **First Byte (TTFB)**: Time from request sent to receiving the first response byte (server processing + network latency)
- **Download**: Time spent downloading the response body

## Example Output

```
HTTP/1.1 200 OK

Headers:
Content-Type: application/json
Content-Length: 1234

Duration: 210ms | Size: 1.23 KB | Type: application/json
Timing: DNS: 10ms | TCP: 20ms | TLS: 50ms | First Byte: 30ms | Download: 100ms

---
{
  "status": "success",
  "data": {...}
}
```

## Understanding the Metrics

### DNS Lookup
- **What it measures**: Time to resolve the hostname to an IP address
- **Typical values**: 10-100ms for first request, <1ms for cached
- **High values indicate**: DNS server issues, network latency, or missing DNS cache

### TCP Connection
- **What it measures**: Time to establish a TCP connection (3-way handshake)
- **Typical values**: 10-50ms for nearby servers, 100-300ms for distant servers
- **High values indicate**: Network congestion, geographical distance, or server load

### TLS Handshake (HTTPS only)
- **What it measures**: Time to negotiate SSL/TLS encryption
- **Typical values**: 50-200ms for first connection, <1ms for session resumption
- **High values indicate**: Weak server CPU, old TLS versions, or complex certificate chains

### First Byte (Time to First Byte - TTFB)
- **What it measures**: Server processing time + network latency
- **Typical values**: 20-500ms depending on endpoint complexity
- **High values indicate**: Slow server processing, database queries, or backend issues

### Download
- **What it measures**: Time to transfer the response body
- **Typical values**: Depends on response size and bandwidth
- **High values indicate**: Large response, slow connection, or bandwidth throttling

## Implementation Details

### Measurement Limitations

Due to the Zed HTTP client API limitations, precise timing for each phase may not be available. The extension uses the following approach:

1. **Direct Measurement** (when possible):
   - Total request duration (start to finish)
   - Request preparation time
   - Response processing time

2. **Estimation** (when detailed hooks unavailable):
   - Phase breakdown is estimated based on typical network behavior
   - Connection setup phases (DNS + TCP + TLS) are estimated as ~40% of total time for HTTPS
   - For HTTPS: DNS ≈ 15%, TCP ≈ 25%, TLS ≈ 60% of connection time
   - For HTTP: DNS ≈ 40%, TCP ≈ 60% of connection time

### Precision

- All timings use microsecond precision (`Duration` with `as_micros()`)
- Display format automatically adjusts:
  - Values < 1ms: displayed in microseconds (μs)
  - Values < 1s: displayed in milliseconds (ms)
  - Values ≥ 1s: displayed in seconds (s) with 3 decimal places

### Accuracy

The timing breakdown provides:
- **Accurate total duration**: Measured from request start to completion
- **Estimated phase breakdown**: Based on common network patterns
- **Sum validation**: All phases sum to total duration (within margin)

## Use Cases

### Performance Debugging

Identify bottlenecks in your API requests:

```
Slow DNS (>100ms):
- Check DNS provider performance
- Consider using DNS caching
- Verify network connectivity

Slow TCP (>200ms):
- Server may be geographically distant
- Network congestion
- Consider using a CDN

Slow TLS (>300ms):
- Server using old TLS version
- Weak server CPU
- Complex certificate chain

Slow TTFB (>1s):
- Backend processing issues
- Database query optimization needed
- Consider caching strategies

Slow Download (relative to size):
- Bandwidth limitations
- Response compression not enabled
- Consider pagination for large datasets
```

### API Comparison

Compare performance across different endpoints or servers:

```
Production:
Timing: DNS: 5ms | TCP: 15ms | TLS: 40ms | First Byte: 80ms | Download: 20ms
Total: 160ms

Staging:
Timing: DNS: 5ms | TCP: 15ms | TLS: 40ms | First Byte: 450ms | Download: 20ms
Total: 530ms

Analysis: Staging server has 370ms slower TTFB → backend performance issue
```

### Geographic Performance Testing

Test API performance from different regions:

```
Local (same datacenter):
TCP: 2ms | TLS: 30ms | TTFB: 50ms

Cross-region (US → EU):
TCP: 120ms | TLS: 180ms | TTFB: 200ms

Analysis: Cross-region adds ~298ms of network overhead
```

## Best Practices

1. **Run multiple requests**: Single measurements may be affected by cold starts
2. **Compare trends**: Look for patterns across similar requests
3. **Consider caching**: First request to a domain will have higher DNS/TLS times
4. **Check total vs. phases**: If phases don't match total, API limitations may be estimating
5. **Use for relative comparison**: Even estimated timing helps compare relative performance

## API Reference

### Timing Functions

```rust
use rest_client::executor::timing::{format_timing_breakdown, TimingCheckpoints};
use rest_client::models::response::RequestTiming;

// Format timing for display
let breakdown = format_timing_breakdown(&timing);
// Returns: "DNS: 10ms | TCP: 20ms | TLS: 50ms | First Byte: 30ms | Download: 100ms"

// Create timing checkpoints during request execution
let mut checkpoints = TimingCheckpoints::new(is_https);
checkpoints.mark_client_start();
checkpoints.mark_request_sent();
checkpoints.mark_first_byte_received();
checkpoints.mark_response_complete();

// Convert to RequestTiming
let timing = checkpoints.to_request_timing();
```

### RequestTiming Structure

```rust
pub struct RequestTiming {
    pub dns_lookup: Duration,
    pub tcp_connection: Duration,
    pub tls_handshake: Option<Duration>,  // None for HTTP
    pub first_byte: Duration,
    pub download: Duration,
}
```

## See Also

- [Response Formatting](./response-formatting.md) - How responses are formatted and displayed
- [Request Execution](./request-execution.md) - How requests are executed
- [Environment Variables](../USAGE.md#variables-and-environments) - Using variables in requests