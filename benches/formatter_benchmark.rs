//! Benchmarks for the HTTP response formatter.
//!
//! These benchmarks measure the performance of formatting responses of various sizes
//! to ensure we meet the requirement of <50ms for formatting start.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rest_client::formatter::{format_json_pretty, format_xml_pretty};
use rest_client::models::response::HttpResponse;
use std::collections::HashMap;

/// Generate a large JSON object with nested structures.
fn generate_large_json(size_kb: usize) -> String {
    let num_items = (size_kb * 1024) / 200; // Approximate items to reach size
    let mut items = Vec::new();

    for i in 0..num_items {
        items.push(format!(
            r#"{{
                "id": {},
                "name": "Item {}",
                "description": "This is a detailed description for item {} with additional metadata",
                "timestamp": "2025-01-01T{}:{}:{}.000Z",
                "metadata": {{
                    "created_by": "user-{}",
                    "updated_by": "user-{}",
                    "version": {},
                    "tags": ["tag1", "tag2", "tag3", "tag4"],
                    "properties": {{
                        "color": "blue",
                        "size": "large",
                        "weight": 123.45,
                        "enabled": true
                    }}
                }},
                "relationships": {{
                    "parent_id": {},
                    "child_ids": [{}, {}, {}]
                }}
            }}"#,
            i,
            i,
            i,
            (i % 24),
            (i % 60),
            (i % 60),
            i,
            i,
            i,
            i.saturating_sub(1),
            i + 1,
            i + 2,
            i + 3
        ));
    }

    format!("[{}]", items.join(","))
}

