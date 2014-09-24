#![feature(macro_rules)]
#![crate_name = "scan"]
#![crate_type = "lib"]

extern crate libc;

use stdin::{Stdin};
use utf8::{UTF8};

mod stdin;
mod utf8;

pub fn stdin(drop_line: bool) -> Scanner {
    Scanner {
        drop_line: drop_line,
        stdin: Stdin::new(),
    }
}

pub struct Scanner {
    drop_line: bool,
    stdin: Stdin,
}

macro_rules! get {
    ($s:ident) => { get_or!($s, {return None}) }
}

macro_rules! get_or {
    ($s:ident, $b:block) => {
        match $s.stdin.next() {
            Ok(b) => b,
            _ => $b,
        }
    }
}

macro_rules! digits {
    ($s:ident, [$($range:pat, $lo:block)|+], $base:expr, $invert:expr, $ty:ty) => {{
        let mut ok = false;
        let mut res = 0u as $ty;
        let base = $base as uint as $ty;
        let mut iterations = 0u;
        loop {
            let next = get_or!($s, {break});
            match next {
                $(
                    $range => {
                        ok = true;
                        res = res * base + (next - $lo) as $ty;
                    },
                )*
                _ => {
                    $s.stdin.push(next);
                    break;
                }
            }
            if $invert {
                iterations += 1;
            }
        }
        if $invert {
            res /= std::num::pow(base, iterations);
        }
        match ok {
            true => Some(res),
            false => None,
        }
    }}
}

enum IntType {
    Binary,
    Octal,
    Decimal,
    Hex,
}

impl Scanner {
    /// Parses binary digits
    pub fn binary(&mut self) -> Option<u64> {
        digits!(self, [b'0'..b'1', {b'0'}], 2, false, u64)
    }

    /// Parses binary digits after the .
    pub fn binary_inv(&mut self) -> Option<f64> {
        digits!(self, [b'0'..b'1', {b'0'}], 2, true, f64)
    }

    /// Parses octal digits
    pub fn octal(&mut self) -> Option<u64> {
        digits!(self, [b'0'..b'7', {b'0'}], 8, false, u64)
    }

    /// Parses octal digits after the .
    pub fn octal_inv(&mut self) -> Option<f64> {
        digits!(self, [b'0'..b'7', {b'0'}], 8, true, f64)
    }

    /// Parses decimal digits
    pub fn decimal(&mut self) -> Option<u64> {
        digits!(self, [b'0'..b'9', {b'0'}], 10, false, u64)
    }

    /// Parses decimal digits after the .
    pub fn decimal_inv(&mut self) -> Option<f64> {
        digits!(self, [b'0'..b'9', {b'0'}], 10, true, f64)
    }

    /// Parses hexadecimal digits
    pub fn hexadecimal(&mut self) -> Option<u64> {
        digits!(self, [b'0'..b'9', {b'0'} |
                       b'a'..b'f', {b'a' - 10} |
                       b'A'..b'F', {b'A' - 10}],
                16, false, u64)
    }

    /// Parses hexadecimal digits after the .
    pub fn hexadecimal_inv(&mut self) -> Option<f64> {
        digits!(self, [b'0'..b'9', {0} | b'a'..b'f', {10} | b'A'..b'F', {10}], 16, true, f64)
    }

    /// Parses digits depending on `ty`.
    fn digits(&mut self, ty: IntType) -> Option<u64> {
        match ty {
            Binary  => self.binary(),
            Octal   => self.octal(),
            Decimal => self.decimal(),
            Hex     => self.hexadecimal(),
        }
    }

    /// Parses digits after the `.` depending on `ty`.
    fn digits_inv(&mut self, ty: IntType) -> Option<f64> {
        match ty {
            Binary  => self.binary_inv(),
            Octal   => self.octal_inv(),
            Decimal => self.decimal_inv(),
            Hex     => self.hexadecimal_inv(),
        }
    }

    /// Parses an unsigned integer including its prefix
    pub fn unsigned_integer(&mut self) -> Option<u64> {
        match self.classify() {
            (ty, true)  => self.digits(ty).or(Some(0)),
            (ty, false) => self.digits(ty),
        }
    }

