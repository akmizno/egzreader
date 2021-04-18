//! A gzip and non-gzip pholymorphic reader.
//!
//! [EGZReader](EGZReader) decodes the underlying reader when it is gzipped stream, and
//! reads as it is when non-gzipped.
//!
//! # Examples
//! ```
//! use std::io::prelude::*;
//! use std::io;
//! use std::fs::File;
//! use egzreader::EGZReader;
//!
//! # fn main() {
//! #     read_hello().unwrap();
//! # }
//! fn read_hello() -> io::Result<()> {
//!     let mut r1 = EGZReader::new(File::open("examples/hello.txt")?);    // text file
//!     let mut r2 = EGZReader::new(File::open("examples/hello.txt.gz")?); // gzip encoded text file
//!
//!     let mut s1 = String::new();
//!     let mut s2 = String::new();
//!
//!     r1.read_to_string(&mut s1)?;
//!     r2.read_to_string(&mut s2)?;
//!
//!     assert_eq!(s1, "Hello!");
//!     assert_eq!(s2, "Hello!");
//!
//!     Ok(())
//! }
//! ```
use flate2::read::GzDecoder;
use std::io::Read;
use std::io::Result;
use std::mem;

struct RawReader<R> {
    preread: [u8; 11],
    pos: usize,
    size: usize,

    reader: R,
}
impl<R: Read> RawReader<R> {
    fn new(preread: [u8; 11], size: usize, r: R) -> RawReader<R> {
        debug_assert!(size <= preread.len());
        RawReader {
            preread: preread,
            pos: 0,
            size: size,
            reader: r,
        }
    }
}
impl<R: Read> Read for RawReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        debug_assert!(self.pos <= self.preread.len());

        if self.pos == self.size {
            self.reader.read(buf)
        } else {
            debug_assert!(self.pos < self.size);

            let n = (&self.preread[self.pos..self.size]).read(buf)?;
            self.pos += n;

            if n < buf.len() {
                match self.read(&mut buf[n..]) {
                    Err(e) => {
                        self.pos -= n; // Reset self.pos
                        Err(e)
                    }
                    Ok(m) => Ok(n + m),
                }
            } else {
                Ok(n)
            }
        }
    }
}

// Wrapper for flate2::GzDecoder
struct GzReader<R> {
    reader: GzDecoder<RawReader<R>>,
}
impl<R: Read> GzReader<R> {
    fn new(preread: [u8; 11], r: R) -> GzReader<R> {
        GzReader {
            reader: GzDecoder::new(RawReader::new(preread, 11, r)),
        }
    }
}
impl<R: Read> Read for GzReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
}

enum ReaderType<R> {
    // Initial state
    Init(R),

    // Actual reader states
    Zero,
    Raw(RawReader<R>), // non-gzip stream
    Gz(GzReader<R>),   // gzip stream
}

impl<R: Read> ReaderType<R> {
    fn is_init(&self) -> bool {
        match self {
            ReaderType::Init(_) => true,
            _ => false,
        }
    }

    fn make_reader(mut reader: R) -> Result<ReaderType<R>> {
        let mut buf = [0; 11];
        let n = reader.read(&mut buf)?;

        if n == 0 {
            Ok(ReaderType::Zero)
        } else if n == 11 && buf[..2] == [0x1f, 0x8b] && buf[2] <= 0x08 {
            // The underlying stream is assumed as gzip when
            // - more than 10 bytes (=header size) can be read.
            // - it begins with magic number '0x1f0x8b'.
            // - its third byte, specifying compression method, would be '0x08'.
            Ok(ReaderType::Gz(GzReader::new(buf, reader)))
        } else {
            Ok(ReaderType::Raw(RawReader::new(buf, n, reader)))
        }
    }

    // Determine actual type of reader.
    // This method is called at first read().
    fn into_actual_reader(self) -> Result<Self> {
        debug_assert!(self.is_init());
        if let ReaderType::Init(r) = self {
            Self::make_reader(r)
        } else {
            Ok(self)
        }
    }
}

impl<R: Read> Read for ReaderType<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            ReaderType::Init(_) => {
                // Update reader state.
                let init = mem::replace(self, ReaderType::Zero);
                *self = init.into_actual_reader()?;

                // Then, call read().
                debug_assert!(!self.is_init());
                self.read(buf)
            }
            ReaderType::Zero => Ok(0),
            ReaderType::Raw(raw) => raw.read(buf),
            ReaderType::Gz(gz) => gz.read(buf),
        }
    }
}

/// A gzip and non-gzip pholymorphic reader.
pub struct EGZReader<R> {
    reader: ReaderType<R>,
}
impl<R: Read> EGZReader<R> {
    pub fn new(r: R) -> EGZReader<R> {
        EGZReader {
            reader: ReaderType::Init(r),
        }
    }
}
impl<R: Read> Read for EGZReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::EGZReader;

    // "Hello!"
    const HELLO: &[u8] = &[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x21];

    // "Hello!" encoded by gzip
    const HELLO_GZ: &[u8] = &[
        0x1f, 0x8b, 0x08, 0x00, 0xeb, 0x47, 0x74, 0x60, 0x00, 0x03, 0xf3, 0x48, 0xcd, 0xc9, 0xc9,
        0x57, 0x04, 0x00, 0x56, 0xcc, 0x2a, 0x9d, 0x06, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn read_zero() {
        let data: &[u8] = &[0; 0];
        let mut r = EGZReader::new(data);
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert_eq!(s, "");
    }
    #[test]
    fn read_long() {
        let data: &[u8] = &[0x41; 20];
        let mut r = EGZReader::new(data);
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert_eq!(s, "AAAAAAAAAAAAAAAAAAAA");
    }
    #[test]
    fn read_hello_txt() {
        let mut r = EGZReader::new(HELLO);
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert_eq!(s, "Hello!");
    }
    #[test]
    fn read_hello_gz() {
        let mut r = EGZReader::new(HELLO_GZ);
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert_eq!(s, "Hello!");
    }
    #[test]
    fn read_fake_gz() {
        let mut r = EGZReader::new(&HELLO_GZ[..10]);
        let mut buf = [0; 11];
        let n = r.read(&mut buf).unwrap();
        assert_eq!(buf[..n], HELLO_GZ[..10]);
    }
}
