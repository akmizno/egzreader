# egzreader

[![crates.io](https://img.shields.io/crates/v/egzreader.svg)](https://crates.io/crates/egzreader)
[![Documentation](https://docs.rs/egzreader/badge.svg)](https://docs.rs/egzreader)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A gzip and non-gzip pholymorphic reader in Rust.

## Installation
```toml
# Cargo.toml
[dependencies]
egzreader = "1"
```

## Example
```rust
use std::io::prelude::*;
use std::io;
use std::fs::File;
use egzreader::EGZReader;

fn read_hello() -> io::Result<()> {
    let mut r1 = EGZReader::new(File::open("examples/hello.txt")?);    // text file
    let mut r2 = EGZReader::new(File::open("examples/hello.txt.gz")?); // gzip encoded text file

    let mut s1 = String::new();
    let mut s2 = String::new();

    r1.read_to_string(&mut s1)?;
    r2.read_to_string(&mut s2)?;

    assert_eq!(s1, "Hello!");
    assert_eq!(s2, "Hello!");

    Ok(())
}
```
