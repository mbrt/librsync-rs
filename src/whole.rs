use std::io::{self, Read, Seek, Write};
use super::*;


pub fn sig<R, W: ?Sized>(input: R,
                         output: &mut W,
                         block_len: usize,
                         strong_len: usize,
                         sig_type: SignatureType)
                         -> Result<u64>
    where R: Read,
          W: Write
{
    let mut signature = try!(Signature::new(input, block_len, strong_len, sig_type));
    let written = try!(io::copy(&mut signature, output));
    Ok(written)
}

pub fn delta<R, S, W: ?Sized>(new: R, base_sig: S, output: &mut W) -> Result<u64>
    where R: Read,
          S: Read,
          W: Write
{
    let mut delta = try!(Delta::new(new, base_sig));
    let written = try!(io::copy(&mut delta, output));
    Ok(written)
}

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
