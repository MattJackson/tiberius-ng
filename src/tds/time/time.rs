//! Mappings between TDS and and time crate types (with `time` feature flag
//! enabled).
//!
//! The time library offers better ergonomy and are highly recommended if
//! needing to modify and deal with date and time in SQL Server.

use std::time::Duration;
pub use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use crate::tds::codec::ColumnData;

#[inline]
fn from_days(days: i64, start_year: i32) -> Date {
    // Use the signed `time::Duration` so that negative day offsets (dates
    // before `start_year`, e.g. `datetime` values prior to 1900) do not
    // overflow. Casting a negative day count into an unsigned type and
    // multiplying it out panics with "multiply with overflow".
    //
    // `days` ultimately comes from untrusted server bytes, so a malformed value
    // can land outside the range `time::Date` can represent. Use `checked_add`
    // and clamp to the type's bounds instead of panicking with "resulting value
    // is out of range".
    let base = Date::from_calendar_date(start_year, Month::January, 1).unwrap();
    base.checked_add(time::Duration::days(days))
        .unwrap_or(if days < 0 { Date::MIN } else { Date::MAX })
}

/// Convert a server-supplied fractional-seconds `increments` at the given
/// `scale` into nanoseconds without panicking. `scale` and `increments` are
/// untrusted; a `scale > 9` would otherwise underflow `9 - scale`, and a large
/// `increments` would overflow the multiply.
#[inline]
#[cfg(feature = "tds73")]
fn nanos_from_increments(increments: u64, scale: u8) -> u64 {
    let pow = 9u32.saturating_sub(scale as u32);
    increments.saturating_mul(10u64.saturating_pow(pow))
}

#[inline]
#[cfg(feature = "tds73")]
fn from_secs(secs: u64) -> Time {
    Time::from_hms(0, 0, 0).unwrap() + Duration::from_secs(secs)
}

#[inline]
fn from_sec_fragments(sec_fragments: u64) -> Time {
    Time::from_hms(0, 0, 0).unwrap() + Duration::from_nanos(sec_fragments * (1e9 as u64) / 300)
}

#[inline]
fn to_days(date: Date, start_year: i32) -> i64 {
    (date - Date::from_calendar_date(start_year, Month::January, 1).unwrap()).whole_days()
}

#[inline]
#[cfg(not(feature = "tds73"))]
fn to_sec_fragments(from: Time) -> i64 {
    let nanos: i64 = (from - Time::from_hms(0, 0, 0).unwrap())
        .whole_nanoseconds()
        .try_into()
        .unwrap();

    nanos * 300 / (1e9 as i64)
}

#[cfg(feature = "tds73")]
from_sql!(
    PrimitiveDateTime:
        ColumnData::SmallDateTime(ref dt) => dt.map(|dt| PrimitiveDateTime::new(
            from_days(dt.days as i64, 1900),
            from_secs(dt.seconds_fragments as u64 * 60),
        )),
        ColumnData::DateTime2(ref dt) => dt.map(|dt| PrimitiveDateTime::new(
            from_days(dt.date.days() as i64, 1),
            Time::from_hms(0,0,0).unwrap() + Duration::from_nanos(nanos_from_increments(dt.time.increments, dt.time.scale))
        )),
        ColumnData::DateTime(ref dt) => dt.map(|dt| PrimitiveDateTime::new(
            from_days(dt.days as i64, 1900),
            from_sec_fragments(dt.seconds_fragments as u64)
        ));
    Time:
        ColumnData::Time(ref time) => time.map(|time| {
            let ns = nanos_from_increments(time.increments, time.scale);
            Time::from_hms(0,0,0).unwrap() + Duration::from_nanos(ns)
        });
    Date:
        ColumnData::Date(ref date) => date.map(|date| from_days(date.days() as i64, 1));
    OffsetDateTime:
        ColumnData::DateTimeOffset(ref dto) => dto.map(|dto| {
            let date = from_days(dto.datetime2.date.days() as i64, 1);
            let dt = dto.datetime2;

            let time = Time::from_hms(0,0,0).unwrap()
                + Duration::from_nanos(nanos_from_increments(dt.time.increments, dt.time.scale));

            // A malformed server offset outside ±14h is not representable by
            // `UtcOffset`; fall back to UTC rather than panicking.
            let offset = UtcOffset::from_whole_seconds(dto.offset as i32 * 60)
                .unwrap_or(UtcOffset::UTC);

            date.with_time(time).assume_utc().to_offset(offset)
        })
);

