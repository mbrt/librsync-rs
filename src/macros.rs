#![macro_use]

macro_rules! try_or_rs_error(
    ($e:expr) => (
        match $e {
            Ok(v) => v,
            _ => {
                return raw::RS_IO_ERROR;
            }
        }
    )
);
