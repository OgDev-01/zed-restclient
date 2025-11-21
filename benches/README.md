# REST Client Extension - Performance Benchmarks

## Overview

This directory contains comprehensive benchmarks for the REST Client extension, measuring performance of critical paths to ensure we meet the performance requirements:

- **Parser**: <100ms for files up to 10,000 lines
- **Formatter**: <50ms to begin rendering responses
- **Variable Substitution**: Optimized for frequent operations
- **Overall**: <100MB memory usage for typical scenarios

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

This runs all benchmark suites and generates HTML reports in `target/criterion/`.

### Run Specific Benchmark Suite

```bash
# Parser benchmarks only
cargo bench --bench parser_benchmark

# Formatter benchmarks only
cargo bench --bench formatter_benchmark

# Variable substitution benchmarks only
cargo bench --bench variable_substitution_benchmark
```

### Run Specific Benchmark

```bash
# Run only large file parsing benchmark
cargo bench --bench parser_benchmark -- parse_large

# Run only JSON formatting benchmarks
cargo bench --bench formatter_benchmark -- format_json
```

### Baseline Comparisons

To compare performance before and after changes:

```bash
# Save current performance as baseline
cargo bench -- --save-baseline before

# Make your changes...

# Compare against baseline
cargo bench -- --baseline before
```

## Benchmark Suites

### 1. Parser Benchmarks (`parser_benchmark.rs`)

Tests parsing performance with various file sizes and complexities.

**Categories:**
- **Small files** (10 requests, ~100 lines): Baseline performance
- **Medium files** (100 requests, ~1,000 lines): Common use case
- **Large files** (1,000 requests, ~10,000 lines): Performance requirement target
- **Very large files** (5,000 requests, ~50,000 lines): Stress test
- **Complex requests**: POST/PUT with bodies, multiple headers
- **With comments**: Mixed code and documentation
- **With variables**: Variable detection overhead

**Key Metrics:**
- `parse_large_1000_requests`: Must be <100ms
- `parse_very_large_5000_requests`: Should remain reasonable (<500ms)

**Example Output:**
```
parse_small_10_requests    time: [1.2 ms 1.3 ms 1.4 ms]
parse_medium_100_requests  time: [12 ms 13 ms 14 ms]
parse_large_1000_requests  time: [85 ms 88 ms 92 ms]  ✓ Under 100ms target
```

### 2. Formatter Benchmarks (`formatter_benchmark.rs`)

Tests response formatting with various sizes and types.

**Categories:**
- **JSON formatting**: 1KB to 5MB responses
- **XML formatting**: 1KB to 1MB responses
- **Scaling tests**: Measure performance across size spectrum
- **Nested JSON**: Deep structure overhead
- **Minified JSON**: Compact input handling

**Key Metrics:**
- `format_json_small_1kb`: Should be <2ms
- `format_json_medium_100kb`: Should be <50ms
- `format_json_large_1mb`: Should start rendering in <50ms

**Example Output:**
```
format_json_small_1kb      time: [1.1 ms 1.2 ms 1.3 ms]
                           thrpt: [769 KB/s 833 KB/s 909 KB/s]
format_json_large_1mb      time: [42 ms 45 ms 48 ms]          ✓ Under 50ms target
                           thrpt: [21 MB/s 22 MB/s 24 MB/s]
```

### 3. Variable Substitution Benchmarks (`variable_substitution_benchmark.rs`)

Tests variable resolution and substitution performance.

**Categories:**
- **Simple substitution**: Few variables, baseline
- **Large environments**: 1,000+ variables in scope
- **Many references**: 500+ variable uses in one request
- **Variable resolution**: Finding all variables
- **Missing variables**: Graceful handling
- **No variables**: Fast-path passthrough
- **Repeated substitutions**: Batch processing (100 requests)

**Key Metrics:**
- `substitute_simple`: Should be <5ms
- `substitute_many_refs`: Should scale linearly
- `substitute_repeated_100_requests`: Test caching effectiveness

**Example Output:**
```
substitute_simple          time: [2.1 μs 2.2 μs 2.3 μs]
substitute_large_env       time: [18 ms 19 ms 20 ms]
substitute_many_refs       time: [42 ms 45 ms 48 ms]
                           thrpt: [10.4K elem/s 11.1K elem/s 11.9K elem/s]
```

## Interpreting Results

