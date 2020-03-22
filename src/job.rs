use std::io::{self, BufRead, Read};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;

use crate::{raw, Error};

pub struct JobDriver<R> {
    input: R,
    job: Job,
    input_ended: bool,
}

pub struct Job(pub *mut raw::rs_job_t);

// Wrapper around rs_buffers_t.
struct Buffers<'a> {
    inner: raw::rs_buffers_t,
    _phantom: PhantomData<&'a u8>,
}

impl<R: BufRead> JobDriver<R> {
    pub fn new(input: R, job: Job) -> Self {
        JobDriver {
            input,
            job,
            input_ended: false,
        }
    }

    pub fn into_inner(self) -> R {
        self.input
    }

    /// Complete the job by working without an output buffer.
    ///
    /// If the job needs to write some data, an `ErrorKind::WouldBlock` error is returned.
    pub fn consume_input(&mut self) -> io::Result<()> {
        loop {
            let (res, read, cap) = {
                let readbuf = self.input.fill_buf()?;
                let cap = readbuf.len();
                if cap == 0 {
                    self.input_ended = true;
                }

                // work
                let mut buffers = Buffers::with_no_out(readbuf, self.input_ended);
                let res = unsafe { raw::rs_job_iter(*self.job, buffers.as_raw()) };
                let read = cap - buffers.available_input();
                (res, read, cap - read)
            };
            // update read size
            self.input.consume(read);

            // determine result
            // NOTE: this should be done here, after the input buffer update, because we need to
            // know if the possible RS_BLOCKED result is due to a full input, or to an empty output
            // buffer
            match res {
                raw::RS_DONE => (),
                raw::RS_BLOCKED => {
                    if cap > 0 {
                        // the block is due to a missing output buffer
                        return Err(io::Error::new(
                            io::ErrorKind::WouldBlock,
                            "cannot consume input without an output buffer",
                        ));
                    }
                }
                _ => {
                    let err = Error::from(res);
                    return Err(io::Error::new(io::ErrorKind::Other, err));
                }
            };

            if self.input_ended {
                return Ok(());
            }
        }
    }
}

impl<R: BufRead> Read for JobDriver<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut out_pos = 0;
        let mut out_cap = buf.len();

        loop {
            let (res, read, written) = {
                let readbuf = self.input.fill_buf()?;
                let cap = readbuf.len();
                if cap == 0 {
                    self.input_ended = true;
                }

                // work
                let mut buffers = Buffers::new(readbuf, &mut buf[out_pos..], self.input_ended);
                let res = unsafe { raw::rs_job_iter(*self.job, buffers.as_raw()) };
                if res != raw::RS_DONE && res != raw::RS_BLOCKED {
                    let err = Error::from(res);
                    return Err(io::Error::new(io::ErrorKind::Other, err));
                }
                let read = cap - buffers.available_input();
                let written = out_cap - buffers.available_output();
                (res, read, written)
            };

            // update read size
            self.input.consume(read);
            // update write size
            out_pos += written;
            out_cap -= written;
            if out_cap == 0 || res == raw::RS_DONE {
                return Ok(out_pos);
            }
        }
    }
}

unsafe impl Send for Job {}

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

    pub fn with_no_out(in_buf: &'a [u8], eof_in: bool) -> Self {
        Buffers {
            inner: raw::rs_buffers_t {
                next_in: in_buf.as_ptr() as *const i8,
                avail_in: in_buf.len(),
                eof_in: if eof_in { 1 } else { 0 },
                next_out: ptr::null_mut(),
                avail_out: 0,
            },
            _phantom: PhantomData,
        }
    }

    pub fn as_raw(&mut self) -> *mut raw::rs_buffers_t {
        &mut self.inner
    }

    pub fn available_input(&self) -> usize {
        self.inner.avail_in
    }

    pub fn available_output(&self) -> usize {
        self.inner.avail_out
    }
}
