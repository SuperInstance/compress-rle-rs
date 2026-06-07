//! High-level encode API.

use crate::basic;
use crate::packbits;
use crate::bitmap;

/// Encode using basic RLE (compact byte format).
pub fn basic_rle(data: &[u8]) -> Vec<u8> {
    basic::encode_compact(data)
}

/// Encode using PackBits.
pub fn packbits(data: &[u8]) -> Vec<u8> {
    packbits::encode(data)
}

/// Encode a single scanline using bitmap RLE.
pub fn bitmap_scanline(scanline: &[u8]) -> Vec<u8> {
    bitmap::encode_scanline(scanline)
}

/// Encode a full bitmap (multiple scanlines).
pub fn bitmap_full(scanlines: &[Vec<u8>]) -> Vec<u8> {
    bitmap::encode_bitmap(scanlines)
}
