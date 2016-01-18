#![allow(bad_style)]

extern crate libc;
use libc::*;

pub type rs_magic_number = c_int;
pub const RS_DELTA_MAGIC: c_int = 0x72730236;
pub const RS_MD4_SIG_MAGIC: c_int = 0x72730136;
pub const RS_BLAKE2_SIG_MAGIC: c_int = 0x72730137;

pub type rs_result = c_int;
/// Completed successfully.
pub const RS_DONE: c_int = 0;
/// Blocked waiting for more data.
pub const RS_BLOCKED: c_int = 1;
/// The job is still running; and not yet finished or blocked.
///
/// (This value should never be seen by the application.)
pub const RS_RUNNING: c_int = 2;
/// Test neither passed or failed.
pub const RS_TEST_SKIPPED: c_int = 77;
/// Error in file or network IO.
pub const RS_IO_ERROR: c_int = 100;
/// Command line syntax error.
pub const RS_SYNTAX_ERROR: c_int = 101;
/// Out of memory.
pub const RS_MEM_ERROR: c_int = 102;
/// Unexpected end of input file.
///
/// Perhaps due to a truncated file or dropped network connection.
pub const RS_INPUT_ENDED: c_int = 103;
/// Bad magic number at start of stream.
///
/// Probably not a librsync file; or possibly the wrong kind of file or from an incompatible
/// library version.
pub const RS_BAD_MAGIC: c_int = 104;
/// Author is lazy.
pub const RS_UNIMPLEMENTED: c_int = 105;
/// Unbelievable value in stream.
pub const RS_CORRUPT: c_int = 106;
/// Probably a library bug.
pub const RS_INTERNAL_ERROR: c_int = 107;
/// Bad value passed in to library.
///
/// Probably an application bug.
pub const RS_PARAM_ERROR: c_int = 108;

#[repr(C)]
pub struct rs_job_t {
    pub dogtag: c_int,
    pub job_name: *const c_char,
    pub stream: *mut rs_buffers_t,
    pub statefn: Option<extern fn(*mut rs_job_t) -> rs_result>,
// TODO
//    rs_result final_result;
//    int                 block_len;
//    int                 strong_sum_len;
//    rs_signature_t      *signature;
//    unsigned char       op;
//    rs_weak_sum_t       weak_sig;
//    Rollsum             weak_sum;
//    rs_long_t           param1, param2;
//    struct rs_prototab_ent const *cmd;
//    rs_mdfour_t      output_md4;
//    rs_stats_t          stats;
//    rs_byte_t   *scoop_buf;          /* the allocation pointer */
//    rs_byte_t   *scoop_next;         /* the data pointer */
//    size_t      scoop_alloc;           /* the allocation size */
//    size_t      scoop_avail;           /* the data size */
//    size_t      scoop_pos;             /* the scan position */
//    rs_byte_t   write_buf[36];
//    int         write_len;
//    rs_long_t   copy_len;
//    rs_long_t       basis_pos, basis_len;
//    rs_copy_cb      *copy_cb;
//    void            *copy_arg;
//    int             magic;
}

pub struct rs_buffers_t {
    pub next_in: *mut c_char,
    pub avail_in: size_t,
    pub eof_in: c_int,
    pub next_out: *mut c_char,
    pub avail_out: size_t,
}


extern {
    pub fn rs_sig_begin(new_block_len: size_t,
                        strong_sum_len: size_t,
                        sig_magic: rs_magic_number)
                        -> *mut rs_job_t;
}
