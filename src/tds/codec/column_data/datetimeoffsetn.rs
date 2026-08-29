use crate::{sql_read_bytes::SqlReadBytes, time::DateTimeOffset, ColumnData};

pub(crate) async fn decode<R>(src: &mut R, len: usize) -> crate::Result<ColumnData<'static>>
where
    R: SqlReadBytes + Unpin,
{
    let rlen = src.read_u8().await?;

    let dto = match rlen {
        0 => ColumnData::DateTimeOffset(None),
        _ => {
            // A datetimeoffset value is a `time` portion (rlen - 5 bytes) then a
            // 3-byte `date` and a 2-byte offset. A server rlen < 5 would underflow.
            let time_len = rlen.checked_sub(5).ok_or_else(|| {
                crate::Error::Protocol(
                    format!("datetimeoffset: invalid value length {rlen}").into(),
                )
            })?;
            let dto = DateTimeOffset::decode(src, len, time_len).await?;
            ColumnData::DateTimeOffset(Some(dto))
        }
    };

    Ok(dto)
}
