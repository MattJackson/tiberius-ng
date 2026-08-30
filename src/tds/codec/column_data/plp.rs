use crate::sql_read_bytes::SqlReadBytes;

// Decode a partially length-prefixed type.
//
// NOTE: values are read via the packet-aware `read_u8`/`read_u16_le`/`read_u32_le`
// helpers (which transparently span TDS packet boundaries). The generic
// `AsyncReadExt::read_exact` must NOT be used here: a PLP value can span multiple
// packets, and `read_exact` treats a packet-boundary `Ok(0)` as EOF.
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
                _ => {
                    let mut data = Vec::with_capacity(len.min(super::MAX_PREALLOC));

                    for _ in 0..len {
                        data.push(src.read_u8().await?);
                    }

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

            let mut chunk_data_left = 0usize;

            loop {
                if chunk_data_left == 0 {
                    // We have no chunk. Start a new one.
                    let chunk_size = src.read_u32_le().await? as usize;

                    if chunk_size == 0 {
                        break; // found a sentinel, we're done
                    } else {
                        chunk_data_left = chunk_size;
                    }
                } else {
                    // Read a byte (packet-aware).
                    let byte = src.read_u8().await?;
                    chunk_data_left -= 1;

                    data.push(byte);
                }
            }

            Ok(Some(data))
        }
    }
}
