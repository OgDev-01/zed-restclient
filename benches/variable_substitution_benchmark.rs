//! Benchmarks for variable substitution and environment management.
//!
//! These benchmarks measure the performance of variable resolution and substitution
//! to identify opportunities for caching and optimization.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rest_client::environment::Environment;
use rest_client::variables::{resolve_variables, substitute_variables};
use std::collections::HashMap;

/// Generate an environment with a specified number of variables.
fn generate_environment(num_vars: usize) -> Environment {
    let mut variables = HashMap::new();

    for i in 0..num_vars {
        variables.insert(format!("var_{}", i), format!("value_{}", i));
    }

    // Add some common variables
    variables.insert("baseUrl".to_string(), "https://api.example.com".to_string());
    variables.insert("authToken".to_string(), "bearer_token_12345".to_string());
    variables.insert("apiKey".to_string(), "api_key_67890".to_string());
    variables.insert("userId".to_string(), "user_123".to_string());

    Environment {
        name: "benchmark".to_string(),
        variables,
    }
}

/// Generate a request string with a specified number of variable references.
fn generate_request_with_variables(num_refs: usize) -> String {
    let mut request = String::from("GET {{baseUrl}}/api/v1/users/{{userId}}\n");
    request.push_str("Authorization: Bearer {{authToken}}\n");
    request.push_str("X-API-Key: {{apiKey}}\n");

    for i in 0..num_refs {
        request.push_str(&format!("X-Custom-Header-{}: {{{{var_{}}}}}\n", i, i % 100));
    }

    request
}

/// Generate a complex nested variable reference.
fn generate_nested_variables(depth: usize) -> String {
    let mut result = String::from("{{baseUrl}}");
    for i in 0..depth {
        result = format!("{{{{prefix_{}_{}_{}}}}}", i, result, i);
    }
    result
}

/// Benchmark simple variable substitution.
fn bench_substitute_simple(c: &mut Criterion) {
    let env = generate_environment(10);
    let request = "GET {{baseUrl}}/users/{{userId}}?api_key={{apiKey}}";

    c.bench_function("substitute_simple", |b| {
        b.iter(|| substitute_variables(black_box(request), black_box(&env.variables)))
    });
}

/// Benchmark variable substitution with many variables in environment.
fn bench_substitute_large_env(c: &mut Criterion) {
    let mut group = c.benchmark_group("substitute_large_env");

    for env_size in [10, 100, 500, 1000].iter() {
        let env = generate_environment(*env_size);
        let request = generate_request_with_variables(10);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_vars", env_size)),
            env_size,
            |b, _| b.iter(|| substitute_variables(black_box(&request), black_box(&env.variables))),
        );
    }

    group.finish();
}

