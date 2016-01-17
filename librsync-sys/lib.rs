#![allow(bad_style)]

extern crate libc;
use libc::*;

pub type rs_magic_number = c_int;
pub const RS_DELTA_MAGIC: c_int = 0x72730236;
pub const RS_MD4_SIG_MAGIC: c_int = 0x72730136;
pub const RS_BLAKE2_SIG_MAGIC: c_int = 0x72730137;


#[repr(C)]
pub struct rs_job_t {
    dogtag: c_int,
    job_name: *const c_char,
// TODO
//    int                 dogtag;
//    const char          *job_name;
//    rs_buffers_t *stream;
//    rs_result           (*statefn)(rs_job_t *);
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

extern {
    pub fn rs_sig_begin(new_block_len: size_t,
                        strong_sum_len: size_t,
                        sig_magic: rs_magic_number)
                        -> *mut rs_job_t;
}
