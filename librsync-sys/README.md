# librsync-sys
Building and wrapping librsync native library.

This library gets rid of librsync build system and provides static configurations for the most used platforms. In this way we can avoid CMake and Perl dependencies for the library users.

## Porting

This library currently supports the following targets:

* `i686-pc-windows-gnu`;
* `i686-pc-windows-msvc`:
* `i686-unknown-linux-gnu`;
* `x86_64-apple-darwin`;
* `x86_64-pc-windows-gnu`;
* `x86_64-pc-windows-msvc`;
* `x86_64-unknown-linux-gnu`.

To port the library to another target, use the utility in [mbrt/librsync](https://github.com/mbrt/librsync/tree/static_config/gen). Run that utility with the Rust toolchain you want to use:

```
cd librsync/gen
cargo run --target <your-target>
```

To do so, you need to have CMake and Perl installed and available in your PATH. If all goes well you will find the specific configuration for your platform, under `static` folder in that repo. Please submit a PR against the `static_config` branch in [mbrt/librsync](https://github.com/mbrt/librsync) by committing only that folder.

After that, `librsync-rs` will have the corresponding configuration available.
