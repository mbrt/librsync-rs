# librsync-sys
Building and wrapping librsync native library.

## Porting

First of all I built the library, as librsync authors configured, by using CMake. This required my environment to have `popt`, `bzip2` and `zlib` development libraries and `perl` executable. Then I copied generated files from `build/src` directory into `prototab` directory. This allowed me to remove configure script and perl calls. The configured files needs to be provided for every platform we need to support.
