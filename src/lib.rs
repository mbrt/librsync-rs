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

pub struct SigStream;

pub fn compute_signature<R: Read>(old: R,
                                  new_block_len: usize,
                                  strong_len: usize,
                                  sig_magic: MagicNumber)
                                  -> Result<SigStream, Error> {
    unsafe {
        let job = raw::rs_sig_begin(new_block_len, strong_len, sig_magic.raw());
        if job.is_null() {
            return Err(Error::BadMagic);
        }
        raw::rs_job_free(job);
    }
    Ok(SigStream)
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

    fn raw(&self) -> raw::rs_magic_number {
        match *self {
            MagicNumber::Delta => raw::RS_DELTA_MAGIC,
            MagicNumber::MD4 =>  raw::RS_MD4_SIG_MAGIC,
            MagicNumber::Blake2 =>  raw::RS_BLAKE2_SIG_MAGIC,
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
        compute_signature(cursor, 10, 5, MagicNumber::MD4).unwrap();
    }

    #[test]
    fn signature_invalid_magic() {
        let data = "this is a string to be tested";
        let cursor = Cursor::new(data);
        assert!(compute_signature(cursor, 10, 5, MagicNumber::Delta).is_err());
    }
}
