extern crate librsync_sys as raw;

mod job;

use job::{Job, JobDriver};

use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read};
use std::ops::Deref;
use std::ptr;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureType {
    MD4,
    Blake2,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Syntax,
    Mem,
    BadMagic,
    Unimplemented,
    Internal,
    Unknown(i32),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Signature<R> {
    driver: JobDriver<R>,
}

pub struct Delta<R> {
    driver: JobDriver<R>,
    _sumset: Sumset,
}


struct Sumset(*mut raw::rs_signature_t);


impl<R: Read> Signature<R> {
    pub fn new(old: R,
               new_block_len: usize,
               strong_len: usize,
               sig_magic: SignatureType)
               -> Result<Self> {
        let job = unsafe { raw::rs_sig_begin(new_block_len, strong_len, sig_magic.as_raw()) };
        if job.is_null() {
            return Err(Error::BadMagic);
        }
        Ok(Signature { driver: JobDriver::new(old, Job(job)) })
    }

    pub fn into_inner(self) -> R {
        self.driver.into_inner()
    }
}

impl<R: Read> Read for Signature<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.driver.read(buf)
    }
}


impl<R: Read> Delta<R> {
    pub fn new<S: Read>(input: R, signature: S) -> Result<Self> {
        // load the signature
        let sumset = unsafe {
            let mut sumset = ptr::null_mut();
            let job = raw::rs_loadsig_begin(&mut sumset);
            assert!(!job.is_null());
            let mut job = JobDriver::new(signature, Job(job));
            try!(job.consume_input());
            let sumset = Sumset(sumset);
            let res = raw::rs_build_hash_table(*sumset);
            if res != raw::RS_DONE {
                return Err(Error::from(res));
            }
            sumset
        };
        let job = unsafe { raw::rs_delta_begin(*sumset) };
        assert!(!job.is_null());
        Ok(Delta {
            driver: JobDriver::new(input, Job(job)),
            _sumset: sumset,
        })
    }
}

impl<R: Read> Read for Delta<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.driver.read(buf)
    }
}



impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Syntax => "syntax error",
            Error::Mem => "out of memory",
            Error::BadMagic => "bad magic number given",
            Error::Unimplemented => "unimplemented feature",
            Error::Internal => "internal error",
            Error::Unknown(_) => "unknown error from librsync",
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(fmt, "{}", e),
            Error::Unknown(n) => write!(fmt, "unknown error {} from native library", n),
            _ => write!(fmt, "{}", std::error::Error::description(self)),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<raw::rs_result> for Error {
    fn from(err: raw::rs_result) -> Error {
        match err {
            raw::RS_BLOCKED => io_err(io::ErrorKind::WouldBlock, "blocked waiting for more data"),
            raw::RS_IO_ERROR => io_err(io::ErrorKind::Other, "unknown IO error from librsync"),
            raw::RS_SYNTAX_ERROR => Error::Syntax,
            raw::RS_MEM_ERROR => Error::Mem,
            raw::RS_INPUT_ENDED => {
                io_err(io::ErrorKind::UnexpectedEof, "unexpected end of input file")
            }
            raw::RS_BAD_MAGIC => Error::BadMagic,
            raw::RS_UNIMPLEMENTED => Error::Unimplemented,
            raw::RS_CORRUPT => io_err(io::ErrorKind::InvalidData, "unbelievable value in stream"),
            raw::RS_INTERNAL_ERROR => Error::Internal,
            raw::RS_PARAM_ERROR => io_err(io::ErrorKind::InvalidInput, "bad parameter"),
            n => Error::Unknown(n),
        }
    }
}


impl SignatureType {
    pub fn from_raw(raw: raw::rs_magic_number) -> Option<Self> {
        match raw {
            raw::RS_MD4_SIG_MAGIC => Some(SignatureType::MD4),
            raw::RS_BLAKE2_SIG_MAGIC => Some(SignatureType::Blake2),
            _ => None,
        }
    }

    fn as_raw(&self) -> raw::rs_magic_number {
        match *self {
            SignatureType::MD4 => raw::RS_MD4_SIG_MAGIC,
            SignatureType::Blake2 => raw::RS_BLAKE2_SIG_MAGIC,
        }
    }
}


impl Drop for Sumset {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                raw::rs_free_sumset(self.0);
            }
        }
    }
}

impl Deref for Sumset {
    type Target = *mut raw::rs_signature_t;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


fn io_err<E>(kind: io::ErrorKind, e: E) -> Error
    where E: Into<Box<error::Error + Send + Sync>>
{
    Error::Io(io::Error::new(kind, e))
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::{Cursor, Read};

    const DATA: &'static str = "this is a string to be tested";
    const DATA2: &'static str = "this is another string to be tested";

    fn data_signature() -> Vec<u8> {
        vec![0x72, 0x73, 0x01, 0x36, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x05, 0x1b, 0x21,
             0x04, 0x8b, 0xad, 0x3c, 0xbd, 0x19, 0x09, 0x1d, 0x1b, 0x04, 0xf0, 0x9d, 0x1f, 0x64,
             0x31, 0xde, 0x15, 0xf4, 0x04, 0x87, 0x60, 0x96, 0x19, 0x50, 0x39]
    }


    #[test]
    fn signature() {
        let cursor = Cursor::new(DATA);
        let mut sig = Signature::new(cursor, 10, 5, SignatureType::MD4).unwrap();
        let mut signature = Vec::new();
        let read = sig.read_to_end(&mut signature).unwrap();
        assert_eq!(read, signature.len());
        assert_eq!(signature, data_signature());
    }

    #[test]
    fn delta() {
        let sig = data_signature();
        let sig = Cursor::new(sig);
        let input = Cursor::new(DATA2);
        let mut job = Delta::new(input, sig).unwrap();
        let mut delta = Vec::new();
        let read = job.read_to_end(&mut delta).unwrap();
        assert_eq!(read, delta.len());
    }
}
