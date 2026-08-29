use crate::{sql_read_bytes::SqlReadBytes, time::DateTime2, ColumnData};

pub(crate) async fn decode<R>(src: &mut R, len: usize) -> crate::Result<ColumnData<'static>>
where
    R: SqlReadBytes + Unpin,
{
    let rlen = src.read_u8().await?;

    let date = match rlen {
        0 => ColumnData::DateTime2(None),
        rlen => {
            // A datetime2 value is a `time` portion (rlen - 3 bytes) followed by
            // a 3-byte `date`. A server-supplied rlen < 3 would underflow.
            let time_len = (rlen as usize).checked_sub(3).ok_or_else(|| {
                crate::Error::Protocol(format!("datetime2: invalid value length {rlen}").into())
            })?;
            let dt = DateTime2::decode(src, len, time_len).await?;
            ColumnData::DateTime2(Some(dt))
        }
    };

    Ok(date)
}
