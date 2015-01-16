#![crate_name="scan_mac"]
#![crate_type="dylib"]
#![feature(plugin_registrar, quote)]
#![allow(unstable)]

extern crate syntax;
extern crate rustc;

use IntType::*;
use Arg::*;

use std::string::{String};

use rustc::plugin::{Registry};

use syntax::{ast};
use syntax::ptr::{P};
use syntax::ast::{TokenTree, LitStr, Expr, ExprLit, Block, DefaultBlock, ExprLoop,
                  ExprTup};
use syntax::codemap::{Span, Pos};
use syntax::ext::base::{DummyResult, ExtCtxt, MacResult, MacExpr};
use syntax::fold::{Folder};
use syntax::parse::{new_parser_from_tts};
use syntax::parse::token::{Eof};

use util::{PeekN, Stream, LeftBrace, LeftBraceBrace, RightBrace, RightBraceBrace,
           Literal, Colon, Space, Token};

mod util;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("scanln", expand_scanln);
    reg.register_macro("scan",   expand_scan);
    reg.register_macro("readln", expand_readln);
}

fn parse_macro(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Option<(String, Span)> {
    let mut parser = new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts.to_vec());
    let arg = cx.expander().fold_expr(parser.parse_expr());
    let arg_str = match arg.node {
        ExprLit(ref lit) => match lit.node {
            LitStr(ref s, _) => Some(s.get().to_string()),
            _ => None,
        },
        _ => None,
    };
    if parser.token != Eof {
        cx.span_err(parser.span, "unexpected token");
        return None;
    }
    match arg_str {
        Some(s) => Some((s, arg.span)),
        None => {
            cx.span_err(arg.span, "expected string literal");
            None
        }
    }
}

