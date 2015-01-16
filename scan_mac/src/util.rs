use std::collections::{RingBuf};

pub use self::Token::*;

pub struct PeekN<U, T> {
    intern: T,
    counter: usize,
    peeked: RingBuf<U>,
}

impl<U: Copy, T: Iterator<Item = U>> PeekN<U, T> {
    pub fn new(intern: T) -> PeekN<U, T> {
        PeekN {
            intern: intern,
            counter: 0,
            peeked: RingBuf::new(),
        }
    }

    pub fn peek(&mut self, n: usize) -> Option<U> {
        if n >= self.peeked.len() {
            let m = n - self.peeked.len() + 1;
            for _ in range(0, m) {
                match self.intern.next() {
                    Some(s) => self.peeked.push_back(s),
                    _ => return None,
                }
            }
        }
        Some(self.peeked[n])
    }
}

impl<U, T: Iterator<Item=U>> Iterator for PeekN<U, T> {
    type Item = (usize, U);

    fn next(&mut self) -> Option<(usize, U)> {
        let res = match self.peeked.pop_front() {
            x @ Some(_) => x,
            _ => self.intern.next(),
        };
        match res {
            Some(x) => {
                self.counter += 1;
                Some((self.counter - 1, x))
            },
            _ => None,
        }
    }
}

#[derive(PartialEq, Copy)]
pub enum Token {
    LeftBrace,
    LeftBraceBrace,
    RightBrace,
    RightBraceBrace,
    Literal(usize),
    Colon,
    Space,
}

pub struct Stream {
    tokens: Vec<(usize, Token)>,
    pos: usize,
}

impl Stream {
    pub fn new(v: Vec<(usize, Token)>) -> Stream {
        Stream {
            tokens: v,
            pos: 0,
        }
    }

    pub fn skip_spaces(&mut self) {
        loop {
            match self.next() {
                Some((_, Space)) => { },
                _ => {
                    self.step_back();
                    break;
                }
            }
        }
    }
    
    pub fn step_back(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }
}

impl Iterator for Stream {
    type Item = (usize, Token);

    fn next(&mut self) -> Option<(usize, Token)> {
        if self.pos < self.tokens.len() {
            self.pos += 1;
            Some(self.tokens[self.pos-1])
        } else {
            None
        }
    }
}
