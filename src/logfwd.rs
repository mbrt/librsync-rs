use std::sync::{ONCE_INIT, Once};
use std::ffi::CStr;

use libc::c_char;
use log::LogLevel;

use raw;


/// Manually initialize logging.
///
/// It is optional to call this function, and safe to do so more than once.
pub fn init() {
    static mut INIT: Once = ONCE_INIT;

    unsafe {
        INIT.call_once(|| {
            raw::rs_trace_to(trace);
        });
    }
}


extern "C" fn trace(level: raw::rs_loglevel, msg: *const c_char) {
    let level = match level {
        raw::RS_LOG_EMERG | raw::RS_LOG_ALERT | raw::RS_LOG_CRIT | raw::RS_LOG_ERR => {
            LogLevel::Error
        }
        raw::RS_LOG_WARNING => LogLevel::Warn,
        raw::RS_LOG_NOTICE | raw::RS_LOG_INFO => LogLevel::Info,
        raw::RS_LOG_DEBUG => LogLevel::Debug,
        _ => LogLevel::Error,
    };
    let msg = unsafe { CStr::from_ptr(msg).to_string_lossy() };
    log!(target: "librsync", level, "{}", msg);
}
