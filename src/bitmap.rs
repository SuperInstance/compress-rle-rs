//! Modified RLE for 1-bit bitmap rows.
//!
//! Each scanline is encoded as a sequence of alternating runs of 0s and 1s,
//! starting with a run of 0s. If the scanline starts with 1s, a leading
//! 0-length run of 0s is emitted. Runs are stored as single bytes (max 255).

/// Encode a single scanline using modified bitmap RLE.
///
/// The output alternates between counts of 0-bits and 1-bits, starting with
/// 0-bits. Each count is stored as a `u8` (1–255).
///
/// # Examples
///
/// ```
/// use compress_rle_rs::bitmap;
///
/// let scanline: Vec<u8> = vec![0,0,0,0,0,1,1,1,0,0];
/// let encoded = bitmap::encode_scanline(&scanline);
/// assert_eq!(encoded, vec![5, 3, 2]);
/// ```
pub fn encode_scanline(scanline: &[u8]) -> Vec<u8> {
    if scanline.is_empty() {
        return Vec::new();
    }

    let mut runs = Vec::new();

    // Determine what value the scanline starts with
    // We always start with 0-runs. If first pixel is 1, emit a 0-length run.
    let first_val = scanline[0];
    let mut current_val: u8 = 0; // Always start counting 0s
    let mut count: usize = 0;

    if first_val == 1 {
        // Emit a 0-length run of 0s to start
        runs.push(0u8);
        current_val = 1;
    }

    for &pixel in scanline {
        if pixel == current_val {
            count += 1;
        } else {
            // Flush current run (split at 255)
            flush_run(&mut runs, count);
            current_val = pixel;
            count = 1;
        }
    }

    // Flush last run
    flush_run(&mut runs, count);

    runs
}

/// Flush a run, splitting at 255.
fn flush_run(runs: &mut Vec<u8>, mut count: usize) {
    while count > 255 {
        runs.push(255);
        runs.push(0); // 0-length run of the other value
        count -= 255;
    }
    if count > 0 {
        runs.push(count as u8);
    }
}

/// Decode a modified bitmap RLE scanline.
///
/// `width` is the expected scanline width in pixels.
/// Returns `None` if the encoded data doesn't match the expected width.
pub fn decode_scanline(runs: &[u8], width: usize) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(width);
    let mut current_val = 0u8;

    for &count in runs {
        for _ in 0..count {
            out.push(current_val);
        }
        current_val = if current_val == 0 { 1 } else { 0 };
    }

    if out.len() != width {
        return None;
    }

    Some(out)
}

/// Encode a full bitmap (multiple scanlines).
///
/// Each scanline is encoded independently. The output format:
/// `[scanline_count_u16, [scanline_len_u16, scanline_bytes], ...]`
pub fn encode_bitmap(bitmap: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    let count = bitmap.len() as u16;
    out.push((count >> 8) as u8);
    out.push((count & 0xFF) as u8);

    for scanline in bitmap {
        let encoded = encode_scanline(scanline);
        let len = encoded.len() as u16;
        out.push((len >> 8) as u8);
        out.push((len & 0xFF) as u8);
        out.extend(encoded);
    }

    out
}

/// Decode a full bitmap.
///
/// Returns `None` if the data is malformed.
pub fn decode_bitmap(data: &[u8]) -> Option<Vec<Vec<u8>>> {
    if data.len() < 2 {
        return None;
    }

    let count = ((data[0] as u16) << 8) | (data[1] as u16);
    let mut bitmap = Vec::with_capacity(count as usize);
    let mut pos = 2;

    for _ in 0..count {
        if pos + 2 > data.len() {
            return None;
        }
        let len = ((data[pos] as u16) << 8) | (data[pos + 1] as u16);
        pos += 2;
        if pos + len as usize > data.len() {
            return None;
        }
        let encoded = &data[pos..pos + len as usize];
        pos += len as usize;

        // Decode
        let mut scanline = Vec::new();
        let mut current_val = 0u8;
        for &run in encoded {
            for _ in 0..run {
                scanline.push(current_val);
            }
            current_val = if current_val == 0 { 1 } else { 0 };
        }
        bitmap.push(scanline);
    }

    Some(bitmap)
}
