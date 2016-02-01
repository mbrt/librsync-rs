use std::sync::{ONCE_INIT, Once};
use libc::c_char;

use raw;


/// Manually initialize logging.
///
/// It is optional to call this function, and safe to do so more than once.
pub fn init() {
    static mut INIT: Once = ONCE_INIT;

    unsafe {
        INIT.call_once(|| {
            init_impl();
        });
    }
}


#[cfg(feature = "log")]
fn init_impl() {
    use log::{self, LogLevelFilter};

    // trace to our callback
    unsafe {
        raw::rs_trace_to(trace);
    }

    // determine log level
    // this is useful because if the setted level is not Debug we can optimize librsync log
    // calls
    let level = match log::max_log_level() {
        LogLevelFilter::Info => raw::RS_LOG_NOTICE,
        LogLevelFilter::Debug | LogLevelFilter::Trace => raw::RS_LOG_DEBUG,
        _ => raw::RS_LOG_WARNING,
    };
    unsafe {
        raw::rs_trace_set_level(level);
    }
}

#[cfg(feature = "log")]
extern "C" fn trace(level: raw::rs_loglevel, msg: *const c_char) {
    use std::ffi::CStr;
    use log::LogLevel;

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


#[cfg(not(feature = "log"))]
fn init_impl() {
    unsafe {
        raw::rs_trace_to(trace);
        raw::rs_trace_set_level(raw::RS_LOG_EMERG);
    }

    extern "C" fn trace(_level: raw::rs_loglevel, _msg: *const c_char) {}
}