struct Parser<'a, 'b: 'a> {
    cx: &'a mut ExtCtxt<'b>,
    bytes: &'a str,
    span: Span,
    stream: Stream,
    args: Vec<Arg>,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn new(cx: &'a mut ExtCtxt<'b>, l: &'a str, span: Span) -> Parser<'a, 'b> {
        Parser {
            cx: cx,
            bytes: l,
            span: span,
            stream: Stream::new(vec!()),
            args: vec!(),
        }
    }

    fn err<T>(&mut self, i: usize, err: &str) -> Result<T,()> {
        self.span.lo = Pos::from_uint(self.span.lo.to_uint() + i + 1);
        self.span.hi = self.span.lo;
        self.cx.span_err(self.span, err);
        Err(())
    }

    fn tokenize(&mut self) -> Result<(), ()> {
        let mut tokens: Vec<(usize, Token)>  = vec!();
        let mut chars = PeekN::new(self.bytes.chars());
        loop {
            let (mut i, b) = match chars.next() {
                Some(b) => b,
                _ => break,
            };

            if b == ' ' || b == '\t' {
                match tokens.pop() {
                    Some((j, Space)) => i = j,
                    Some(x)          => tokens.push(x),
                    None             => { },
                }
                tokens.push((i, Space));
                continue;
            }

            match b as u32 {
                0x20 ... 0x7E => { },
                _ => try!(self.err(i, "expected Ascii character")),
            }

            let mut is_punct = false;
            macro_rules! punc {
                ($p:expr) => {{ tokens.push((i, $p)); is_punct = true; }}
            };

            match (b, chars.peek(0)) {
                ('{', Some('{')) => punc!(LeftBraceBrace),
                ('}', Some('}')) => punc!(RightBraceBrace),
                _ => { },
            }
            if is_punct {
                chars.next();
                continue;
            }

            match b {
                ':' => punc!(Colon),
                '{' => punc!(LeftBrace),
                '}' => punc!(RightBrace),
                _ => { },
            }
            if is_punct {
                continue;
            }

            let mut len = 1;
            match tokens.pop() {
                Some((j, Literal(old_len))) => { i = j; len += old_len; },
                Some(x) => tokens.push(x),
                None => { },
            }
            tokens.push((i, Literal(len)));
        }
        self.stream = Stream::new(tokens);
        Ok(())
    }

    fn parse(mut self) -> Result<Vec<Arg>, ()> {
        try!(self.tokenize());

        loop {
            let (i, t) = match self.stream.next() {
                Some(t) => t,
                _ => break,
            };

            macro_rules! push_lit {
                ($l:expr) => {
                    match self.args.pop() {
                        Some(Lit(mut v)) => {
                            v.push_str($l);
                            self.args.push(Lit(v));
                        },
                        Some(x) => {
                            self.args.push(x);
                            self.args.push(Lit($l.to_string()));
                        },
                        _ => {
                            self.args.push(Lit($l.to_string()));
                        },
                    }
                }
            };

            match t {
                Space           => self.args.push(Whitespace),
                Literal(len)    => {
                    push_lit!(self.bytes.slice(i, i+len));
                },
                LeftBraceBrace  => push_lit!("{"),
                RightBraceBrace => push_lit!("}"),
                Colon           => push_lit!(":"),
                RightBrace      => try!(self.err(i, "Unexpected token")),
                LeftBrace => {
                    self.stream.skip_spaces();
                    let (j, len) = match self.stream.next() {
                        Some((j, Literal(len))) => (j, len),
                        Some((j, _)) => try!(self.err(j, "Expected type")),
                        _ => try!(self.err(i, "Unexpected EOF")),
                    };
                    let arg = match self.bytes.slice(j, j+len) {
                        "i8"  => Int(I8),
                        "u8"  => Int(U8), 
                        "i16" => Int(I16), 
                        "u16" => Int(U16), 
                        "i32" => Int(I32), 
                        "u32" => Int(U32), 
                        "i64" => Int(I64), 
                        "u64" => Int(U64), 
                        "i"   => Int(I), 
                        "u"   => Int(U),
                        "f32" => Float(false),
                        "f64" => Float(true),
                        "s"   => Strin,
                        _ => try!(self.err(j, "Unknown type")),
                    };
                    self.args.push(arg);
                    self.stream.skip_spaces();
                    match self.stream.next() {
                        Some((_, RightBrace)) => { },
                        None => try!(self.err(0, "Unexpected EOF")),
                        Some((i, _)) => try!(self.err(i, "Unexpected token")),
                    }
                },
            }
        }

        Ok(self.args)
    }
}

#[derive(Copy)]
enum IntType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    I,
    U,
}

impl IntType {
    fn map<'a>(self, cx: &ExtCtxt<'a>) -> P<Expr> {
        let ty = match self {
            I8  => quote_expr!(cx, i8),
            U8  => quote_expr!(cx, u8),
            I16 => quote_expr!(cx, i16),
            U16 => quote_expr!(cx, u16),
            I32 => quote_expr!(cx, i32),
            U32 => quote_expr!(cx, u32),
            I64 => quote_expr!(cx, i64),
            U64 => quote_expr!(cx, u64),
            I   => quote_expr!(cx, isize),
            U   => quote_expr!(cx, usize),
        };
        quote_expr!(cx, |v| v as $ty)
    }

    fn signed(self) -> bool {
        match self {
            I8 | I16 | I32 | I64 | I => true,
            U8 | U16 | U32 | U64 | U => false,
        }
    }
}

enum Arg {
    Lit(String),
    Whitespace,
    Int(IntType),
    Float(bool),
    Strin,
}

fn expand_scanln<'a>(cx: &'a mut ExtCtxt, sp: Span,
                      tts: &[TokenTree]) -> Box<MacResult+'static> {
    expand_scan_common(cx, sp, tts, true)
}

fn expand_scan<'a>(cx: &'a mut ExtCtxt, sp: Span,
                   tts: &[TokenTree]) -> Box<MacResult+'static> {
    expand_scan_common(cx, sp, tts, false)
}

