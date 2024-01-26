# Rust Bindings to libeditorconfig

This crate provides a safe Rust wrapper to the native `libeditorconfig` C library, using the [editorconfig-sys](https://github.com/toblux/editorconfig-sys) FFI bindings.

## Dependencies

This crate uses `editorconfig-sys` which currently requires you to install `libeditorconfig`. Please refer to the [editorconfig-sys README](https://github.com/toblux/editorconfig-sys) for more information.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
editorconfig-rs = "0.1.0"
```

## Usage

Examples can be found in the [tests](tests/editorconfig.rs).
