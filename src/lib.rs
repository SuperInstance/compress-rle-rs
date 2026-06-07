//! # compress-rle-rs
//!
//! A pure-Rust run-length encoding library with multiple RLE variants.
//!
//! # Modules
//!
//! - [`basic`] — Basic RLE: `[(count, byte)]` pairs.
//! - [`packbits`] — PackBits encoding as specified by Apple / TIFF.
//! - [`bitmap`] — Modified RLE for 1-bit bitmap rows.
//! - [`encode`] — High-level encode API.
//! - [`decode`] — High-level decode API.
//!
//! # Quick Start
//!
//! ```
//! use compress_rle_rs::{encode, decode};
//!
//! let data = b"AAAAAABBBBCCD";
//! let encoded = encode::basic_rle(data);
//! let decoded = decode::basic_rle(&encoded);
//! assert_eq!(data.as_slice(), decoded.as_slice());
//! ```

pub mod basic;
pub mod packbits;
pub mod bitmap;
pub mod encode;
pub mod decode;

#[cfg(test)]
mod tests;