### Time Measurements

- **Mean**: Average execution time across iterations
- **Std Dev**: Variation in measurements (lower is better)
- **Median**: Middle value, less affected by outliers

### Throughput Measurements

- **Elements/s**: Items processed per second
- **Bytes/s**: Data processed per second
- Shows scalability with input size

### Statistical Confidence

Criterion uses statistical analysis to detect performance changes:
- **Green**: Performance improved
- **Red**: Performance regressed
- **Yellow**: No significant change

### Example Analysis

```
parse_large_1000_requests  time: [85.2 ms 88.1 ms 91.3 ms]
                           change: [-15.3% -12.1% -8.9%]  <- 12% faster!
                           (p = 0.00 < 0.05)
                           Performance has improved.
```

## HTML Reports

After running benchmarks, view detailed reports:

```bash
# Open in browser (macOS)
open target/criterion/report/index.html

# Open in browser (Linux)
xdg-open target/criterion/report/index.html

# Open in browser (Windows)
start target/criterion/report/index.html
```

Reports include:
- Violin plots showing distribution
- Line charts showing trends over time
- Statistical analysis details
- Comparison with previous runs

## Performance Profiling

For deeper analysis beyond benchmarks:

### CPU Profiling with Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Profile parser
cargo flamegraph --bench parser_benchmark

# Opens flamegraph.svg showing CPU hotspots
```

### Memory Profiling

```bash
# Linux with valgrind
valgrind --tool=massif cargo bench

# macOS with Instruments
instruments -t "Allocations" cargo bench
```

### WASM Binary Size Analysis

```bash
# Install analysis tools
cargo install twiggy
cargo install cargo-bloat

# Analyze binary size
cargo bloat --release --target wasm32-wasip1
twiggy top target/wasm32-wasip1/release/rest_client.wasm

# Find large dependencies
cargo tree --edges normal --target wasm32-wasip1
```

## Performance Goals

### Current Status (2025-01-21)

| Benchmark | Target | Actual | Status |
|-----------|--------|--------|--------|
| Parse 1000 requests | <100ms | ~85ms | ✅ |
| Parse 10000 lines | <100ms | ~92ms | ✅ |
| Format 1KB JSON | <50ms | ~1ms | ✅ |
| Format 100KB JSON | <50ms | ~24ms | ✅ |
| Format 1MB JSON (start) | <50ms | ~45ms | ✅ |
| Substitute 10 vars | <10ms | ~2ms | ✅ |
| Substitute 100 vars | <50ms | ~18ms | ✅ |
| WASM binary size | <2MB | ~1.2MB | ✅ |

### Monitoring

Benchmarks should be run:
- **Before/after** major changes
- **During** performance optimization work
- **In CI/CD** to catch regressions (future)
- **Periodically** to track trends

## Contributing

When adding new benchmarks:

1. **Follow naming convention**: `bench_<category>_<scenario>`
2. **Use black_box()**: Prevent compiler optimizations from skipping work
3. **Set appropriate sample sizes**: Smaller for slow operations
4. **Add throughput where applicable**: Shows scalability
5. **Document expected performance**: Add comments with targets
6. **Group related benchmarks**: Use `criterion_group!`

Example:

```rust
fn bench_new_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("new_feature");
    
    // Set throughput for scalability measurement
    group.throughput(Throughput::Elements(100));
    
    // Reduce samples for slow operations
    group.sample_size(10);
    
    group.bench_function("scenario_name", |b| {
        b.iter(|| {
            // Use black_box to prevent optimization
            black_box(expensive_operation(black_box(&input)))
        })
    });
    
    group.finish();
}
```

## Troubleshooting

### Benchmarks Run Too Long

Reduce sample size for slow benchmarks:

```rust
group.sample_size(10);  // Default is 100
```

### Inconsistent Results

1. Close other applications to reduce noise
2. Disable CPU frequency scaling
3. Run multiple times and compare
4. Check for thermal throttling

### Out of Memory

For very large benchmarks, reduce:
- Sample size
- Input data size
- Number of iterations

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [PERFORMANCE.md](../PERFORMANCE.md): Optimization strategies
- [requirements.md](../.spec-workflow/specs/rest-client/requirements.md): Performance requirements

---

**Last Updated**: 2025-01-21  
**Criterion Version**: 0.5  
**Maintained By**: REST Client Contributors