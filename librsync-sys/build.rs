extern crate gcc;

use std::env;

fn main() {
    let mut cfg = gcc::Config::new();

    if env::var("TARGET").unwrap().contains("windows") {
        cfg.define("_WIN32", None);
    }

    cfg.include("config")
       .include("prototab")
       .include("librsync/src")
       .define("STDC_HEADERS", Some("1"))
       .define("DO_RS_TRACE", Some("0"))
       .define("HAVE_PROGRAM_INVOCATION_NAME", Some("0"))
       .define("HAVE_VARARG_MACROS", Some("0"))
       .file("prototab/prototab.c")
       .file("librsync/src/base64.c")
       .file("librsync/src/buf.c")
       .file("librsync/src/checksum.c")
       .file("librsync/src/command.c")
       .file("librsync/src/delta.c")
       .file("librsync/src/emit.c")
       .file("librsync/src/fileutil.c")
       .file("librsync/src/hex.c")
       .file("librsync/src/job.c")
       .file("librsync/src/mdfour.c")
       .file("librsync/src/mksum.c")
       .file("librsync/src/msg.c")
       .file("librsync/src/netint.c")
       .file("librsync/src/patch.c")
       .file("librsync/src/readsums.c")
       .file("librsync/src/rollsum.c")
       .file("librsync/src/scoop.c")
       .file("librsync/src/search.c")
       .file("librsync/src/stats.c")
       .file("librsync/src/stream.c")
       .file("librsync/src/sumset.c")
       .file("librsync/src/trace.c")
       .file("librsync/src/tube.c")
       .file("librsync/src/util.c")
       .file("librsync/src/version.c")
       .file("librsync/src/whole.c")
       .file("librsync/src/blake2b-ref.c")
       .compile("librsync.a");
}
