//! Benchmarks for the HTTP request parser.
//!
//! These benchmarks measure the performance of parsing .http files of various sizes
//! to ensure we meet the requirement of <100ms for files up to 10,000 lines.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rest_client::parser::parse_file;
use std::path::PathBuf;

/// Generate a synthetic .http file with the specified number of requests.
fn generate_http_file(num_requests: usize) -> String {
    let mut content = String::new();

    for i in 0..num_requests {
        content.push_str(&format!(
            "### Request {}\n\
             GET https://api.example.com/users/{}\n\
             Authorization: Bearer token-{}\n\
             Accept: application/json\n\
             User-Agent: RestClient/1.0\n\
             X-Request-ID: {}\n\
             \n\
             ###\n\
             \n",
            i, i, i, i
        ));
    }

    content
}

/// Generate a complex .http file with various request types.
fn generate_complex_http_file(num_requests: usize) -> String {
    let mut content = String::new();

    for i in 0..num_requests {
        let method = match i % 5 {
            0 => "GET",
            1 => "POST",
            2 => "PUT",
            3 => "DELETE",
            _ => "PATCH",
        };

        let has_body = matches!(method, "POST" | "PUT" | "PATCH");

        content.push_str(&format!(
            "### Request {} - {}\n\
             {} https://api.example.com/resource/{}\n\
             Authorization: Bearer token-{}\n\
             Content-Type: application/json\n\
             Accept: application/json\n\
             X-Correlation-ID: correlation-{}\n",
            i, method, method, i, i, i
        ));

        if has_body {
            content.push_str(&format!(
                "\n\
                 {{\n\
                   \"id\": {},\n\
                   \"name\": \"Resource {}\",\n\
                   \"description\": \"This is a test resource with ID {}\",\n\
                   \"metadata\": {{\n\
                     \"created_at\": \"2025-01-01T00:00:00Z\",\n\
                     \"updated_at\": \"2025-01-01T00:00:00Z\",\n\
                     \"version\": {}\n\
                   }}\n\
                 }}\n",
                i, i, i, i
            ));
        }

        content.push_str("\n###\n\n");
    }

    content
}

/// Benchmark parsing small files (10 requests, ~100 lines).
fn bench_parse_small(c: &mut Criterion) {
    let content = generate_http_file(10);
    let path = PathBuf::from("bench_small.http");

    c.bench_function("parse_small_10_requests", |b| {
        b.iter(|| parse_file(black_box(&content), black_box(&path)).unwrap())
    });
}

/// Benchmark parsing medium files (100 requests, ~1000 lines).
fn bench_parse_medium(c: &mut Criterion) {
    let content = generate_http_file(100);
    let path = PathBuf::from("bench_medium.http");

    c.bench_function("parse_medium_100_requests", |b| {
        b.iter(|| parse_file(black_box(&content), black_box(&path)).unwrap())
    });
}

/// Benchmark parsing large files (1000 requests, ~10,000 lines).
fn bench_parse_large(c: &mut Criterion) {
    let content = generate_http_file(1000);
    let path = PathBuf::from("bench_large.http");

    let mut group = c.benchmark_group("parse_large");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("parse_large_1000_requests", |b| {
        b.iter(|| parse_file(black_box(&content), black_box(&path)).unwrap())
    });

    group.finish();
}

/// Benchmark parsing very large files (5000 requests, ~50,000 lines).
fn bench_parse_very_large(c: &mut Criterion) {
    let content = generate_http_file(5000);
    let path = PathBuf::from("bench_very_large.http");

    let mut group = c.benchmark_group("parse_very_large");
    group.throughput(Throughput::Elements(5000));
    group.sample_size(10); // Reduce sample size for large files

    group.bench_function("parse_very_large_5000_requests", |b| {
        b.iter(|| parse_file(black_box(&content), black_box(&path)).unwrap())
    });

    group.finish();
}

/// Benchmark parsing complex files with various request types and bodies.
fn bench_parse_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_complex");

    for size in [10, 100, 500, 1000].iter() {
        let content = generate_complex_http_file(*size);
        let path = PathBuf::from(format!("bench_complex_{}.http", size));

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_requests", size)),
            size,
            |b, _| b.iter(|| parse_file(black_box(&content), black_box(&path)).unwrap()),
        );
    }

    group.finish();
}

/// Benchmark parsing files with comments and whitespace.
fn bench_parse_with_comments(c: &mut Criterion) {
    let mut content = String::new();

    for i in 0..500 {
        content.push_str(&format!(
            "# This is a comment for request {}\n\
             // Another comment style\n\
             ### Request {}\n\
             \n\
             # Method and URL\n\
             GET https://api.example.com/resource/{}\n\
             \n\
             # Headers\n\
             Authorization: Bearer token-{}\n\
             Accept: application/json\n\
             \n\
             ###\n\
             \n",
            i, i, i, i
        ));
    }

    let path = PathBuf::from("bench_with_comments.http");

    c.bench_function("parse_with_comments_500_requests", |b| {
        b.iter(|| parse_file(black_box(&content), black_box(&path)).unwrap())
    });
}

/// Benchmark parsing files with variables (to measure variable detection overhead).
fn bench_parse_with_variables(c: &mut Criterion) {
    let mut content = String::new();

    for i in 0..500 {
        content.push_str(&format!(
            "### Request {}\n\
             GET {{{{baseUrl}}}}/resource/{{{{resourceId}}}}\n\
             Authorization: Bearer {{{{authToken}}}}\n\
             X-User-ID: {{{{userId}}}}\n\
             Accept: application/json\n\
             \n\
             ###\n\
             \n",
            i
        ));
    }

    let path = PathBuf::from("bench_with_variables.http");

    c.bench_function("parse_with_variables_500_requests", |b| {
        b.iter(|| parse_file(black_box(&content), black_box(&path)).unwrap())
    });
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large,
    bench_parse_very_large,
    bench_parse_complex,
    bench_parse_with_comments,
    bench_parse_with_variables
);

criterion_main!(benches);
