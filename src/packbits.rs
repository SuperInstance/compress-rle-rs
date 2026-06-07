//! PackBits encoding as specified by Apple / TIFF.
//!
//! PackBits uses a header byte:
//! - `0..=127`: copy the next `n+1` literal bytes
//! - `129..=255`: repeat the next byte `257-n` times
//! - `128`: no operation (nop)

/// Encode data using PackBits.
///
/// # Format
///
/// Each run starts with a header byte:
/// - Header `0..127`: Copy the next `header + 1` literal bytes.
/// - Header `129..255`: Repeat the next byte `257 - header` times.
/// - Header `128`: No-op (skipped).
///
/// # Examples
///
/// ```
/// use compress_rle_rs::packbits;
///
/// let encoded = packbits::encode(b"AAABBC");
/// let decoded = packbits::decode(&encoded).unwrap();
/// assert_eq!(b"AAABBC".as_slice(), decoded.as_slice());
/// ```
pub fn encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Count run of identical bytes
        let byte = data[i];
        let mut run_len = 1;
        while i + run_len < data.len() && data[i + run_len] == byte && run_len < 128 {
            run_len += 1;
        }

        if run_len > 2 {
            // Emit a run: header = 257 - run_len
            // For run_len = 3: header = 254 (0xFE)
            // For run_len = 128: header = 129 (0x81)
            out.push((257 - run_len) as u8);
            out.push(byte);
            i += run_len;
        } else {
            // Collect literal bytes
            let literal_start = i;
            let mut literal_end = i;

            while literal_end < data.len() {
                // Check if a run of 3+ starts here
                let b = data[literal_end];
                let mut ahead = 1;
                while literal_end + ahead < data.len() && data[literal_end + ahead] == b && ahead < 3 {
                    ahead += 1;
                }
                if ahead >= 3 {
                    break;
                }
                literal_end += 1;
                if literal_end - literal_start >= 128 {
                    break;
                }
            }

            let literal_len = literal_end - literal_start;
            if literal_len > 0 {
                out.push((literal_len - 1) as u8);
                out.extend_from_slice(&data[literal_start..literal_end]);
                i = literal_end;
            }
        }
    }

    out
}

/// Decode PackBits-encoded data.
///
/// Returns `None` if the input is malformed (truncated).
pub fn decode(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let header = data[i];
        i += 1;

        if header <= 127 {
            // Copy next header+1 literal bytes
            let count = (header as usize) + 1;
            if i + count > data.len() {
                return None;
            }
            out.extend_from_slice(&data[i..i + count]);
            i += count;
        } else if header > 128 {
            // Repeat next byte (257 - header) times
            let count = 257 - header as usize;
            if i >= data.len() {
                return None;
            }
            let byte = data[i];
            for _ in 0..count {
                out.push(byte);
            }
            i += 1;
        }
        // header == 128 (0x80) is nop
    }

    Some(out)
}

/// Compute the encoded size without actually encoding.
pub fn encoded_size(data: &[u8]) -> usize {
    encode(data).len()
}
