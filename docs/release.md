# Releasing

This repository is a Cargo workspace with two published crates:

* `librsync-sys`: the vendored librsync C sources and the raw FFI bindings;
* `librsync`: the safe Rust API, which depends on `librsync-sys`.

They are versioned independently. Only bump and publish the crates that changed
since their last release.

## 1. Pick the crates to release

The version in each manifest is the one already published, so a crate needs a
bump if it changed since its last release:

```sh
cargo search librsync    # the published librsync and librsync-sys versions
git log --oneline "$(git describe --tags --abbrev=0)"..HEAD --stat
```

Changes under `librsync-sys/` (including the submodule pointer) mean the sys
crate needs a release too. Changes under `src/` mean `librsync` does.

## 2. Bump the versions

Edit the `version` field of the crates being released:

* `Cargo.toml`, for `librsync`;
* `librsync-sys/Cargo.toml`, for `librsync-sys`.

If the release contains a `librsync-sys` fix that `librsync` relies on, also
tighten the requirement in `[workspace.dependencies]` in the root `Cargo.toml`,
so that users cannot resolve an older, unfixed `librsync-sys`:

```toml
librsync-sys = { version = "0.1.4", path = "librsync-sys" }
```

Then refresh the lock file with `cargo update --workspace`.

## 3. Verify

The vendored librsync sources live in a git submodule and are shipped inside the
`librsync-sys` package. A package built from a missing submodule fails to
compile for everyone, so make sure it is checked out first:

```sh
git submodule update --init --recursive
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

The C sources must show up in the `librsync-sys` package:

```sh
cargo package -p librsync-sys --list | grep 'librsync/src/.*\.c'
```

Then package both crates without uploading anything. The dry run builds
`librsync` against the freshly packaged `librsync-sys`, so it also catches
version bumps that don't fit together. Already published versions are only
warned about here, so this works no matter which crates are being released:

```sh
cargo publish --workspace --dry-run
```

## 4. Publish

Commit the version bumps, then publish the crates that got a new version. When
both did, a single command is enough: it publishes them in dependency order and
waits for the registry index in between, so `librsync` resolves the
`librsync-sys` version it was verified against.

```sh
cargo publish --workspace          # both crates got a new version
cargo publish -p librsync          # only the safe wrapper changed
cargo publish -p librsync-sys      # only the vendored sources changed
```

Cargo refuses to upload a version that is already on crates.io, so unlike the
dry run, `--workspace` fails outright when one of the two members was not
bumped.

## 5. Tag

Tags are named after the `librsync` version:

```sh
git tag v0.2.6
git push origin master --tags
```
