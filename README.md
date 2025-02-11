# Rust Bindings to libeditorconfig

This crate provides a safe Rust wrapper to the native `libeditorconfig` C library, using the [editorconfig-sys](https://github.com/toblux/editorconfig-sys) FFI bindings.

[![Build status](https://github.com/toblux/editorconfig-rs/actions/workflows/test.yml/badge.svg)](https://github.com/toblux/editorconfig-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/editorconfig-rs.svg)](https://crates.io/crates/editorconfig-rs)

## Dependencies

This crate uses `editorconfig-sys` which currently requires you to install `libeditorconfig`. Please refer to the [editorconfig-sys README](https://github.com/toblux/editorconfig-sys) for more information.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
editorconfig-rs = "0.2.3"
```

## Usage

Examples can be found in the [tests](tests/editorconfig.rs).
