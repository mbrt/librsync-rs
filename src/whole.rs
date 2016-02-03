//! Process whole files with a single call
//!
//! Provides functions to compute signatures, delta and patches with a single call. Those functions
//! are useful if the application prefers to process whole files instead of using fine-grained APIs
//! over IO.
//!
//! If fine-grained control over IO is necessary, it is provided by `Signature`, `Delta` and
//! `Patch` structs.

use std::io::{self, Read, Seek, Write};
use super::*;


/// Generates the signature of a basis input, and writes it out to an output stream.
///
/// This function will consume the given input stream and attempt to write the resulting signature
/// to the given output. In case of success, the number of bytes written is returned, otherwise
/// an error is reported.
///
/// The accepted arguments, among the input and output streams, are:
/// * `block_len`: the block size for signature generation, in bytes;
/// * `strong_len`: the truncated length of strong checksums, in bytes;
/// * `sig_type`: the signature format to be used.
pub fn sig<R, W: ?Sized>(input: R,
                         output: &mut W,
                         block_len: usize,
                         strong_len: usize,
                         sig_type: SignatureType)
                         -> Result<u64>
    where R: Read,
          W: Write
{
    let mut signature = try!(Signature::with_options(input, block_len, strong_len, sig_type));
    let written = try!(io::copy(&mut signature, output));
    Ok(written)
}


/// Generates a delta between a signature and a new file streams.
///
/// This function will consume the new file and base signature inputs and writes to the given
/// output the delta between them. In case of success, the number of bytes written is returned,
/// otherwise an error is reported. The `new` parameter is the input stream representing a possibly
/// modified file with respect to some base, for which its signature is provided as `base_sig`
/// parameter.
///
/// To generate a signature, see the `sig` function, or the `Signature` struct.
pub fn delta<R, S, W: ?Sized>(new: R, base_sig: S, output: &mut W) -> Result<u64>
    where R: Read,
          S: Read,
          W: Write
{
    let mut delta = try!(Delta::new(new, base_sig));
    let written = try!(io::copy(&mut delta, output));
    Ok(written)
}


/// Applies a patch, relative to a basis, into a new file.
///
/// This function will consume the base file and the new file delta inputs and writes to the given
/// output the patched input. In case of success, the number of bytes written is returned,
/// otherwise an error is reported. The `base` parameter is the input stream representing the base
/// file from which apply the patch. This stream must be seekable. The `delta` parameter is a
/// stream containing the delta between the base file and the new one. The output parameter will
/// be used to write the output.
///
/// To generate a delta, see the `delta` function, or the `Delta` struct.
pub fn patch<B, D, W: ?Sized>(base: B, delta: D, output: &mut W) -> Result<u64>
    where B: Read + Seek,
          D: Read,
          W: Write
{
    let mut patch = try!(Patch::new(base, delta));
    let written = try!(io::copy(&mut patch, output));
    Ok(written)
}


#[cfg(test)]
mod test {
    use super::*;
    use SignatureType;

    use std::io::Cursor;
    use std::str::from_utf8;

    const DATA: &'static str = "this is a string to be tested";
    const DATA2: &'static str = "this is another string to be tested";


    #[test]
    fn integration() {
        // signature
        let base = Cursor::new(DATA);
        let mut signature: Vec<u8> = Vec::new();
        sig(base, &mut signature, 10, 5, SignatureType::Blake2).unwrap();

        // delta
        let sig_in = Cursor::new(signature);
        let new = Cursor::new(DATA2);
        let mut dlt: Vec<u8> = Vec::new();
        delta(new, sig_in, &mut dlt).unwrap();

        // patch
        let base = Cursor::new(DATA);
        let dlt = Cursor::new(dlt);
        let mut out: Vec<u8> = Vec::new();
        patch(base, dlt, &mut out).unwrap();
        let out_str = from_utf8(&out).unwrap();

        assert_eq!(out_str, DATA2);
    }
}
