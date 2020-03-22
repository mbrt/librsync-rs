//! librsync bindings for Rust.
//!
//! This library contains bindings to librsync [1], encapsulating the algorithms of the rsync
//! protocol, which computes differences between files efficiently.
//!
//! The rsync protocol, when computes differences, does not require the presence of both files.
//! It needs instead the new file and a set of checksums of the first file (the signature).
//! Computed differences can be stored in a delta file. The rsync protocol is then able to
//! reproduce the new file, by having the old one and the delta.
//!
//! [1]: http://librsync.sourcefrog.net/
//!
//!
//! # Overview of types and modules
//!
//! This crate provides the streaming operations to produce signatures, delta and patches in the
//! top-level module with `Signature`, `Delta` and `Patch` structs. Those structs take some input
//! stream (`Read` or `Read + Seek` traits) and implement another stream (`Read` trait) from which
//! the output can be read.
//!
//! Higher level operations are provided within the `whole` submodule. If the application does not
//! need fine-grained control over IO operations, `signature`, `delta` and `patch` functions can be
//! used. Those functions apply the results to an output stream (implementing the `Write` trait)
//! in a single call.
//!
//!
//! # Example: streams
//!
//! This example shows how to go trough the streaming APIs, starting from an input string and a
//! modified string which act as old and new files. The example simulates a real world scenario, in
//! which the signature of a base file is computed, used as input to compute differencies between
//! the base file and the new one, and finally the new file is reconstructed, by using the base
//! file and the delta.
//!
//! ```rust
//! use std::io::prelude::*;
//! use std::io::Cursor;
//! use librsync::{Delta, Patch, Signature};
//!
//! let base = "base file".as_bytes();
//! let new = "modified base file".as_bytes();
//!
//! // create signature starting from base file
//! let mut sig = Signature::new(base).unwrap();
//! // create delta from new file and the base signature
//! let delta = Delta::new(new, &mut sig).unwrap();
//! // create and store the new file from the base one and the delta
//! let mut patch = Patch::new(Cursor::new(base), delta).unwrap();
//! let mut computed_new = Vec::new();
//! patch.read_to_end(&mut computed_new).unwrap();
//!
//! // test whether the computed file is exactly the new file, as expected
//! assert_eq!(computed_new, new);
//! ```
//!
//! Note that intermediate results are not stored in temporary containers. This is possible because
//! the operations implement the `Read` trait. In this way the results does not need to be fully in
//! memory, during computation.
//!
//!
//! # Example: whole file API
//!
//! This example shows how to go trough the whole file APIs, starting from an input string and a
//! modified string which act as old and new files. Unlike the streaming example, here we call a
//! single function, to get the computation result of signature, delta and patch operations. This
//! is convenient when an output stream (like a network socket or a file) is used as output for an
//! operation.
//!
//! ```rust
//! use std::io::Cursor;
//! use librsync::whole::*;
//!
//! let base = "base file".as_bytes();
//! let new = "modified base file".as_bytes();
//!
//! // signature
//! let mut sig = Vec::new();
//! signature(&mut Cursor::new(base), &mut sig).unwrap();
//!
//! // delta
//! let mut dlt = Vec::new();
//! delta(&mut Cursor::new(new), &mut Cursor::new(sig), &mut dlt).unwrap();
//!
//! // patch
//! let mut out = Vec::new();
//! patch(&mut Cursor::new(base), &mut Cursor::new(dlt), &mut out).unwrap();
//!
//! assert_eq!(out, new);
//! ```

#![deny(
    missing_copy_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(feature = "nightly", allow(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]

extern crate libc;
extern crate librsync_sys as raw;
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

mod job;
mod logfwd;
mod macros;
pub mod whole;

use crate::job::{Job, JobDriver};

use std::cell::{RefCell, RefMut};
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, BufRead, BufReader, Read, Seek};
use std::mem;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;
use std::slice;

/// The signature type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureType {
    /// A signature file with MD4 signatures.
    ///
    /// Backward compatible with librsync < 1.0, but deprecated because of a security
    /// vulnerability.
    MD4,
    /// A signature file using BLAKE2 hash.
    Blake2,
}

/// Enumeration of all possible errors in this crate.
#[derive(Debug)]
pub enum Error {
    /// An IO error.
    Io(io::Error),
    /// Out of memory.
    Mem,
    /// Bad magic number at start of stream.
    BadMagic,
    /// The feature is not available yet.
    Unimplemented,
    /// Probably a library bug.
    Internal,
    /// All the other error numbers.
    ///
    /// This error should never occur, as it is an indication of a bug.
    Unknown(i32),
}

