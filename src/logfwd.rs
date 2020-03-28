use libc::c_char;
use std::sync::Once;

use crate::raw;

/// Manually initialize logging.
///
/// It is optional to call this function, and safe to do so more than once.
pub fn init() {
    static mut INIT: Once = Once::new();

    unsafe {
        INIT.call_once(|| {
            init_impl();
        });
    }
}

#[cfg(feature = "log")]
fn init_impl() {
    use log::LevelFilter;

    // trace to our callback
    unsafe {
        raw::rs_trace_to(trace);
    }

    // determine log level
    // this is useful because if the setted level is not Debug we can optimize librsync log
    // calls
    let level = match log::max_level() {
        LevelFilter::Info => raw::RS_LOG_NOTICE,
        LevelFilter::Debug | LevelFilter::Trace => raw::RS_LOG_DEBUG,
        _ => raw::RS_LOG_WARNING,
    };
    unsafe {
        raw::rs_trace_set_level(level);
    }
}

#[cfg(feature = "log")]
extern "C" fn trace(level: raw::rs_loglevel, msg: *const c_char) {
    use log::Level;
    use std::ffi::CStr;

    let level = match level {
        raw::RS_LOG_EMERG | raw::RS_LOG_ALERT | raw::RS_LOG_CRIT | raw::RS_LOG_ERR => Level::Error,
        raw::RS_LOG_WARNING => Level::Warn,
        raw::RS_LOG_NOTICE | raw::RS_LOG_INFO => Level::Info,
        raw::RS_LOG_DEBUG => Level::Debug,
        _ => Level::Error,
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
