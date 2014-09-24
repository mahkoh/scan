use libc;

static mut pushed: Option<u8> = None;

pub struct Stdin {
    done: bool,
}

impl Stdin {
    pub fn new() -> Stdin {
        Stdin { done: false }
    }

    pub fn next(&mut self) -> Result<u8, ()> {
        if self.done {
            return Err(());
        }

        unsafe {
            if pushed.is_some() {
                return Ok(pushed.take().unwrap());
            }
        }

        let mut buf = [0u8];
        match unsafe { libc::read(0, buf.as_mut_ptr() as *mut libc::c_void, 1) } {
            -1 | 0 => Err(()),
            _ if buf[0] == b'\n' => {
                self.done = true;
                Err(())
            },
            _ => Ok(buf[0]),
        }
    }

    pub fn push(&mut self, b: u8) {
        unsafe {
            if pushed.is_none() {
                pushed = Some(b);
            }
        }
    }
}