/// A `Result` type alias for this crate's `Error` type.
pub type Result<T> = std::result::Result<T, Error>;

/// A struct to generate a signature.
///
/// This type takes a `Read` stream for the input from which compute the signatures, and implements
/// another `Read` stream from which get the result.
pub struct Signature<R> {
    driver: JobDriver<R>,
}

/// A struct to generate a delta between two files.
///
/// This type takes two `Read` streams, one for the signature of the base file and one for the new
/// file. It then provides another `Read` stream from which get the result.
pub struct Delta<R> {
    driver: JobDriver<R>,
    _sumset: Sumset,
}

/// A struct to apply a delta to a basis file, to recreate the new file.
///
/// This type takes a `Read + Seek` stream for the base file, and a `Read` stream for the delta
/// file. It then provides another `Read` stream from which get the resulting patched file.
pub struct Patch<'a, B: 'a, D> {
    driver: JobDriver<D>,
    base: Rc<RefCell<B>>,
    raw: Box<Rc<RefCell<dyn ReadAndSeek + 'a>>>,
}

struct Sumset(*mut raw::rs_signature_t);

// workaround for E0225
trait ReadAndSeek: Read + Seek {}
impl<T: Read + Seek> ReadAndSeek for T {}

impl<R: Read> Signature<BufReader<R>> {
    /// Creates a new signature stream with default parameters.
    ///
    /// This constructor takes an input stream for the file from which compute the signatures.
    /// Default options are used for the signature format: BLAKE2 for the hashing, 2048 bytes for
    /// the block length and full length for the strong signature size.
    pub fn new(input: R) -> Result<Self> {
        Self::with_options(input, raw::RS_DEFAULT_BLOCK_LEN, 0, SignatureType::Blake2)
    }

    /// Creates a new signature stream by specifying custom parameters.
    ///
    /// This constructor takes the input stream for the file from which compute the signatures, the
    /// size of checksum blocks as `block_len` parameter (larger values make the signature shorter
    /// and the delta longer), and the size of strong signatures in bytes as `strong_len`
    /// parameter. If it is non-zero the signature will be truncated to that amount of bytes.
    /// The last parameter specifies which version of the signature format to be used.
    pub fn with_options(
        input: R,
        block_len: usize,
        strong_len: usize,
        sig_magic: SignatureType,
    ) -> Result<Self> {
        Self::with_buf_read(BufReader::new(input), block_len, strong_len, sig_magic)
    }
}

impl<R: BufRead> Signature<R> {
    /// Creates a new signature stream by using a `BufRead`.
    ///
    /// This constructor takes an already built `BufRead` instance. Prefer this constructor if
    /// you already have a `BufRead` as input stream, since it avoids wrapping the input stream
    /// into another `BufRead` instance. See `with_options` constructor for details on the other
    /// parameters.
    pub fn with_buf_read(
        input: R,
        block_len: usize,
        strong_len: usize,
        sig_magic: SignatureType,
    ) -> Result<Self> {
        logfwd::init();

        let job = unsafe { raw::rs_sig_begin(block_len, strong_len, sig_magic.as_raw()) };
        if job.is_null() {
            return Err(Error::BadMagic);
        }
        Ok(Signature {
            driver: JobDriver::new(input, Job(job)),
        })
    }

    /// Unwraps this stream, returning the underlying input stream.
    pub fn into_inner(self) -> R {
        self.driver.into_inner()
    }
}

impl<R: BufRead> Read for Signature<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.driver.read(buf)
    }
}

impl<R: Read> Delta<BufReader<R>> {
    /// Creates a new delta stream.
    ///
    /// This constructor takes two `Read` streams for the new file (`new` parameter) and for the
    /// signatures of the base file (`base_sig` parameter). It produces a delta stream from which
    /// read the resulting delta file.
    pub fn new<S: Read + ?Sized>(new: R, base_sig: &mut S) -> Result<Self> {
        Self::with_buf_read(BufReader::new(new), base_sig)
    }
}

