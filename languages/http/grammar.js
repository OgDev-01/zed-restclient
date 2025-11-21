/**
 * Tree-sitter Grammar for HTTP Request Files (.http, .rest)
 *
 * Comprehensive grammar for parsing HTTP request files with support for:
 * - HTTP methods (GET, POST, PUT, DELETE, etc.)
 * - URLs with variables
 * - HTTP versions
 * - Headers with multi-line values
 * - Request bodies (JSON, XML, GraphQL, etc.)
 * - Variables {{variable}}
 * - Comments (# and //)
 * - Request separators (###)
 */

module.exports = grammar({
  name: 'http',

  extras: $ => [
    /[ \t]/,
  ],

  conflicts: $ => [
    [$.request],
  ],

  rules: {
    source_file: $ => repeat(
      choice(
        $.request,
        $.comment,
        $.request_separator,
        $.blank_line,
      )
    ),

    blank_line: $ => /\r?\n/,

    // Request separator: ###
    request_separator: $ => seq(
      '###',
      optional(/[^\r\n]*/),
      /\r?\n/,
    ),

    // Complete HTTP request
    request: $ => seq(
      $.method_line,
      optional($.headers),
      optional($.body),
    ),

    // Method line: METHOD URL [HTTP/VERSION]
    method_line: $ => seq(
      field('method', $.method),
      /[ \t]+/,
      field('url', $.target),
      optional(seq(
        /[ \t]+/,
        field('version', $.http_version),
      )),
      /\r?\n/,
    ),

    // HTTP methods
    method: $ => choice(
      'GET',
      'POST',
      'PUT',
      'DELETE',
      'PATCH',
      'OPTIONS',
      'HEAD',
      'TRACE',
      'CONNECT',
    ),

    // Target URL (with or without variables)
    target: $ => /[^\s\r\n]+/,

    // HTTP version (e.g., HTTP/1.1, HTTP/2)
    http_version: $ => /HTTP\/[0-9]+(\.[0-9]+)?/,

    // Headers section - at least one header
    headers: $ => repeat1($.header),

    // Single header: Name: Value
    header: $ => seq(
      field('name', $.header_name),
      ':',
      optional(seq(
        optional(/[ \t]+/),
        field('value', $.header_value),
      )),
      /\r?\n/,
    ),

    header_name: $ => /[A-Za-z][\w-]*/,

    header_value: $ => /[^\r\n]+/,

    // Request body - starts with blank line, content must not start with ###
    body: $ => seq(
      /\r?\n/,
      field('content', $.body_content),
    ),

    // Body content - at least one line that doesn't start with ###
    body_content: $ => prec.right(repeat1(
      choice(
        // Non-empty line that doesn't start with #
        seq(/[^\r\n#]+/, /\r?\n/),
        // Single # (comment in body)
        seq('#', token.immediate(/[^#\r\n]/), /[^\r\n]*/, /\r?\n/),
        // Double ## (but not ###)
        seq('##', token.immediate(/[^#\r\n]/), /[^\r\n]*/, /\r?\n/),
        // Blank line (but stop at EOF or ###)
        /\r?\n/,
      )
    )),

    // Variable: {{variableName}} or {{$systemVar}}
    variable: $ => seq(
      '{{',
      field('name', $.variable_name),
      '}}',
    ),

    variable_name: $ => /[^}]+/,

    // Comments: # or //
    comment: $ => seq(
      choice('#', '//'),
      /[^\r\n]*/,
      /\r?\n/,
    ),
  }
});
