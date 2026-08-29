use crate::sql_read_bytes::SqlReadBytes;
use futures_util::io::AsyncReadExt;

// Decode a partially length-prefixed type.
pub(crate) async fn decode<R>(src: &mut R, len: usize) -> crate::Result<Option<Vec<u8>>>
where
    R: SqlReadBytes + Unpin,
{
    match len {
        // Fixed size
        len if len < 0xffff => {
            let len = src.read_u16_le().await? as usize;

            match len {
                // NULL
                0xffff => Ok(None),
                // `len` is a u16 here, so it is inherently bounded (< 64 KiB);
                // read it in one shot instead of byte-by-byte.
                _ => {
                    let mut data = vec![0u8; len];
                    src.read_exact(&mut data).await?;

                    Ok(Some(data))
                }
            }
        }
        // Unknown size, length-prefixed blobs
        _ => {
            let len = src.read_u64_le().await?;

            let mut data = match len {
                // NULL
                0xffffffffffffffff => return Ok(None),
                // Unknown size
                0xfffffffffffffffe => Vec::new(),
                // Known size. `len` is an untrusted 64-bit wire value; cap the
                // up-front reservation (avoids memory-exhaustion and the
                // `Vec` capacity-overflow panic for values near u64::MAX).
                _ => Vec::with_capacity((len as usize).min(super::MAX_PREALLOC)),
            };

            // Reusable stack buffer so each chunk is read in bounded bulk copies
            // rather than one async `read_u8()` future per byte, without ever
            // pre-allocating a whole (untrusted) chunk length up front.
            let mut scratch = [0u8; super::MAX_PREALLOC];

            loop {
                let chunk_size = src.read_u32_le().await? as usize;

                if chunk_size == 0 {
                    break; // found a sentinel, we're done
                }

                let mut remaining = chunk_size;
                while remaining > 0 {
                    let take = remaining.min(scratch.len());
                    src.read_exact(&mut scratch[..take]).await?;
                    data.extend_from_slice(&scratch[..take]);
                    remaining -= take;
                }
            }

            Ok(Some(data))
        }
    }
}
