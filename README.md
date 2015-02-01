# scan

Macros for user input

## Example

See the [this file](https://github.com/mahkoh/scan/tree/master/example/src).

## Description

**scan** provides three macros:

- `scan!`
- `scanln!`
- `readln!`

`scan!` and `scanln!` take one string argument that specifies the input,
`readln!` takes no arguments.

`readln!` reads one line and returns it (without the terminating LF) as a
`String`. It can be thought of as having the following function signature:

```rust
fn readln!() -> String
```

`scan!` parses the input according to its argument and returns the parsed values
in a tuple. `scan!` can be used multiple times on the same input line.

`scanln!` is like `scan!` but always consumes a whole line. Depending on the argument,
`scan!` and `scanln!` fit one of the following function signatures:
```rust
fn scanln!(spec: &str)
fn scanln!(spec: &str) -> Option<T>
fn scanln!(spec: &str) -> (Option<T1>, Option<T2>)
fn scanln!(spec: &str) -> (Option<T1>, Option<T2>, Option<T3>)
...
```

## Specifiers

There are six kinds of specifiers:

- Unsigned integers
- Signed integers
- Floats
- Strings,
- Literals
- Whitespace

The first four are written like this: `{i32}`. Whitespace is either a space or
a tab. All other characters are literals. Note that the `{` literal has to be
escaped as `{{` and the `}` literal has to be escaped as `}}`.

Specifier | Parses        | Return type
--------- | ------------- | ------------
`{i8}`  | Signed   integer | `Option<i8>`
`{u8}`  | Unsigned integer | `Option<u8>`
`{i16}` | Signed   integer | `Option<i16>`
`{u16}` | Unsigned integer | `Option<u16>`
`{i32}` | Signed   integer | `Option<i32>`
`{u32}` | Unsigned integer | `Option<u32>`
`{i64}` | Signed   integer | `Option<i64>`
`{u64}` | Unsigned integer | `Option<u64>`
`{i}`   | Signed   integer | `Option<int>`
`{u}`   | Unsigned integer | `Option<uint>`
`{f32}` | Float            | `Option<f32>`
`{f64}` | Float            | `Option<f64>`
`{s}`   | String           | `Option<String>`
`{{`    | Literal `{`      | n/a
`}}`    | Literal `}`      | n/a
` `     | Whitespace       | n/a
`\t `   | Whitespace       | n/a
Otherwise | Literal        | n/a

## Parsing

Parsing is always greedy. This implies that the `{u8}` specifier will happily
parse the string `0xFFFFFF` and it will be indistinguishable from `0xFF`.

Parsing stops when the parser encounters a byte that doesn't fit the input
specification. In the returned tuple all values from that point on will be
`None`.

Whitespace parses zero or more of the following characters: Horizontal tab,
vertical tab, form feed, carriage return, space.

Literals must appear literally in the input stream.

String parses zero or more non-whitespace characters. Note that parsing a string
always succeeds but the returned value can still be `None` if the process was
stopped before it reached the String specifier.

Unsigned integers will be parsed according to one of the following regular
expressions:

- `0b[0-1]+`
- `0o[0-7]+`
- `[0-9]+`
- `0x[0-9a-fA-F]+`

Signed integers can have a `+` or `-` prefix.

Floats look like this:

- `[+-]?0b[0-1]+(\.[0-1]*)?`
- `[+-]?0o[0-7]+(\.[0-7]*)?`
- `[+-]?[0-9]+(\.[0-9]*)?`
- `[+-]?0x[0-9a-fA-F]+(\.[0-9a-fA-F]*)?`

## Limitations

You cannot use the `}}` specifier right after the end of a specifier of the form
`{i8}` because `{i8}}}` will be parsed like this: `{ i8 }} }`.

## Usage

### Importing the crates:
```rust
#![feature(phase)]

extern crate scan;
#[phase(plugin)] extern crate scan_mac;
```

### Cargo

```
[dependencies.scan]
git = "https://github.com/mahkoh/scan"

[dependencies.scan_mac]
git = "https://github.com/mahkoh/scan"
```