/// Generate a deeply nested JSON object.
fn generate_nested_json(depth: usize) -> String {
    let mut json = String::from(r#"{"data":"#);
    for _ in 0..depth {
        json.push_str(r#"{"nested":"#);
    }
    json.push_str(r#""value""#);
    for _ in 0..depth {
        json.push('}');
    }
    json.push('}');
    json
}

/// Generate a large XML document.
fn generate_large_xml(size_kb: usize) -> String {
    let num_items = (size_kb * 1024) / 300; // Approximate items to reach size
    let mut items = String::new();

    for i in 0..num_items {
        items.push_str(&format!(
            r#"<item>
                <id>{}</id>
                <name>Item {}</name>
                <description>This is a detailed description for item {}</description>
                <metadata>
                    <created_by>user-{}</created_by>
                    <updated_by>user-{}</updated_by>
                    <version>{}</version>
                    <tags>
                        <tag>tag1</tag>
                        <tag>tag2</tag>
                        <tag>tag3</tag>
                    </tags>
                </metadata>
                <properties>
                    <color>blue</color>
                    <size>large</size>
                    <weight>123.45</weight>
                    <enabled>true</enabled>
                </properties>
            </item>
            "#,
            i, i, i, i, i, i
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><items>{}</items>"#,
        items
    )
}

/// Benchmark formatting small JSON responses (1KB).
fn bench_format_json_small(c: &mut Criterion) {
    let json = generate_large_json(1);

    c.bench_function("format_json_small_1kb", |b| {
        b.iter(|| format_json_pretty(black_box(&json)))
    });
}

/// Benchmark formatting medium JSON responses (100KB).
fn bench_format_json_medium(c: &mut Criterion) {
    let json = generate_large_json(100);

    let mut group = c.benchmark_group("format_json_medium");
    group.throughput(Throughput::Bytes(json.len() as u64));

    group.bench_function("format_json_medium_100kb", |b| {
        b.iter(|| format_json_pretty(black_box(&json)))
    });

    group.finish();
}

/// Benchmark formatting large JSON responses (1MB).
fn bench_format_json_large(c: &mut Criterion) {
    let json = generate_large_json(1024);

    let mut group = c.benchmark_group("format_json_large");
    group.throughput(Throughput::Bytes(json.len() as u64));
    group.sample_size(10); // Reduce sample size for large responses

    group.bench_function("format_json_large_1mb", |b| {
        b.iter(|| format_json_pretty(black_box(&json)))
    });

    group.finish();
}

/// Benchmark formatting very large JSON responses (5MB).
fn bench_format_json_very_large(c: &mut Criterion) {
    let json = generate_large_json(5 * 1024);

    let mut group = c.benchmark_group("format_json_very_large");
    group.throughput(Throughput::Bytes(json.len() as u64));
    group.sample_size(10);

    group.bench_function("format_json_very_large_5mb", |b| {
        b.iter(|| format_json_pretty(black_box(&json)))
    });

    group.finish();
}

/// Benchmark formatting JSON with varying sizes.
fn bench_format_json_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_json_scaling");

    for size_kb in [1, 10, 50, 100, 500, 1000].iter() {
        let json = generate_large_json(*size_kb);

        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}kb", size_kb)),
            size_kb,
            |b, _| b.iter(|| format_json_pretty(black_box(&json))),
        );
    }

    group.finish();
}

/// Benchmark formatting deeply nested JSON.
fn bench_format_json_nested(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_json_nested");

    for depth in [10, 50, 100, 200].iter() {
        let json = generate_nested_json(*depth);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth_{}", depth)),
            depth,
            |b, _| b.iter(|| format_json_pretty(black_box(&json))),
        );
    }

    group.finish();
}

/// Benchmark formatting small XML responses (1KB).
fn bench_format_xml_small(c: &mut Criterion) {
    let xml = generate_large_xml(1);

    c.bench_function("format_xml_small_1kb", |b| {
        b.iter(|| format_xml_pretty(black_box(&xml)))
    });
}

/// Benchmark formatting medium XML responses (100KB).
fn bench_format_xml_medium(c: &mut Criterion) {
    let xml = generate_large_xml(100);

    let mut group = c.benchmark_group("format_xml_medium");
    group.throughput(Throughput::Bytes(xml.len() as u64));

    group.bench_function("format_xml_medium_100kb", |b| {
        b.iter(|| format_xml_pretty(black_box(&xml)))
    });

    group.finish();
}

/// Benchmark formatting large XML responses (1MB).
fn bench_format_xml_large(c: &mut Criterion) {
    let xml = generate_large_xml(1024);

    let mut group = c.benchmark_group("format_xml_large");
    group.throughput(Throughput::Bytes(xml.len() as u64));
    group.sample_size(10);

    group.bench_function("format_xml_large_1mb", |b| {
        b.iter(|| format_xml_pretty(black_box(&xml)))
    });

    group.finish();
}

/// Benchmark formatting XML with varying sizes.
fn bench_format_xml_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_xml_scaling");

    for size_kb in [1, 10, 50, 100, 500, 1000].iter() {
        let xml = generate_large_xml(*size_kb);

        group.throughput(Throughput::Bytes(xml.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}kb", size_kb)),
            size_kb,
            |b, _| b.iter(|| format_xml_pretty(black_box(&xml))),
        );
    }

    group.finish();
}

/// Benchmark minified vs formatted JSON parsing.
fn bench_format_json_minified(c: &mut Criterion) {
    let minified =
        r#"{"id":1,"name":"Test","data":{"nested":{"value":"test"},"array":[1,2,3,4,5]}}"#
            .repeat(1000);

    c.bench_function("format_json_minified", |b| {
        b.iter(|| format_json_pretty(black_box(&minified)))
    });
}

criterion_group!(
    benches,
    bench_format_json_small,
    bench_format_json_medium,
    bench_format_json_large,
    bench_format_json_very_large,
    bench_format_json_scaling,
    bench_format_json_nested,
    bench_format_xml_small,
    bench_format_xml_medium,
    bench_format_xml_large,
    bench_format_xml_scaling,
    bench_format_json_minified
);

criterion_main!(benches);
