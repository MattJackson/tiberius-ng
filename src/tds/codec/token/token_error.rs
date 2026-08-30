use crate::{tds::codec::FeatureLevel, SqlReadBytes};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
/// An error token returned from the server.
pub struct TokenError {
    /// ErrorCode
    pub(crate) code: u32,
    /// ErrorState (describing code)
    pub(crate) state: u8,
    /// The class (severity) of the error
    pub(crate) class: u8,
    /// The error message
    pub(crate) message: String,
    pub(crate) server: String,
    pub(crate) procedure: String,
    pub(crate) line: u32,
}

impl TokenError {
    pub(crate) async fn decode<R>(src: &mut R) -> crate::Result<Self>
    where
        R: SqlReadBytes + Unpin,
    {
        let _length = src.read_u16_le().await? as usize;

        let code = src.read_u32_le().await?;
        let state = src.read_u8().await?;
        let class = src.read_u8().await?;

        let message = src.read_us_varchar().await?;
        let server = src.read_b_varchar().await?;
        let procedure = src.read_b_varchar().await?;

        // MS-TDS 2.2.7.10: LineNumber is a 4-byte LONG for TDS 7.2 (SQL Server
        // 2005) and later, and a 2-byte USHORT before that. The boundary is
        // inclusive of 7.2, so use `>=` — a strict `>` mis-reads a 2-byte value
        // against a real SQL Server 2005 and desyncs the token stream.
        let line = if src.context().version() >= FeatureLevel::SqlServer2005 {
            src.read_u32_le().await?
        } else {
            src.read_u16_le().await? as u32
        };

        let token = TokenError {
            code,
            state,
            class,
            message,
            server,
            procedure,
            line,
        };

        Ok(token)
    }

    /// The error code, see descriptions from [the manual].
    ///
    /// [the manual]: https://docs.microsoft.com/en-us/sql/relational-databases/errors-events/database-engine-events-and-errors?view=sql-server-ver15
    pub fn code(&self) -> u32 {
        self.code
    }

    /// The error state, used as a modifier to the error number.
    pub fn state(&self) -> u8 {
        self.state
    }

    /// The class (severity) of the error. A class of less than 10 indicates an
    /// informational message.
    pub fn class(&self) -> u8 {
        self.class
    }

    /// The error message returned from the server.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The server name.
    pub fn server(&self) -> &str {
        &self.server
    }

    /// The name of the stored procedure causing the error.
    pub fn procedure(&self) -> &str {
        &self.procedure
    }

    /// The line number in the SQL batch or stored procedure that caused the
    /// error. Line numbers begin at 1. If the line number is not applicable to
    /// the message, the value is 0.
    pub fn line(&self) -> u32 {
        self.line
    }
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' on server {} executing {} on line {} (code: {}, state: {}, class: {})",
            self.message, self.server, self.procedure, self.line, self.code, self.state, self.class
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TokenError {
        TokenError {
            code: 1205,
            state: 2,
            class: 13,
            message: "deadlocked".to_string(),
            server: "myserver".to_string(),
            procedure: "myproc".to_string(),
            line: 42,
        }
    }

    #[test]
    fn accessors() {
        let e = sample();
        assert_eq!(e.code(), 1205);
        assert_eq!(e.state(), 2);
        assert_eq!(e.class(), 13);
        assert_eq!(e.message(), "deadlocked");
        assert_eq!(e.server(), "myserver");
        assert_eq!(e.procedure(), "myproc");
        assert_eq!(e.line(), 42);
    }

    #[test]
    fn display_contains_all_fields() {
        let rendered = format!("{}", sample());
        assert_eq!(
            rendered,
            "'deadlocked' on server myserver executing myproc on line 42 (code: 1205, state: 2, class: 13)"
        );
    }
}