/// Benchmark variable substitution with many references.
fn bench_substitute_many_refs(c: &mut Criterion) {
    let mut group = c.benchmark_group("substitute_many_refs");

    for num_refs in [10, 50, 100, 500].iter() {
        let env = generate_environment(100);
        let request = generate_request_with_variables(*num_refs);

        group.throughput(Throughput::Elements(*num_refs as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_refs", num_refs)),
            num_refs,
            |b, _| b.iter(|| substitute_variables(black_box(&request), black_box(&env.variables))),
        );
    }

    group.finish();
}

/// Benchmark variable resolution (finding all variable references).
fn bench_resolve_variables(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_variables");

    for num_refs in [10, 50, 100, 500].iter() {
        let request = generate_request_with_variables(*num_refs);

        group.throughput(Throughput::Elements(*num_refs as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_refs", num_refs)),
            num_refs,
            |b, _| b.iter(|| resolve_variables(black_box(&request))),
        );
    }

    group.finish();
}

/// Benchmark substitution with missing variables.
fn bench_substitute_missing_vars(c: &mut Criterion) {
    let env = generate_environment(10);
    let request = "GET {{baseUrl}}/users/{{missingVar1}}/posts/{{missingVar2}}?key={{apiKey}}";

    c.bench_function("substitute_missing_vars", |b| {
        b.iter(|| substitute_variables(black_box(request), black_box(&env.variables)))
    });
}

/// Benchmark substitution with no variables (passthrough).
fn bench_substitute_no_vars(c: &mut Criterion) {
    let env = generate_environment(10);
    let request = "GET https://api.example.com/users/123\nAuthorization: Bearer token123\nAccept: application/json";

    c.bench_function("substitute_no_vars", |b| {
        b.iter(|| substitute_variables(black_box(request), black_box(&env.variables)))
    });
}

/// Benchmark substitution in large request bodies.
fn bench_substitute_large_body(c: &mut Criterion) {
    let env = generate_environment(50);

    let mut body = String::from("{\n");
    for i in 0..100 {
        body.push_str(&format!("  \"field_{}\": \"{{{{var_{}}}}}\",\n", i, i % 50));
    }
    body.push_str("}");

    let request = format!(
        "POST {{{{baseUrl}}}}/api/data\n\
         Content-Type: application/json\n\
         Authorization: Bearer {{{{authToken}}}}\n\
         \n\
         {}",
        body
    );

    let mut group = c.benchmark_group("substitute_large_body");
    group.throughput(Throughput::Bytes(request.len() as u64));

    group.bench_function("substitute_large_body", |b| {
        b.iter(|| substitute_variables(black_box(&request), black_box(&env.variables)))
    });

    group.finish();
}

/// Benchmark variable substitution with dynamic variables (timestamps, UUIDs, etc.).
fn bench_substitute_dynamic_vars(c: &mut Criterion) {
    let mut env_vars = HashMap::new();
    env_vars.insert("baseUrl".to_string(), "https://api.example.com".to_string());
    env_vars.insert("$timestamp".to_string(), "1704067200000".to_string());
    env_vars.insert("$randomInt".to_string(), "42".to_string());
    env_vars.insert(
        "$uuid".to_string(),
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
    );

    let request = "POST {{baseUrl}}/events\n\
                   X-Request-ID: {{$uuid}}\n\
                   X-Timestamp: {{$timestamp}}\n\
                   X-Random: {{$randomInt}}\n";

    c.bench_function("substitute_dynamic_vars", |b| {
        b.iter(|| substitute_variables(black_box(request), black_box(&env_vars)))
    });
}

/// Benchmark environment lookup performance.
fn bench_environment_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("environment_lookup");

    for env_size in [10, 100, 500, 1000].iter() {
        let env = generate_environment(*env_size);
        let keys: Vec<String> = (0..*env_size).map(|i| format!("var_{}", i)).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_vars", env_size)),
            env_size,
            |b, _| {
                b.iter(|| {
                    for key in &keys {
                        black_box(env.variables.get(key));
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark variable substitution with URL encoding.
fn bench_substitute_url_encoding(c: &mut Criterion) {
    let mut env_vars = HashMap::new();
    env_vars.insert(
        "query".to_string(),
        "hello world & special=chars".to_string(),
    );
    env_vars.insert("path".to_string(), "users/john doe".to_string());
    env_vars.insert("baseUrl".to_string(), "https://api.example.com".to_string());

    let request = "GET {{baseUrl}}/{{path}}?q={{query}}";

    c.bench_function("substitute_url_encoding", |b| {
        b.iter(|| substitute_variables(black_box(request), black_box(&env_vars)))
    });
}

/// Benchmark repeated substitutions (simulating multiple requests).
fn bench_substitute_repeated(c: &mut Criterion) {
    let env = generate_environment(50);
    let requests: Vec<String> = (0..100)
        .map(|i| {
            format!(
                "GET {{{{baseUrl}}}}/resource/{}\n\
                 Authorization: Bearer {{{{authToken}}}}\n\
                 X-Request-ID: {{{{var_{}}}}}\n",
                i,
                i % 50
            )
        })
        .collect();

    let mut group = c.benchmark_group("substitute_repeated");
    group.throughput(Throughput::Elements(100));

    group.bench_function("substitute_100_requests", |b| {
        b.iter(|| {
            for request in &requests {
                black_box(substitute_variables(
                    black_box(request),
                    black_box(&env.variables),
                ));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_substitute_simple,
    bench_substitute_large_env,
    bench_substitute_many_refs,
    bench_resolve_variables,
    bench_substitute_missing_vars,
    bench_substitute_no_vars,
    bench_substitute_large_body,
    bench_substitute_dynamic_vars,
    bench_environment_lookup,
    bench_substitute_url_encoding,
    bench_substitute_repeated
);

criterion_main!(benches);