fn expand_scan_common<'a>(cx: &'a mut ExtCtxt, sp: Span, tts: &[TokenTree],
                          drop_line: bool) -> Box<MacResult+'static> {
    let (lit, span) = match parse_macro(cx, tts) {
        Some(x) => x,
        None => return DummyResult::expr(sp),
    };

    let args = match Parser::new(cx, lit.as_slice(), span).parse() {
        Ok(args) => args,
        _ => return DummyResult::expr(sp),
    };

    let cx = &*cx;

    let mut decls = vec!();
    let mut retvs = vec!();
    let mut tupel_vals = vec!();

    retvs.push(quote_stmt!(cx,
        let mut pb = ::scan::stdin($drop_line);
    ));

    for arg in args.into_iter() {
        let i = decls.len();
        let ident = cx.ident_of(format!("a{}", i).as_slice());
        match arg {
            Int(..) | Float(..) | Strin => {
                decls.push(quote_stmt!(cx,
                    let mut $ident = None;
                ));
                // ¯\_(ツ)_/¯
                decls.push(quote_stmt!(cx,
                    let _ = $ident;
                ));
                tupel_vals.push(quote_expr!(cx, $ident));
            },
            _ => { },
        }
        match arg {
            Lit(ref v) => {
                let ss = v.as_slice();
                retvs.push(quote_stmt!(cx,
                    if pb.literal($ss).is_none() {
                        break;
                    }
                ));
            },
            Whitespace => {
                retvs.push(quote_stmt!(cx,
                    pb.whitespace();
                ));
            },
            Int(ty) => {
                let map = ty.map(cx);
                if ty.signed() {
                    retvs.push(quote_stmt!(cx,
                        $ident = pb.signed_integer().map($map);
                    ));
                } else {
                    retvs.push(quote_stmt!(cx,
                        $ident = pb.unsigned_integer().map($map);
                    ));
                }
                retvs.push(quote_stmt!(cx,
                    if $ident.is_none() {
                        break;
                    }
                ));
            },
            Float(long) => {
                if long {
                    retvs.push(quote_stmt!(cx,
                        $ident = pb.float();
                    ));
                } else {
                    retvs.push(quote_stmt!(cx,
                        $ident = pb.float().map(|v| as f32);
                    ));
                }
            },
            Strin => {
                retvs.push(quote_stmt!(cx,
                    $ident = Some(pb.word());
                ));
            },
        }
    }
    retvs.push(quote_stmt!(cx, break;));

    let loop_block = P(Block {
        view_items: vec!(),
        stmts: retvs,
        expr: None,
        id: ast::DUMMY_NODE_ID,
        rules: DefaultBlock,
        span: sp,
    });

    let looop = P(Expr {
        id: ast::DUMMY_NODE_ID,
        node: ExprLoop(loop_block, None),
        span: sp,
    });


    let tupel = if tupel_vals.len() != 1 {
        P(Expr {
            id: ast::DUMMY_NODE_ID,
            node: ExprTup(tupel_vals),
            span: sp,
        })
    } else {
        quote_expr!(cx, a0)
    };

    let mut statements = vec!();
    for d in decls.into_iter() {
        statements.push(quote_stmt!(cx, $d));
    }
    statements.push(quote_stmt!(cx, $looop));

    let final_block = P(Block {
        view_items: vec!(),
        stmts: statements,
        expr: Some(tupel),
        id: ast::DUMMY_NODE_ID,
        rules: DefaultBlock,
        span: sp,
    });

    MacExpr::new(quote_expr!(cx, $final_block))
}

fn expand_readln<'a>(cx: &'a mut ExtCtxt, sp: Span,
                     _tts: &[TokenTree]) -> Box<MacResult+'static> {
    let decl = quote_stmt!(cx,
        let mut pb = ::scan::stdin(false);
    );
    let res = quote_expr!(cx,
        pb.line()
    );

    let final_block = P(Block {
        view_items: vec!(),
        stmts: vec!(decl),
        expr: Some(res),
        id: ast::DUMMY_NODE_ID,
        rules: DefaultBlock,
        span: sp,
    });

    MacExpr::new(quote_expr!(cx, $final_block))
}
