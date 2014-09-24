use std;

pub static REPLACEMENT: char = '\ufffd';

pub struct UTF8 {
    cur: u32,
    rem: u32,
}

impl UTF8 {
    pub fn new() -> UTF8 {
        UTF8 {
            cur: 0,
            rem: 0,
        }
    }

    pub fn push(&mut self, b: u8) -> (Option<char>, Option<char>) {
        if b < 128 {
            if self.rem > 0 {
                self.rem = 0;
                (Some(REPLACEMENT), Some(b as char))
            } else {
                (None, Some(b as char))
            }
        } else if b >= 0xC0 {
            let old = if self.rem > 0 {
                Some(REPLACEMENT)
            } else {
                None
            };
            self.rem = 0;
            match std::str::utf8_char_width(b) {
                0 => (old, Some(REPLACEMENT)),
                n => {
                    self.rem = n as u32;
                    self.cur = b as u32 & (0xFF >> (n + 1));
                    (old, Some(REPLACEMENT))
                },
            }
        } else {
            if self.rem > 0 {
                self.rem -= 1;
                self.cur = self.cur << 6 | (b & 0x3F) as u32;
                if self.rem == 0 {
                    match std::char::from_u32(self.cur) {
                        Some(c) => (Some(c), None),
                        _ => (Some(REPLACEMENT), None),
                    }
                } else {
                    (None, None)
                }
            } else {
                (Some(REPLACEMENT), None)
            }
        }
    }

    pub fn pending(&self) -> bool {
        self.rem > 0
    }
}