impl<R: BufRead> Delta<R> {
    /// Creates a new delta stream by using a `BufRead` as new file.
    ///
    /// This constructor specializes the `new` constructor by taking a `BufRead` instance as
    /// `new` parameter. Prefer this constructor if you already have a `BufRead` as input stream,
    /// since it avoids wrapping the input stream into another `BufRead` instance. See `new`
    /// constructor for more details on the parameters.
    pub fn with_buf_read<S: Read + ?Sized>(new: R, base_sig: &mut S) -> Result<Self> {
        logfwd::init();

        // load the signature
        let sumset = unsafe {
            let mut sumset = ptr::null_mut();
            let job = raw::rs_loadsig_begin(&mut sumset);
            assert!(!job.is_null());
            let mut job = JobDriver::new(BufReader::new(base_sig), Job(job));
            job.consume_input()?;
            let sumset = Sumset(sumset);
            let res = raw::rs_build_hash_table(*sumset);
            if res != raw::RS_DONE {
                return Err(Error::from(res));
            }
            sumset
        };
        let job = unsafe { raw::rs_delta_begin(*sumset) };
        if job.is_null() {
            return Err(io_err(
                io::ErrorKind::InvalidData,
                "invalid signature given",
            ));
        }
        Ok(Delta {
            driver: JobDriver::new(new, Job(job)),
            _sumset: sumset,
        })
    }

    /// Unwraps this stream, returning the underlying new file stream.
    pub fn into_inner(self) -> R {
        self.driver.into_inner()
    }
}

impl<R: BufRead> Read for Delta<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.driver.read(buf)
    }
}

impl<'a, B: Read + Seek + 'a, D: Read> Patch<'a, B, BufReader<D>> {
    /// Creates a new patch stream.
    ///
    /// This constructor takes a `Read + Seek` stream for the basis file (`base` parameter), and a
    /// `Read` stream for the delta file (`delta` parameter). It produces a stream from which read
    /// the resulting patched file.
    pub fn new(base: B, delta: D) -> Result<Self> {
        Self::with_buf_read(base, BufReader::new(delta))
    }
}

impl<'a, B: Read + Seek + 'a, D: BufRead> Patch<'a, B, D> {
    /// Creates a new patch stream by using a `BufRead` as delta stream.
    ///
    /// This constructor specializes the `new` constructor by taking a `BufRead` instance as
    /// `delta` parameter. Prefer this constructor if you already have a `BufRead` as input
    /// stream, since it avoids wrapping the input stream into another `BufRead` instance. See
    /// `new` constructor for more details on the parameters.
    pub fn with_buf_read(base: B, delta: D) -> Result<Self> {
        logfwd::init();

        let base = Rc::new(RefCell::new(base));
        let cb_data: Box<Rc<RefCell<dyn ReadAndSeek>>> = Box::new(base.clone());
        let job = unsafe { raw::rs_patch_begin(patch_copy_cb, mem::transmute(&*cb_data)) };
        assert!(!job.is_null());
        Ok(Patch {
            driver: JobDriver::new(delta, Job(job)),
            base,
            raw: cb_data,
        })
    }

    /// Unwraps this stream and returns the underlying streams.
    pub fn into_inner(self) -> (B, D) {
        // drop the secondary Rc before unwrapping the other
        {
            let _drop = self.raw;
        }
        let base = match Rc::try_unwrap(self.base) {
            Ok(base) => base,
            _ => unreachable!(),
        };
        (base.into_inner(), self.driver.into_inner())
    }
}

impl<'a, B, D: BufRead> Read for Patch<'a, B, D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.driver.read(buf)
    }
}

