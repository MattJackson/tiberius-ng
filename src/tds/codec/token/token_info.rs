use crate::SqlReadBytes;

#[allow(dead_code)] // we might want to debug the values
#[derive(Debug)]
pub struct TokenInfo {
    /// info number
    pub(crate) number: u32,
    /// error state
    pub(crate) state: u8,
    /// severity (<10: Info)
    pub(crate) class: u8,
    pub(crate) message: String,
    pub(crate) server: String,
    pub(crate) procedure: String,
    pub(crate) line: u32,
}

impl TokenInfo {
    pub(crate) async fn decode<R>(src: &mut R) -> crate::Result<Self>
    where
        R: SqlReadBytes + Unpin,
    {
        let _length = src.read_u16_le().await?;

        let number = src.read_u32_le().await?;
        let state = src.read_u8().await?;
        let class = src.read_u8().await?;
        let message = src.read_us_varchar().await?;
        let server = src.read_b_varchar().await?;
        let procedure = src.read_b_varchar().await?;
        let line = src.read_u32_le().await?;

        Ok(TokenInfo {
            number,
            state,
            class,
            message,
            server,
            procedure,
            line,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql_read_bytes::test_utils::IntoSqlReadBytes;
    use bytes::{BufMut, BytesMut};

    fn put_b_varchar(buf: &mut BytesMut, s: &str) {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        buf.put_u8(utf16.len() as u8);
        for c in utf16 {
            buf.put_u16_le(c);
        }
    }

    fn put_us_varchar(buf: &mut BytesMut, s: &str) {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        buf.put_u16_le(utf16.len() as u16);
        for c in utf16 {
            buf.put_u16_le(c);
        }
    }

    #[tokio::test]
    async fn decodes_all_fields() {
        let mut buf = BytesMut::new();
        buf.put_u16_le(0); // length, ignored
        buf.put_u32_le(4711);
        buf.put_u8(2);
        buf.put_u8(9);
        put_us_varchar(&mut buf, "informational");
        put_b_varchar(&mut buf, "server");
        put_b_varchar(&mut buf, "proc");
        buf.put_u32_le(123);

        let info = TokenInfo::decode(&mut buf.into_sql_read_bytes())
            .await
            .unwrap();

        assert_eq!(info.number, 4711);
        assert_eq!(info.state, 2);
        assert_eq!(info.class, 9);
        assert_eq!(info.message, "informational");
        assert_eq!(info.server, "server");
        assert_eq!(info.procedure, "proc");
        assert_eq!(info.line, 123);
    }
}
