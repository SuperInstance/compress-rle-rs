//! Basic run-length encoding: `[(count, byte)]` pairs.

/// A run-length pair: `(count, byte_value)`.
pub type RlePair = (usize, u8);

/// Encode data into basic RLE pairs.
///
/// Consecutive identical bytes are grouped into `(count, byte)` pairs.
///
/// # Examples
///
/// ```
/// use compress_rle_rs::basic;
///
/// let pairs = basic::encode(b"AAABBC");
/// assert_eq!(pairs, vec![(3, b'A'), (2, b'B'), (1, b'C')]);
/// ```
pub fn encode(data: &[u8]) -> Vec<RlePair> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut pairs = Vec::new();
    let mut current = data[0];
    let mut count = 1;

    for &byte in &data[1..] {
        if byte == current {
            count += 1;
        } else {
            pairs.push((count, current));
            current = byte;
            count = 1;
        }
    }
    pairs.push((count, current));
    pairs
}

/// Decode basic RLE pairs back into bytes.
///
/// # Examples
///
/// ```
/// use compress_rle_rs::basic;
///
/// let data = basic::decode(&[(3, b'A'), (2, b'B'), (1, b'C')]);
/// assert_eq!(data, b"AAABBC");
/// ```
pub fn decode(pairs: &[RlePair]) -> Vec<u8> {
    let total: usize = pairs.iter().map(|(c, _)| *c).sum();
    let mut out = Vec::with_capacity(total);
    for &(count, byte) in pairs {
        for _ in 0..count {
            out.push(byte);
        }
    }
    out
}

/// Encode into a compact byte format: `[count_byte, value_byte, ...]`.
///
/// Each run uses 2 bytes. Runs longer than 255 are split into multiple pairs.
pub fn encode_compact(data: &[u8]) -> Vec<u8> {
    let pairs = encode(data);
    let mut out = Vec::with_capacity(pairs.len() * 2);
    for (mut count, byte) in pairs {
        while count > 0 {
            let chunk = count.min(255);
            out.push(chunk as u8);
            out.push(byte);
            count -= chunk;
        }
    }
    out
}

/// Decode from compact byte format.
///
/// Returns `None` if the input has odd length (malformed).
pub fn decode_compact(data: &[u8]) -> Option<Vec<u8>> {
    if !data.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let count = data[i] as usize;
        let byte = data[i + 1];
        for _ in 0..count {
            out.push(byte);
        }
        i += 2;
    }
    Some(out)
}

/// Compute the compression ratio for basic RLE.
///
/// Returns `(encoded_size, original_size)` in bytes.
pub fn compression_ratio(data: &[u8]) -> (usize, usize) {
    let pairs = encode(data);
    (pairs.len() * 2, data.len())
}

/// Count the number of runs in the data.
pub fn run_count(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }
    let mut count = 1;
    for i in 1..data.len() {
        if data[i] != data[i - 1] {
            count += 1;
        }
    }
    count
}
