# egzreader

[![Crates.io](https://img.shields.io/crates/v/egzreader.svg)](https://crates.io/crates/egzreader)
[![Documentation](https://docs.rs/egzreader/badge.svg)](https://docs.rs/egzreader)
[![License](https://img.shields.io/crates/l/egzreader)](https://crates.io/crates/egzreader)

Read gzip/non-gzip stream easily in Rust.

## Installation
```toml
# Cargo.toml
[dependencies]
egzreader = "2"
```

## Example
```rust
use std::io::prelude::*;
use std::io;
use std::fs::File;
use egzreader::EGZReader;

fn read_hello() -> io::Result<()> {
    // text file
    let mut r1 = EGZReader::new(
        File::open("examples/hello.txt")?
    );
    // gzip encoded text file
    let mut r2 = EGZReader::new(
        File::open("examples/hello.txt.gz")?
    );

    let mut s1 = String::new();
    let mut s2 = String::new();

    r1.read_to_string(&mut s1)?;
    r2.read_to_string(&mut s2)?;

    assert_eq!(s1, "Hello!");
    assert_eq!(s2, "Hello!");

    Ok(())
}
```