    /// Returns the type of the following integer and, in the case of a decimal, if one
    /// `0` has already been consumed.
    fn classify(&mut self) -> (IntType, bool) {
        match get_or!(self, {return (Decimal, false)}) {
            b'0' => {
                match get_or!(self, {return (Decimal, true)}) {
                    b'x' | b'X' => (Hex, false),
                    b'o' | b'O' => (Octal, false),
                    b'b' | b'B' => (Binary, false),
                    b => {
                        self.stdin.push(b);
                        (Decimal, true)
                    },
                }
            },
            b => {
                self.stdin.push(b);
                (Decimal, false)
            },
        }
    }

    /// Parses the sign of the following integer.
    fn sign(&mut self) -> i64 {
        match get_or!(self, {return 1}) {
            b'+' => 1,
            b'-' => -1,
            b => {
                self.stdin.push(b);
                1
            },
        }
    }

    /// Parses a signed integer.
    pub fn signed_integer(&mut self) -> Option<i64> {
        let sign = self.sign();
        self.unsigned_integer().map(|v| sign * v as i64)
    }

    /// Parses a floating point number.
    pub fn float(&mut self) -> Option<f64> {
        let sign = self.sign() as f64;
        let (ty, cons) = self.classify();
        let pre = match (self.digits(ty), cons) {
            (Some(d), _) => d as f64,
            (None, true) => 0.0,
            _ => return None,
        };
        match get_or!(self, {return Some(pre)}) {
            b'.' => { },
            b => {
                self.stdin.push(b);
                return Some(pre);
            },
        }
        match self.digits_inv(ty) {
            Some(d) => Some(sign * (pre + d)),
            _ => Some(sign * pre),
        }
    }

    /// Reads a word from the stream.
    ///
    /// A word is defined as the longest sequence of characters that doesn't contain a
    /// whitespace character. Invalid UTF-8 sequences will be replaced by U+FFFD.
    pub fn word(&mut self) -> String {
        self.string(true, false)
    }

    /// Reads a string from the stream.
    pub fn string(&mut self, word: bool, line: bool) -> String {
        let mut res = String::new();
        let mut utf8 = UTF8::new();
        loop {
            let next = get_or!(self, {break});
            if word && is_whitespace(next) {
                self.stdin.push(next);
                break;
            } else if line && next == b'\n' {
                break;
            }
            match utf8.push(next) {
                (Some(c1), Some(c2)) => {
                    res.push_char(c1);
                    res.push_char(c2);
                },
                (Some(c1), None) => res.push_char(c1),
                (None, Some(c2)) => res.push_char(c2),
                _ => { },
            }
        }
        if utf8.pending() {
            res.push_char(utf8::REPLACEMENT);
        }
        res
    }

    /// Read until the first non-whitespace character.
    pub fn whitespace(&mut self) {
        loop {
            let b = get_or!(self, {return});
            if !is_whitespace(b) {
                self.stdin.push(b);
                return;
            }
        }
    }

    /// Read `lit` from the stream.
    ///
    /// Reads the longest initial sequence from the stream that is also an initial
    /// sequence of `lit`. Returns `Some(())` if this sequence coincides with `lit`,
    /// `None` otherwise.
    pub fn literal(&mut self, lit: &str) -> Option<()> {
        for &b in lit.as_bytes().iter() {
            match self.stdin.next() {
                Ok(c) if c == b => { },
                Ok(c) => {
                    self.stdin.push(c);
                    return None;
                },
                _ => break,
            }
        }
        Some(())
    }

    /// Reads until the first newline.
    pub fn line(&mut self) -> String {
        self.string(false, true)
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        if self.drop_line {
            loop {
                if self.stdin.next().is_err() {
                    break;
                }
            }
        }
    }
}

/// Checks if `b` is a whitespace character.
fn is_whitespace(b: u8) -> bool {
    match b {
    // HT | LF | VT | FF | CR | SPACE
        9 | 10 | 11 | 12 | 13 | 32 => true,
        _ => false,
    }
}
