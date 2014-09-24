use std::collections::{RingBuf, Deque};

pub struct PeekN<U, T> {
    intern: T,
    counter: uint,
    peeked: RingBuf<U>,
}

impl<U: Copy, T: Iterator<U>> PeekN<U, T> {
    pub fn new(intern: T) -> PeekN<U, T> {
        PeekN {
            intern: intern,
            counter: 0,
            peeked: RingBuf::new(),
        }
    }

    pub fn peek(&mut self, n: uint) -> Option<U> {
        if n >= self.peeked.len() {
            let m = n - self.peeked.len() + 1;
            for _ in range(0, m) {
                match self.intern.next() {
                    Some(s) => self.peeked.push(s),
                    _ => return None,
                }
            }
        }
        Some(self.peeked[n])
    }
}

impl<U, T: Iterator<U>> Iterator<(uint, U)> for PeekN<U, T> {
    fn next(&mut self) -> Option<(uint, U)> {
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

#[deriving(PartialEq)]
pub enum Token {
    LeftBrace,
    LeftBraceBrace,
    RightBrace,
    RightBraceBrace,
    Literal(uint),
    Colon,
    Space,
}

pub struct Stream {
    tokens: Vec<(uint, Token)>,
    pos: uint,
}

impl Stream {
    pub fn new(v: Vec<(uint, Token)>) -> Stream {
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

impl Iterator<(uint, Token)> for Stream {
    fn next(&mut self) -> Option<(uint, Token)> {
        if self.pos < self.tokens.len() {
            self.pos += 1;
            Some(self.tokens[self.pos-1])
        } else {
            None
        }
    }
}
