use std::io::{self, Read};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;

use {Error, raw};


pub struct JobDriver<R: Read> {
    input: R,
    job: Job,
    buf: Vec<u8>,
    pos: usize,
    cap: usize,
    input_ended: bool,
}


pub struct Job(pub *mut raw::rs_job_t);

// Wrapper around rs_buffers_t.
struct Buffers<'a> {
    inner: raw::rs_buffers_t,
    _phantom: PhantomData<&'a u8>,
}


impl<R: Read> JobDriver<R> {
    pub fn new(input: R, job: Job) -> Self {
        JobDriver {
            input: input,
            job: job,
            buf: vec![0; raw::RS_DEFAULT_BLOCK_LEN],
            pos: 0,
            cap: 0,
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
            if self.cap == 0 && !self.input_ended {
                // need to read some data from input
                let read = try!(self.input.read(&mut self.buf));
                if read == 0 {
                    self.input_ended = true;
                }
                self.pos = 0;
                self.cap = read;
            }

            // work
            let mut buffers = Buffers::with_no_out(&self.buf[self.pos..self.pos + self.cap],
                                                   self.input_ended);
            let res = unsafe { raw::rs_job_iter(*self.job, buffers.as_raw()) };
            match res {
                raw::RS_DONE => (),
                raw::RS_BLOCKED => {
                    return Err(io::Error::new(io::ErrorKind::WouldBlock,
                                              "Cannot consume input without an output buffer"));
                }
                _ => {
                    let err = Error::from(res);
                    return Err(io::Error::new(io::ErrorKind::Other, err));
                }
            };

            // update buffer cap and pos
            let read = self.cap - buffers.available_input();
            self.pos += read;
            self.cap -= read;
            if self.input_ended {
                return Ok(());
            }
        }
    }
}

impl<R: Read> Read for JobDriver<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut out_pos = 0;
        let mut out_cap = buf.len();

        loop {
            if self.cap == 0 && !self.input_ended {
                // need to read some data from input
                let read = try!(self.input.read(&mut self.buf));
                if read == 0 {
                    self.input_ended = true;
                }
                self.pos = 0;
                self.cap = read;
            }

            // work
            let mut buffers = Buffers::new(&self.buf[self.pos..self.pos + self.cap],
                                           &mut buf[out_pos..],
                                           self.input_ended);
            let res = unsafe { raw::rs_job_iter(*self.job, buffers.as_raw()) };
            if res != raw::RS_DONE && res != raw::RS_BLOCKED {
                let err = Error::from(res);
                return Err(io::Error::new(io::ErrorKind::Other, err));
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
        self.inner.avail_in as usize
    }

    pub fn available_output(&self) -> usize {
        self.inner.avail_out as usize
    }
}
