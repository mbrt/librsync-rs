extern crate librsync_sys as raw;

use std::io::{self, Read};

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
    SyntaxError,
    MemError,
    InputEnded,
    BadMagic,
    Unimplemented,
    Corrupt,
    InternalError,
    ParamError,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Signature<R: Read> {
    old: R,
    job: *mut raw::rs_job_t,
    buf: Vec<u8>,
    pos: usize,
    cap: usize,
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
            job: job,
            buf: vec![0; raw::RS_DEFAULT_BLOCK_LEN],
            pos: 0,
            cap: 0,
        })
    }
}

impl<R: Read> Drop for Signature<R> {
    fn drop(&mut self) {
        assert!(!self.job.is_null());
        unsafe {
            raw::rs_job_free(self.job);
        }
    }
}

impl<R: Read> Read for Signature<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
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


#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn signature() {
        let data = "this is a string to be tested";
        let cursor = Cursor::new(data);
        let _sig = Signature::new(cursor, 10, 5, MagicNumber::MD4).unwrap();
    }

    #[test]
    fn signature_invalid_magic() {
        let data = "this is a string to be tested";
        let cursor = Cursor::new(data);
        let sig = Signature::new(cursor, 10, 5, MagicNumber::Delta);
        assert!(sig.is_err());
    }
}
