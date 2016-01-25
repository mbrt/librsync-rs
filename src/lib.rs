extern crate librsync_sys as raw;

mod job;

use job::{Job, JobDriver};

use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read};
use std::ptr;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureType {
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
    Unknown(i32),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Signature<R: Read> {
    driver: JobDriver<R>,
}

pub struct Delta<R: Read> {
    driver: JobDriver<R>,
    sumset: Sumset,
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
        Ok(Signature { driver: JobDriver::new(old, Job(job)), })
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
    pub fn new<R2: Read>(input: R, signature: R2) -> Result<Self> {
        let sumset = unsafe {
            let mut sumset = ptr::null_mut();
            let job = raw::rs_loadsig_begin(&mut sumset);
            assert!(!job.is_null());
            let mut job = JobDriver::new(signature, Job(job));
            try!(job.consume_input());
            Sumset(sumset)
        };
        unimplemented!()
    }
}


impl error::Error for Error {
    fn description(&self) -> &str {
        ""
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(fmt, "{}", e),
            Error::Unknown(n) => write!(fmt, "Unknown error {} from librsync", n),
            _ => write!(fmt, "Error {:?} from librsync", self),
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
            raw::RS_DONE => Error::Done,
            raw::RS_BLOCKED => Error::Blocked,
            raw::RS_RUNNING => Error::Running,
            raw::RS_TEST_SKIPPED => Error::TestSkipped,
            raw::RS_IO_ERROR => Error::Io(other_io_err("Unknown IO error from librsync")),
            raw::RS_SYNTAX_ERROR => Error::Syntax,
            raw::RS_MEM_ERROR => Error::Mem,
            raw::RS_INPUT_ENDED => Error::InputEnded,
            raw::RS_BAD_MAGIC => Error::BadMagic,
            raw::RS_UNIMPLEMENTED => Error::Unimplemented,
            raw::RS_CORRUPT => Error::Corrupt,
            raw::RS_INTERNAL_ERROR => Error::Internal,
            raw::RS_PARAM_ERROR => Error::Param,
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
        let mut sig = Signature::new(cursor, 10, 5, SignatureType::MD4).unwrap();
        let mut signature = Vec::new();
        let read = sig.read_to_end(&mut signature).unwrap();
        assert_eq!(read, signature.len());
        let expected = vec![0x72, 0x73, 0x01, 0x36, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00,
                            0x05, 0x1b, 0x21, 0x04, 0x8b, 0xad, 0x3c, 0xbd, 0x19, 0x09, 0x1d,
                            0x1b, 0x04, 0xf0, 0x9d, 0x1f, 0x64, 0x31, 0xde, 0x15, 0xf4, 0x04,
                            0x87, 0x60, 0x96, 0x19, 0x50, 0x39];
        assert_eq!(signature, expected);
    }
}
