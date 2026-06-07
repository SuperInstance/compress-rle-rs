//! Comprehensive test suite for compress-rle-rs.

#[cfg(test)]
mod tests {
    use crate::{basic, packbits, bitmap, encode, decode};

    // ── Basic RLE tests ─────────────────────────────────────────────────

    #[test]
    fn test_basic_encode_simple() {
        let pairs = basic::encode(b"AAABBC");
        assert_eq!(pairs, vec![(3, b'A'), (2, b'B'), (1, b'C')]);
    }

    #[test]
    fn test_basic_encode_empty() {
        assert_eq!(basic::encode(b""), vec![]);
    }

    #[test]
    fn test_basic_encode_single() {
        assert_eq!(basic::encode(b"A"), vec![(1, b'A')]);
    }

    #[test]
    fn test_basic_encode_all_same() {
        assert_eq!(basic::encode(b"AAAAA"), vec![(5, b'A')]);
    }

    #[test]
    fn test_basic_encode_no_runs() {
        let pairs = basic::encode(b"ABCDE");
        assert_eq!(pairs, vec![
            (1, b'A'), (1, b'B'), (1, b'C'), (1, b'D'), (1, b'E')
        ]);
    }

    #[test]
    fn test_basic_decode() {
        let data = basic::decode(&[(3, b'A'), (2, b'B'), (1, b'C')]);
        assert_eq!(data, b"AAABBC");
    }

    #[test]
    fn test_basic_decode_empty() {
        assert_eq!(basic::decode(&[]), Vec::<u8>::new());
    }

