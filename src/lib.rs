extern crate librsync_sys as raw;

use std::io::{self, Read};
use std::marker::PhantomData;
use std::ops::Deref;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MagicNumber {
    Delta,
    MD4,
    Blake2,
}

#[derive(Debug)]
pub enum Error {
    Done,
    Blocked,
    Running,
    TestSkipped,
    Io(io::Error),
    Syntax,
    Mem,
    InputEnded,
    BadMagic,
    Unimplemented,
    Corrupt,
    Internal,
    Param,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Signature<R: Read> {
    old: R,
    job: Job,
    buf: Vec<u8>,
    pos: usize,
    cap: usize,
    // true when input has ended
    done: bool,
}

struct Job(*mut raw::rs_job_t);

// Wrapper around rs_buffers_t.
struct Buffers<'a> {
    inner: raw::rs_buffers_t,
    _phantom: PhantomData<&'a u8>,
}


impl<R: Read> Signature<R> {
    pub fn new(old: R,
               new_block_len: usize,
               strong_len: usize,
               sig_magic: MagicNumber)
               -> Result<Self> {
        let job = unsafe { raw::rs_sig_begin(new_block_len, strong_len, sig_magic.as_raw()) };
        if job.is_null() {
            return Err(Error::BadMagic);
        }
        Ok(Signature {
            old: old,
            job: Job(job),
            buf: vec![0; raw::RS_DEFAULT_BLOCK_LEN],
            pos: 0,
            cap: 0,
            done: false,
        })
    }

    pub fn into_inner(self) -> R {
        self.old
    }
}

impl<R: Read> Read for Signature<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut out_pos = 0;
        let mut out_cap = buf.len();

        loop {
            if self.cap == 0 && !self.done {
                // need to read some data from input
                let read = try!(self.old.read(&mut self.buf));
                if read == 0 {
                    self.done = true;
                }
                self.pos = 0;
                self.cap = read;
            }

            // work
            let mut buffers = Buffers::new(&self.buf[self.pos..self.pos + self.cap],
                                           &mut buf[out_pos..],
                                           self.done);
            let res = unsafe { raw::rs_job_iter(*self.job, buffers.as_raw()) };
            if res != raw::RS_DONE && res != raw::RS_BLOCKED {
                let err = Error::from_raw(res);
                return Err(other_io_err(format!("Error processing signature: \'{:?}\'", err)));
            }

            // update buffer cap and pos
            let read = self.cap - buffers.available_input();
            self.pos += read;
            self.cap -= read;
            // determine written
            let written = out_cap - buffers.available_output();
            out_pos += written;
            out_cap -= written;
            if out_cap == 0 || written == 0 {
                return Ok(out_pos);
            }
        }
    }
}


impl Error {
    fn from_raw(val: raw::rs_result) -> Option<Self> {
        match val {
            raw::RS_DONE => Some(Error::Done),
            raw::RS_BLOCKED => Some(Error::Blocked),
            raw::RS_RUNNING => Some(Error::Running),
            raw::RS_TEST_SKIPPED => Some(Error::TestSkipped),
            raw::RS_IO_ERROR => Some(Error::Io(other_io_err("Unknown IO error"))),
            raw::RS_SYNTAX_ERROR => Some(Error::Syntax),
            raw::RS_MEM_ERROR => Some(Error::Mem),
            raw::RS_INPUT_ENDED => Some(Error::InputEnded),
            raw::RS_BAD_MAGIC => Some(Error::BadMagic),
            raw::RS_UNIMPLEMENTED => Some(Error::Unimplemented),
            raw::RS_CORRUPT => Some(Error::Corrupt),
            raw::RS_INTERNAL_ERROR => Some(Error::Internal),
            raw::RS_PARAM_ERROR => Some(Error::Param),
            _ => None,
        }
    }
}


impl MagicNumber {
    pub fn from_raw(raw: raw::rs_magic_number) -> Option<Self> {
        match raw {
            raw::RS_DELTA_MAGIC => Some(MagicNumber::Delta),
            raw::RS_MD4_SIG_MAGIC => Some(MagicNumber::MD4),
            raw::RS_BLAKE2_SIG_MAGIC => Some(MagicNumber::Blake2),
            _ => None,
        }
    }

    fn as_raw(&self) -> raw::rs_magic_number {
        match *self {
            MagicNumber::Delta => raw::RS_DELTA_MAGIC,
            MagicNumber::MD4 => raw::RS_MD4_SIG_MAGIC,
            MagicNumber::Blake2 => raw::RS_BLAKE2_SIG_MAGIC,
        }
    }
}


impl Deref for Job {
    type Target = *mut raw::rs_job_t;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                raw::rs_job_free(self.0);
            }
        }
    }
}


impl<'a> Buffers<'a> {
    pub fn new(in_buf: &'a [u8], out_buf: &'a mut [u8], eof_in: bool) -> Self {
        Buffers {
            inner: raw::rs_buffers_t {
                next_in: in_buf.as_ptr() as *const i8,
                avail_in: in_buf.len(),
                eof_in: if eof_in { 1 } else { 0 },
                next_out: out_buf.as_mut_ptr() as *mut i8,
                avail_out: out_buf.len(),
            },
            _phantom: PhantomData,
        }
    }

    pub fn as_raw(&mut self) -> *mut raw::rs_buffers_t {
        &mut self.inner
    }

    pub fn available_input(&self) -> usize {
        self.inner.avail_in as usize
    }

    pub fn available_output(&self) -> usize {
        self.inner.avail_out as usize
    }
}


fn other_io_err<T: AsRef<str>>(msg: T) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg.as_ref())
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::{Cursor, Read};

    #[test]
    fn signature() {
        let data = "this is a string to be tested";
        let cursor = Cursor::new(data);
        let mut sig = Signature::new(cursor, 10, 5, MagicNumber::MD4).unwrap();
        let mut signature = Vec::new();
        let read = sig.read_to_end(&mut signature).unwrap();
        assert_eq!(read, signature.len());
        let expected = vec![0x72, 0x73, 0x01, 0x36, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00,
                            0x05, 0x1b, 0x21, 0x04, 0x8b, 0xad, 0x3c, 0xbd, 0x19, 0x09, 0x1d,
                            0x1b, 0x04, 0xf0, 0x9d, 0x1f, 0x64, 0x31, 0xde, 0x15, 0xf4, 0x04,
                            0x87, 0x60, 0x96, 0x19, 0x50, 0x39];
        assert_eq!(signature, expected);
    }

    #[test]
    fn signature_invalid_magic() {
        let data = "this is a string to be tested";
        let cursor = Cursor::new(data);
        let sig = Signature::new(cursor, 10, 5, MagicNumber::Delta);
        assert!(sig.is_err());
    }
}
