# librsync-rs
[![Build Status](https://travis-ci.org/mbrt/librsync-rs.svg?branch=master)](https://travis-ci.org/mbrt/librsync-rs)
[![Coverage Status](https://coveralls.io/repos/github/mbrt/librsync-rs/badge.svg?branch=master)](https://coveralls.io/github/mbrt/librsync-rs?branch=master)
[![](http://meritbadge.herokuapp.com/librsync)](https://crates.io/crates/librsync)

Rust bindings to [librsync](https://github.com/librsync/librsync).

[API Documentation](https://docs.rs/librsync)


## Introduction

This library contains bindings to librsync [1], to support computation and application of
network deltas, used in rsync and duplicity backup applications. This library encapsulates the
algorithms of the rsync protocol, which computes differences between files efficiently.

The rsync protocol, when computes differences, does not require the presence of both files.
It needs instead the new file and a set of checksums of the first file (namely the signature).
Computed differences can be stored in a delta file. The rsync protocol is then able to
reproduce the new file, by having the old one and the delta.

[1]: http://librsync.sourcefrog.net/


## Installation

Simply add a corresponding entry to your `Cargo.toml` dependency list:

```toml
[dependencies]
librsync = "0.2"
```

And add this to your crate root:

```rust
extern crate librsync;
```


## Overview of types and modules

This crate provides the streaming operations to produce signatures, delta and patches in the
top-level module, with `Signature`, `Delta` and `Patch` structs. Those structs take some input
stream (`Read` or `Read + Seek` traits) and implement another stream (`Read` trait) from which
the output can be read.

Higher level operations are provided within the `whole` submodule. If the application does not
need fine-grained control over IO operations, `sig`, `delta` and `patch` submodules can be
used. Those functions apply the algorithms to an output stream (implementing the `Write` trait)
in a single call.


## Example: streams

This example shows how to go through the streaming APIs, starting from an input string and a
modified string which act as old and new files. The example simulates a real world scenario, in
which the signature of a base file is computed, used as input to compute differences between
the base file and the new one, and finally the new file is reconstructed, by using the patch
and the base file.

```rust
extern crate librsync;

use std::io::prelude::*;
use std::io::Cursor;
use librsync::{Delta, Patch, Signature};

fn main() {
    let base = "base file".as_bytes();
    let new = "modified base file".as_bytes();

    // create signature starting from base file
    let mut sig = Signature::new(base).unwrap();
    // create delta from new file and the base signature
    let delta = Delta::new(new, &mut sig).unwrap();
    // create and store the new file from the base one and the delta
    let mut patch = Patch::new(Cursor::new(base), delta).unwrap();
    let mut computed_new = Vec::new();
    patch.read_to_end(&mut computed_new).unwrap();

    // test whether the computed file is exactly the new file, as expected
    assert_eq!(computed_new, new);
}
```

Note that intermediate results are not stored in temporary containers. This is possible because
the operations implement the `Read` trait. In this way the results does not need to be fully in
memory, during computation.


## Example: whole file API

This example shows how to go trough the whole file APIs, starting from an input string and a
modified string which act as old and new files. Unlike the streaming example, here we call a
single function, to get the computation result of signature, delta and patch operations. This
is convenient when an output stream (like a network socket or a file) is used as output for an
operation.

```rust
extern crate librsync;

use std::io::Cursor;
use librsync::whole::*;

fn main() {
    let base = "base file".as_bytes();
    let new = "modified base file".as_bytes();

    // signature
    let mut sig = Vec::new();
    signature(&mut Cursor::new(base), &mut sig).unwrap();

    // delta
    let mut dlt = Vec::new();
    delta(&mut Cursor::new(new), &mut Cursor::new(sig), &mut dlt).unwrap();

    // patch
    let mut out = Vec::new();
    patch(&mut Cursor::new(base), &mut Cursor::new(dlt), &mut out).unwrap();

    assert_eq!(out, new);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

This library uses [librsync](https://github.com/librsync/librsync), which comes with an
[LGPL-2.0](https://github.com/librsync/librsync/blob/master/COPYING) license. Please, be sure to fulfill librsync
licensing requirements before to use this library.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
