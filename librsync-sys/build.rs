extern crate gcc;

use std::env;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();
    let windows = target.contains("windows");

    let mut cfg = gcc::Config::new();

    if windows {
        cfg.define("_WIN32", None);
    }

    let cfg_dir = {
        let mut p = Path::new("librsync/static").to_path_buf();
        p.push(target);
        p
    };

    cfg.include(cfg_dir)
       .include("librsync/static")
       .include("librsync/src")
       .define("STDC_HEADERS", Some("1"))
       .file("librsync/static/prototab.c")
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