unsafe impl<'a, B: 'a, D> Send for Patch<'a, B, D>
where
    B: Send,
    D: Send,
{
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(fmt, "{}", e),
            Error::Mem => write!(fmt, "out of memory"),
            Error::BadMagic => write!(fmt, "bad magic number given"),
            Error::Unimplemented => write!(fmt, "unimplemented feature"),
            Error::Internal => write!(fmt, "internal error"),
            Error::Unknown(n) => write!(fmt, "unknown error {} from native library", n),
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
    fn as_raw(self) -> raw::rs_magic_number {
        match self {
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

unsafe impl Send for Sumset {}

extern "C" fn patch_copy_cb(
    opaque: *mut libc::c_void,
    pos: raw::rs_long_t,
    len: *mut libc::size_t,
    buf: *mut *mut libc::c_void,
) -> raw::rs_result {
    let mut input: RefMut<dyn ReadAndSeek> = unsafe {
        let h: *mut Rc<RefCell<dyn ReadAndSeek>> = mem::transmute(opaque);
        (*h).borrow_mut()
    };
    let output = unsafe {
        let buf: *mut u8 = mem::transmute(*buf);
        slice::from_raw_parts_mut(buf, *len)
    };
    try_or_rs_error!(input.seek(io::SeekFrom::Start(pos as u64)));
    try_or_rs_error!(input.read(output));
    raw::RS_DONE
}

fn io_err<E>(kind: io::ErrorKind, e: E) -> Error
where
    E: Into<Box<dyn error::Error + Send + Sync>>,
{
    Error::Io(io::Error::new(kind, e))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{Cursor, Read};
    use std::thread;

    const DATA: &'static str = "this is a string to be tested";
    const DATA2: &'static str = "this is another string to be tested";

    // generated with `rdiff signature -b 10 -S 5 data data.sig`
    fn data_signature() -> Vec<u8> {
        vec![
            0x72, 0x73, 0x01, 0x36, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x05, 0x1b, 0x21,
            0x04, 0x8b, 0xad, 0x3c, 0xbd, 0x19, 0x09, 0x1d, 0x1b, 0x04, 0xf0, 0x9d, 0x1f, 0x64,
            0x31, 0xde, 0x15, 0xf4, 0x04, 0x87, 0x60, 0x96, 0x19, 0x50, 0x39,
        ]
    }

    // generated with `rdiff delta data.sig data2 data2.delta`
    fn data2_delta() -> Vec<u8> {
        vec![
            0x72, 0x73, 0x02, 0x36, 0x10, 0x74, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61,
            0x6e, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x20, 0x45, 0x0a, 0x13, 0x00,
        ]
    }

    #[test]
    fn signature() {
        let cursor = Cursor::new(DATA);
        let mut sig = Signature::with_options(cursor, 10, 5, SignatureType::MD4).unwrap();
        let mut signature = Vec::new();
        let read = sig.read_to_end(&mut signature).unwrap();
        assert_eq!(read, signature.len());
        assert_eq!(signature, data_signature());
        sig.into_inner();
    }

    #[test]
    fn delta() {
        let sig = data_signature();
        let input = Cursor::new(DATA2);
        let mut job = Delta::new(input, &mut Cursor::new(sig)).unwrap();
        let mut delta = Vec::new();
        let read = job.read_to_end(&mut delta).unwrap();
        assert_eq!(read, delta.len());
        assert_eq!(delta, data2_delta());
        job.into_inner();
    }

    #[test]
    fn patch() {
        let base = Cursor::new(DATA);
        let delta = data2_delta();
        let delta = Cursor::new(delta);
        let mut patch = Patch::new(base, delta).unwrap();
        let mut computed_new = String::new();
        patch.read_to_string(&mut computed_new).unwrap();
        assert_eq!(computed_new, DATA2);
        patch.into_inner();
    }

    #[test]
    fn integration() {
        let base = Cursor::new(DATA);
        let new = Cursor::new(DATA2);
        let mut sig = Signature::with_options(base, 10, 5, SignatureType::MD4).unwrap();
        let delta = Delta::new(new, &mut sig).unwrap();
        let base = Cursor::new(DATA);
        let mut patch = Patch::new(base, delta).unwrap();
        let mut computed_new = String::new();
        patch.read_to_string(&mut computed_new).unwrap();
        assert_eq!(computed_new, DATA2);
    }

    #[test]
    fn send_sig() {
        let cursor = Cursor::new(DATA);
        let mut sig = Signature::new(cursor).unwrap();
        let t = thread::spawn(move || {
            let mut signature = Vec::new();
            sig.read_to_end(&mut signature).unwrap();
        });
        t.join().unwrap();
    }

    #[test]
    fn send_delta() {
        let sig = data_signature();
        let input = Cursor::new(DATA2);
        let mut job = Delta::new(input, &mut Cursor::new(sig)).unwrap();
        let t = thread::spawn(move || {
            let mut delta = Vec::new();
            job.read_to_end(&mut delta).unwrap();
        });
        t.join().unwrap();
    }

    #[test]
    fn send_patch() {
        let base = Cursor::new(DATA);
        let delta = data2_delta();
        let delta = Cursor::new(delta);
        let mut patch = Patch::new(base, delta).unwrap();
        let t = thread::spawn(move || {
            let mut computed_new = String::new();
            patch.read_to_string(&mut computed_new).unwrap();
        });
        t.join().unwrap();
    }

    #[test]
    fn trivial_large_file() {
        let data = vec![0; 65536];
        let mut sig =
            Signature::with_options(Cursor::new(&data), 16384, 5, SignatureType::MD4).unwrap();
        let delta = Delta::new(Cursor::new(&data), &mut sig).unwrap();
        let mut computed_new = vec![];
        Patch::new(Cursor::new(&data), delta)
            .unwrap()
            .read_to_end(&mut computed_new)
            .unwrap();
        assert_eq!(computed_new, data);
    }
}
