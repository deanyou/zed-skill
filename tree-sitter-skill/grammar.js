/**
 * Tree-sitter grammar for Cadence SKILL (Virtuoso) — a Lisp dialect.
 *
 * Features:
 *  - Line comments ";" and block comments "/* ... *\/"
 *  - Quote ' / quasiquote ` / unquote , / unquote-splicing ,@
 *  - Keyword arguments like ?keys ?count
 *  - Character literals like ?a ?\n
 *  - Booleans t / nil
 *  - Dotted pairs (a . b)
 *  - Strings with escapes, numbers (decimal / float / hex)
 */
module.exports = grammar({
  name: 'skill',

  extras: $ => [
    /\s/,
    $.comment,
    $.block_comment,
  ],

  rules: {
    program: $ => repeat($._form),

    _form: $ => choice(
      $.quoting,
      $.unquoting,
      $.list,
      $.bracket_list,
      $.brace_list,
      $.string,
      $.number,
      $.character,
      $.keyword,
      $.boolean,
      $.symbol,
    ),

    list: $ => seq(
      '(',
      repeat($._form),
      optional(seq('.', $._form)),
      ')',
    ),

    bracket_list: $ => seq('[', repeat($._form), ']'),

    brace_list: $ => seq('{', repeat($._form), '}'),

    quoting: $ => seq(choice("'", '`'), $._form),

    unquoting: $ => choice(
      seq($.unquote_splicing, $._form),
      seq(',', $._form),
    ),

    unquote_splicing: $ => token(seq(',', '@')),

    comment: $ => token(seq(';', /[^\n]*/)),

    block_comment: $ => token(seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),

    string: $ => seq(
      '"',
      repeat(choice($.escape_sequence, /[^"\\]/)),
      '"',
    ),

    escape_sequence: $ => /\\./,

    number: $ => token(choice(
      /0[xX][0-9a-fA-F]+/,
      /[+-]?(\d+(\.\d*)?|\.\d+)([eE][+-]?\d+)?/,
    )),

    // ?a ?\n ?\t (single character after '?'; letters are keyword args)
    character: $ => /\?(\\.|[^a-zA-Z\s()'"`,;])/,

    // ?key ?argList etc. — SKILL keyword arguments
    keyword: $ => token(seq('?', /[a-zA-Z][a-zA-Z0-9_.\-]*/)),

    boolean: $ => choice('t', 'nil'),

    symbol: $ => /[^0-9()\s;'"`,][^()\s;'"`,]*/,
  },
});