#[cfg(feature = "tds73")]
to_sql!(self_,
        Date: (ColumnData::Date, super::Date::new(to_days(*self_, 1) as u32));
        Time: (ColumnData::Time, {
            let nanos: u64 = (*self_ - Time::from_hms(0, 0, 0).unwrap()).whole_nanoseconds().try_into().unwrap();
            let increments = nanos / 100;

            super::Time {increments, scale: 7}
        });
        PrimitiveDateTime: (ColumnData::DateTime2, {
            let time = self_.time();
            let nanos: u64 = (time - Time::from_hms(0, 0, 0).unwrap()).whole_nanoseconds().try_into().unwrap();
            let increments = nanos / 100;

            let date = super::Date::new(to_days(self_.date(), 1) as u32);
            let time = super::Time {increments, scale: 7};

            super::DateTime2::new(date, time)
        });
        OffsetDateTime: (ColumnData::DateTimeOffset, {
            let tz = self_.offset();
            let offset = (tz.whole_seconds() / 60) as i16;

            let utc_date = self_.to_offset(UtcOffset::UTC);

            let nanos: u64 = (utc_date.time() - Time::from_hms(0, 0, 0).unwrap()).whole_nanoseconds().try_into().unwrap();

            let date = super::Date::new(to_days(utc_date.date(), 1) as u32);
            let time = super::Time { increments: nanos / 100, scale: 7 };

            super::DateTimeOffset::new(super::DateTime2::new(date, time), offset)
        });
);

#[cfg(not(feature = "tds73"))]
to_sql!(self_,
        PrimitiveDateTime: (ColumnData::DateTime, {
            let date = self_.date();
            let time = self_.time();

            let days = to_days(date, 1900) as i32;
            let seconds_fragments = to_sec_fragments(time);

            super::DateTime::new(days, seconds_fragments as u32)
        });
);

#[cfg(not(feature = "tds73"))]
from_sql!(
    PrimitiveDateTime:
    ColumnData::DateTime(ref dt) => dt.map(|dt| {
        from_days(dt.days as i64, 1900).with_time(from_sec_fragments(dt.seconds_fragments as u64))
    })
);

#[cfg(test)]
mod tests {
    use super::*;

    // Regression test for #316: a `datetime` value with a date before 1900 has
    // a negative day offset from the 1900 base date. This must round-trip
    // without a "multiply with overflow" panic.
    #[test]
    fn from_days_handles_negative_offsets() {
        // 1899-12-31 is one day before the 1900 base date.
        assert_eq!(
            from_days(-1, 1900),
            Date::from_calendar_date(1899, Month::December, 31).unwrap()
        );

        // A date well before 1900, at the lower edge of the `datetime` range.
        let expected = Date::from_calendar_date(1850, Month::January, 1).unwrap();
        let days = to_days(expected, 1900);
        assert!(
            days < 0,
            "expected a negative day offset for pre-1900 dates"
        );

        // Rebuilding from the (negative) day offset must not overflow.
        assert_eq!(from_days(days, 1900), expected);
    }

    // Exercise the full decode path (`DateTime` -> `PrimitiveDateTime`) for a
    // pre-1900 value, matching what happens when reading a `datetime` column.
    #[test]
    fn datetime_before_1900_decodes() {
        let expected_date = Date::from_calendar_date(1850, Month::January, 1).unwrap();
        let days = to_days(expected_date, 1900) as i32;

        // Reconstruct the way the `from_sql!` mapping does for `ColumnData::DateTime`.
        let dt = crate::tds::time::DateTime::new(days, 0);
        let decoded = from_days(dt.days() as i64, 1900)
            .with_time(from_sec_fragments(dt.seconds_fragments() as u64));

        assert_eq!(decoded.date(), expected_date);
        assert_eq!(decoded.time(), Time::from_hms(0, 0, 0).unwrap());
    }
}
