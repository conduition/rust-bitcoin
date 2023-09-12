//! Rust-Bitcoin IO Library
//!
//! Because the core `std::io` module is not yet exposed in `no-std` Rust, building `no-std`
//! applications which require reading and writing objects via standard traits is not generally
//! possible. While there is ongoing work to improve this situation, this module is not likely to
//! be available for applications with broad rustc version support for some time.
//!
//! Thus, this library exists to export a minmal version of `std::io`'s traits which `no-std`
//! applications may need. With the `std` feature, these traits are also implemented for the
//! `std::io` traits, allowing standard objects to be used wherever the traits from this crate are
//! required.
//!
//! This traits are not one-for-one drop-ins, but are as close as possible while still implementing
//! `std::io`'s traits without unnecessary complexity.

// Experimental features we need.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), not(feature = "core2")))]
compile_error!("At least one of std or core2 must be enabled");

#[cfg(feature = "std")]
pub use std::error;
#[cfg(not(feature = "std"))]
pub use core2::error;

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

/// Standard I/O stream definitions which are API-equivalent to `std`'s `io` module. See
/// [`std::io`] for more info.
pub mod io {
    #[cfg(all(not(feature = "std"), not(feature = "core2")))]
    compile_error!("At least one of std or core2 must be enabled");

    #[cfg(feature = "std")]
    pub use std::io::{Read, sink, Cursor, Take, Error, ErrorKind, Result};

    #[cfg(not(feature = "std"))]
    pub use core2::io::{Read, Cursor, Take, Error, ErrorKind, Result};

    /// A generic trait describing an output stream. See [`std::io::Write`] for more info.
    pub trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize>;
        fn flush(&mut self) -> Result<()>;

        #[inline]
        fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
            while !buf.is_empty() {
                match self.write(buf) {
                    Ok(0) => return Err(Error::new(ErrorKind::UnexpectedEof, "")),
                    Ok(len) => buf = &buf[len..],
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        }
    }

    #[cfg(feature = "std")]
    impl<W: std::io::Write> Write for W {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            <W as std::io::Write>::write(self, buf)
        }
        #[inline]
        fn flush(&mut self) -> Result<()> {
            <W as std::io::Write>::flush(self)
        }
    }

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    impl Write for alloc::vec::Vec<u8> {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            self.extend_from_slice(buf);
            Ok(buf.len())
        }
        #[inline]
        fn flush(&mut self) -> Result<()> { Ok(()) }
    }

    #[cfg(not(feature = "std"))]
    impl<'a> Write for &'a mut [u8] {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let cnt = core::cmp::min(self.len(), buf.len());
            self[..cnt].copy_from_slice(&buf[..cnt]);
            *self = &mut core::mem::take(self)[cnt..];
            Ok(cnt)
        }
        #[inline]
        fn flush(&mut self) -> Result<()> { Ok(()) }
    }
}
