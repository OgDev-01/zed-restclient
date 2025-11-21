/**
 * Tree-sitter Grammar for HTTP Request Files (.http, .rest)
 *
 * This is a minimal grammar for basic syntax highlighting.
 * Full grammar implementation will be added in Phase 4.
 */

module.exports = grammar({
  name: 'http',

  extras: $ => [
    /\s/,
    $.comment,
  ],

  rules: {
    source_file: $ => repeat(
      choice(
        $.request,
        $.comment,
        $.request_separator,
      )
    ),

    request_separator: $ => /###.*/,

    request: $ => seq(
      $.method_line,
      optional($.headers),
      optional($.body),
    ),

    method_line: $ => seq(
      field('method', $.method),
      field('url', $.url),
      optional(field('version', $.http_version)),
    ),

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

    url: $ => /https?:\/\/[^\s]+/,

    http_version: $ => /HTTP\/[0-9.]+/,

    headers: $ => repeat1($.header),

    header: $ => seq(
      field('name', $.header_name),
      ':',
      field('value', $.header_value),
    ),

    header_name: $ => /[A-Za-z][\w-]*/,

    header_value: $ => /[^\r\n]+/,

    body: $ => seq(
      /\n/,
      field('content', $.body_content),
    ),

    body_content: $ => /[^#]([^#]|#[^#]|##[^#])*/,

    variable: $ => /\{\{[^}]+\}\}/,

    comment: $ => choice(
      seq('#', /.*/),
      seq('//', /.*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'),
    ),

    // Tokens for syntax highlighting
    string: $ => choice(
      seq('"', /[^"]*/, '"'),
      seq("'", /[^']*/, "'"),
      $.url,
      $.header_value,
    ),

    number: $ => /\d+/,

    boolean: $ => choice('true', 'false'),

    null: $ => 'null',

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    property: $ => seq($.header_name, ':'),

    operator: $ => choice(
      '###',
      '=',
      ':',
    ),
  }
});