    #[test]
    fn test_basic_roundtrip() {
        let data = b"AAAAAABBBCCD";
        let pairs = basic::encode(data);
        let decoded = basic::decode(&pairs);
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_basic_compact_roundtrip() {
        let data = b"AAABBBBCCDDDDDDD";
        let compact = basic::encode_compact(data);
        let decoded = basic::decode_compact(&compact).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_basic_compact_malformed() {
        assert!(basic::decode_compact(&[0x01]).is_none());
    }

    #[test]
    fn test_basic_run_count() {
        assert_eq!(basic::run_count(b""), 0);
        assert_eq!(basic::run_count(b"AAA"), 1);
        assert_eq!(basic::run_count(b"AABB"), 2);
        assert_eq!(basic::run_count(b"ABCD"), 4);
    }

    #[test]
    fn test_basic_compression_ratio_best_case() {
        let data = vec![0x42u8; 1000];
        let (encoded, original) = basic::compression_ratio(&data);
        assert!(encoded < original, "best case should compress: {encoded} vs {original}");
    }

    #[test]
    fn test_basic_compression_ratio_worst_case() {
        let data: Vec<u8> = (0..200).map(|i| (i % 256) as u8).collect();
        let (encoded, original) = basic::compression_ratio(&data);
        assert!(encoded > original, "worst case should expand: {encoded} vs {original}");
    }

    // ── PackBits tests ──────────────────────────────────────────────────

    #[test]
    fn test_packbits_encode_decode() {
        let data = b"AAABBC";
        let encoded = packbits::encode(data);
        let decoded = packbits::decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_packbits_empty() {
        assert_eq!(packbits::encode(b""), Vec::<u8>::new());
    }

    #[test]
    fn test_packbits_all_same() {
        let data = vec![b'X'; 300];
        let encoded = packbits::encode(&data);
        let decoded = packbits::decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_packbits_no_runs() {
        let data = b"ABCDEF";
        let encoded = packbits::encode(data);
        let decoded = packbits::decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_packbits_mixed() {
        let data = b"AAAABBBCCDEEEEEFFFFFF";
        let encoded = packbits::encode(data);
        let decoded = packbits::decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_packbits_long_run() {
        let data = vec![b'A'; 200];
        let encoded = packbits::encode(&data);
        let decoded = packbits::decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_packbits_spec_example() {
        // Classic PackBits example
        let data: Vec<u8> = vec![
            0xAA, 0xAA, 0xAA, 0x80, 0x00, 0x2A, 0xAA, 0xAA, 0xAA, 0xAA,
            0x80, 0x00, 0x2A, 0x22, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
            0xAA, 0xAA, 0xAA, 0xAA,
        ];
        let encoded = packbits::encode(&data);
        let decoded = packbits::decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_packbits_malformed() {
        // Header says copy 3 bytes but only 1 available
        assert!(packbits::decode(&[0x02, 0x41]).is_none());
    }

    #[test]
    fn test_packbits_nop() {
        // 0x80 = nop
        let encoded = vec![0x80, 0x00, 0x41]; // nop, then literal 'A'
        let decoded = packbits::decode(&encoded);
        // The 0x80 is nop, then 0x00 means copy 1 byte: 0x41
        assert_eq!(decoded, Some(vec![0x41]));
    }

    #[test]
    fn test_packbits_encoded_size() {
        let data = b"AAAAAABBB";
        let size = packbits::encoded_size(data);
        let encoded = packbits::encode(data);
        assert_eq!(size, encoded.len());
    }

    // ── Bitmap RLE tests ────────────────────────────────────────────────

    #[test]
    fn test_bitmap_encode_scanline() {
        let scanline: Vec<u8> = vec![0,0,0,0,0,1,1,1,0,0];
        let encoded = bitmap::encode_scanline(&scanline);
        assert_eq!(encoded, vec![5, 3, 2]);
    }

    #[test]
    fn test_bitmap_decode_scanline() {
        let decoded = bitmap::decode_scanline(&[5, 3, 2], 10).unwrap();
        assert_eq!(decoded, vec![0,0,0,0,0,1,1,1,0,0]);
    }

    #[test]
    fn test_bitmap_scanline_roundtrip() {
        let scanline: Vec<u8> = vec![0,0,0,1,1,0,0,0,0,1,1,1,1];
        let encoded = bitmap::encode_scanline(&scanline);
        let decoded = bitmap::decode_scanline(&encoded, scanline.len()).unwrap();
        assert_eq!(scanline, decoded);
    }

    #[test]
    fn test_bitmap_scanline_all_zeros() {
        let scanline: Vec<u8> = vec![0; 20];
        let encoded = bitmap::encode_scanline(&scanline);
        let decoded = bitmap::decode_scanline(&encoded, 20).unwrap();
        assert_eq!(scanline, decoded);
    }

    #[test]
    fn test_bitmap_scanline_all_ones() {
        let scanline: Vec<u8> = vec![1; 15];
        let encoded = bitmap::encode_scanline(&scanline);
        let decoded = bitmap::decode_scanline(&encoded, 15).unwrap();
        assert_eq!(scanline, decoded);
    }

    #[test]
    fn test_bitmap_scanline_empty() {
        assert_eq!(bitmap::encode_scanline(&[]), Vec::<u8>::new());
    }

    #[test]
    fn test_bitmap_full_roundtrip() {
        let scanlines = vec![
            vec![0,0,0,1,1,0],
            vec![1,1,0,0,0,0],
            vec![0,0,0,0,0,0],
        ];
        let encoded = bitmap::encode_bitmap(&scanlines);
        let decoded = bitmap::decode_bitmap(&encoded).unwrap();
        assert_eq!(scanlines, decoded);
    }

    #[test]
    fn test_bitmap_decode_wrong_width() {
        assert!(bitmap::decode_scanline(&[5, 3], 10).is_none());
    }

    // ── High-level API tests ────────────────────────────────────────────

    #[test]
    fn test_encode_decode_basic() {
        let data = b"AAAAAABBBCC";
        let encoded = encode::basic_rle(data);
        let decoded = decode::basic_rle(&encoded);
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_encode_decode_packbits() {
        let data = b"AABBCCDDDDDEEEEE";
        let encoded = encode::packbits(data);
        let decoded = decode::packbits(&encoded);
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_encode_decode_bitmap_scanline() {
        let scanline: Vec<u8> = vec![0,0,1,1,1,0];
        let encoded = encode::bitmap_scanline(&scanline);
        let decoded = decode::bitmap_scanline(&encoded, 6).unwrap();
        assert_eq!(scanline, decoded);
    }

    #[test]
    fn test_encode_decode_bitmap_full() {
        let scanlines = vec![
            vec![0,1,0,1],
            vec![1,0,1,0],
        ];
        let encoded = encode::bitmap_full(&scanlines);
        let decoded = decode::bitmap_full(&encoded).unwrap();
        assert_eq!(scanlines, decoded);
    }
}
