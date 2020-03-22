//! Process whole files in a single call.
//!
//! Provides functions to compute signatures, delta and patches with a single call. Those functions
//! are useful if the application prefers to process whole files instead of using fine-grained APIs
//! over IO.
//!
//! If fine-grained control over IO is necessary, it is provided by `Signature`, `Delta` and
//! `Patch` structs.

use super::*;
use std::io::{self, BufRead, Read, Seek, Write};

/// Generates the signature of a basis input, and writes it out to an output stream.
///
/// This function will consume the given input stream and attempt to write the resulting signature
/// to the given output. In case of success, the number of bytes written is returned, otherwise
/// an error is reported.
///
/// The accepted arguments, among the input and output streams, are:
///
/// * `block_len`: the block size for signature generation, in bytes;
/// * `strong_len`: the truncated length of strong checksums, in bytes;
/// * `sig_type`: the signature format to be used.
pub fn signature_with_options<R: ?Sized, W: ?Sized>(
    input: &mut R,
    output: &mut W,
    block_len: usize,
    strong_len: usize,
    sig_type: SignatureType,
) -> Result<u64>
where
    R: BufRead,
    W: Write,
{
    let mut sig = Signature::with_options(input, block_len, strong_len, sig_type)?;
    let written = io::copy(&mut sig, output)?;
    Ok(written)
}

/// Generates the signature of a basis input, by using default settings.
///
/// This function will consume the given input stream and attempt to write the resulting signature
/// to the given output. In case of success, the number of bytes written is returned, otherwise
/// an error is reported. Default settings are used to produce the signature. BLAKE2 for the
/// hashing, 2048 bytes for the block length and full length for the strong signature size.
pub fn signature<R: ?Sized, W: ?Sized>(input: &mut R, output: &mut W) -> Result<u64>
where
    R: Read,
    W: Write,
{
    let mut sig = Signature::new(input)?;
    let written = io::copy(&mut sig, output)?;
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
/// To generate a signature, see the `signature` function, or the `Signature` struct.
pub fn delta<R: ?Sized, S: ?Sized, W: ?Sized>(
    new: &mut R,
    base_sig: &mut S,
    output: &mut W,
) -> Result<u64>
where
    R: Read,
    S: Read,
    W: Write,
{
    let mut delta = Delta::new(new, base_sig)?;
    let written = io::copy(&mut delta, output)?;
    Ok(written)
}

/// Applies a patch, relative to a basis, into an output stream.
///
/// This function will consume the base file and the new file delta inputs and writes to the given
/// output the patched input. In case of success, the number of bytes written is returned,
/// otherwise an error is reported. The `base` parameter is the input stream representing the base
/// file from which apply the patch. This stream must be seekable. The `delta` parameter is a
/// stream containing the delta between the base file and the new one. The output parameter will
/// be used to write the output.
///
/// To generate a delta, see the `delta` function, or the `Delta` struct.
pub fn patch<B: ?Sized, D: ?Sized, W: ?Sized>(
    base: &mut B,
    delta: &mut D,
    output: &mut W,
) -> Result<u64>
where
    B: Read + Seek,
    D: Read,
    W: Write,
{
    let mut patch = Patch::new(base, delta)?;
    let written = io::copy(&mut patch, output)?;
    Ok(written)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::SignatureType;

    use std::io::Cursor;
    use std::str::from_utf8;

    const DATA: &'static str = "this is a string to be tested";
    const DATA2: &'static str = "this is another string to be tested";

    #[test]
    fn integration() {
        // signature
        let mut sig = Vec::new();
        signature_with_options(
            &mut Cursor::new(DATA),
            &mut sig,
            10,
            5,
            SignatureType::Blake2,
        )
        .unwrap();

        // delta
        let mut dlt = Vec::new();
        delta(&mut Cursor::new(DATA2), &mut Cursor::new(sig), &mut dlt).unwrap();

        // patch
        let mut out = Vec::new();
        patch(&mut Cursor::new(DATA), &mut Cursor::new(dlt), &mut out).unwrap();

        // check that patched version is the same as DATA2
        let out_str = from_utf8(&out).unwrap();
        assert_eq!(out_str, DATA2);
    }
}
