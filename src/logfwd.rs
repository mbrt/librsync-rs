use std::sync::{ONCE_INIT, Once};
use std::ffi::CStr;

use libc::c_char;
use log::{self, LogLevel, LogLevelFilter};

use raw;


/// Manually initialize logging.
///
/// It is optional to call this function, and safe to do so more than once.
pub fn init() {
    static mut INIT: Once = ONCE_INIT;

    unsafe {
        INIT.call_once(|| {
            // trace to our callback
            raw::rs_trace_to(trace);

            // determine log level
            // this is useful because if the setted level is not Debug we can optimize librsync log
            // calls
            let level = match log::max_log_level() {
                LogLevelFilter::Info => raw::RS_LOG_NOTICE,
                LogLevelFilter::Debug | LogLevelFilter::Trace => raw::RS_LOG_DEBUG,
                _ => raw::RS_LOG_WARNING,
            };
            raw::rs_trace_set_level(level);
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
