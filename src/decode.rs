//! High-level decode API.

use crate::basic;
use crate::packbits;
use crate::bitmap;

/// Decode basic RLE (compact byte format).
///
/// Returns `None` if the input is malformed.
pub fn basic_rle(data: &[u8]) -> Vec<u8> {
    basic::decode_compact(data).unwrap_or_default()
}

/// Decode PackBits.
///
/// Returns an empty vec if the input is malformed.
pub fn packbits(data: &[u8]) -> Vec<u8> {
    packbits::decode(data).unwrap_or_default()
}

/// Decode a bitmap RLE scanline.
///
/// Returns `None` if the data doesn't match the expected width.
pub fn bitmap_scanline(runs: &[u8], width: usize) -> Option<Vec<u8>> {
    bitmap::decode_scanline(runs, width)
}

/// Decode a full bitmap.
///
/// Returns `None` if the data is malformed.
pub fn bitmap_full(data: &[u8]) -> Option<Vec<Vec<u8>>> {
    bitmap::decode_bitmap(data)
}
