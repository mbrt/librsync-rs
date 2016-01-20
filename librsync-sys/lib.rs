#![allow(bad_style)]

extern crate libc;
use libc::*;

pub type rs_magic_number = c_int;
pub const RS_DELTA_MAGIC: c_int = 0x72730236;
pub const RS_MD4_SIG_MAGIC: c_int = 0x72730136;
pub const RS_BLAKE2_SIG_MAGIC: c_int = 0x72730137;

pub type rs_result = c_int;
pub const RS_DONE: c_int = 0;
pub const RS_BLOCKED: c_int = 1;
pub const RS_RUNNING: c_int = 2;
pub const RS_TEST_SKIPPED: c_int = 77;
pub const RS_IO_ERROR: c_int = 100;
pub const RS_SYNTAX_ERROR: c_int = 101;
pub const RS_MEM_ERROR: c_int = 102;
pub const RS_INPUT_ENDED: c_int = 103;
pub const RS_BAD_MAGIC: c_int = 104;
pub const RS_UNIMPLEMENTED: c_int = 105;
pub const RS_CORRUPT: c_int = 106;
pub const RS_INTERNAL_ERROR: c_int = 107;
pub const RS_PARAM_ERROR: c_int = 108;

pub const RS_DEFAULT_BLOCK_LEN: size_t = 2048;

pub type rs_long_t = c_longlong;

pub enum rs_job_t {}
pub enum rs_signature_t {}

#[repr(C)]
pub struct rs_buffers_t {
    pub next_in: *const c_char,
    pub avail_in: size_t,
    pub eof_in: c_int,
    pub next_out: *mut c_char,
    pub avail_out: size_t,
}

pub type rs_driven_cb = extern "C" fn(job: *mut rs_job_t,
                                      buf: *mut rs_buffers_t,
                                      opaque: *mut c_void)
                                      -> rs_result;
pub type rs_copy_cb = extern "C" fn(opaque: *mut c_void,
                                    pos: rs_long_t,
                                    len: *mut size_t,
                                    buf: *mut *mut c_void)
                                    -> rs_result;


extern "C" {
    pub fn rs_job_iter(job: *mut rs_job_t, buffers: *mut rs_buffers_t) -> rs_result;
    pub fn fs_job_drive(job: *mut rs_job_t,
                        buf: *mut rs_buffers_t,
                        in_cb: rs_driven_cb,
                        in_opaque: *mut c_void,
                        out_cb: rs_driven_cb,
                        out_opaque: *mut c_void)
                        -> rs_result;
    pub fn rs_job_free(job: *mut rs_job_t) -> rs_result;

    pub fn rs_sig_begin(new_block_len: size_t,
                        strong_sum_len: size_t,
                        sig_magic: rs_magic_number)
                        -> *mut rs_job_t;
    pub fn rs_delta_begin(sig: *mut rs_signature_t) -> *mut rs_job_t;
    pub fn rs_loadsig_begin(sig: *mut *mut rs_signature_t) -> *mut rs_job_t;
    pub fn rs_build_hash_table(sums: *mut rs_signature_t) -> rs_result;
    pub fn rs_patch_begin(copy_cb: rs_copy_cb, copy_arg: *mut c_void) -> *mut rs_job_t;
}
