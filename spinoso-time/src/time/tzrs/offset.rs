use core::fmt;
use std::error;

use regex::Regex;
use tz::timezone::{LocalTimeType, TimeZoneRef};
use tzdb::local_tz;
use tzdb::time_zone::etc::GMT;

#[derive(Debug, Clone)]
pub struct TzStringError(String);

impl fmt::Display for TzStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "invalid timezone {}", self.0)
    }
}

impl error::Error for TzStringError {}

const SECONDS_IN_MINUTE: i32 = 60;
const SECONDS_IN_HOUR: i32 = SECONDS_IN_MINUTE * 60;

/// tzdb provides [`local_tz`] to get the local system timezone. If this ever fails, we can
/// assume `GMT`. `GMT` is used instead of `UTC` since it has a [`time_zone_designation`] - which
/// if it is an empty string, then it is considered to be a UTC time.
///
/// Note: this matches MRI Ruby implmentation. Where `TZ="" ruby -e "puts Time::now"` will return a
/// new _time_ with 0 offset from UTC, but still still report as a non utc time.
///
/// [`local_tz`]: https://docs.rs/tzdb/latest/tzdb/fn.local_tz.html
/// [`time_zone_designation`]: https://docs.rs/tz-rs/0.6.9/tz/timezone/struct.LocalTimeType.html#method.time_zone_designation
#[inline]
#[must_use]
fn local_time_zone() -> TimeZoneRef<'static> {
    match local_tz() {
        Some(tz) => tz,
        None => GMT,
    }
}

/// Generates a [+/-]HHMM timezone format from a given number of seconds
/// Note: the actual seconds element is effectively ignored here
#[inline]
#[must_use]
fn offset_hhmm_from_seconds(seconds: i32) -> String {
    let flag = if seconds < 0 { '-' } else { '+' };
    let minutes = seconds.abs() / 60;

    let offset_hours = minutes / 60;
    let offset_minutes = minutes - (offset_hours * 60);

    format!("{}{:0>2}{:0>2}", flag, offset_hours, offset_minutes)
}

/// Represents the number of seconds offset from UTC
#[allow(variant_size_differences)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Offset {
    /// UTC offset, zero offset, Zulu time
    Utc,
    /// Fixed offset from UTC
    ///
    /// Note: A fixed offset of 0 is different from UTC time
    Fixed([LocalTimeType; 1]),
    /// A time zone based offset
    Tz(TimeZoneRef<'static>),
}

impl<'a> Offset {
    /// Generate a UTC based offset
    #[inline]
    #[must_use]
    pub fn utc() -> Self {
        Self::Utc
    }

    /// Generate an offset based on the detected local time zone of the system
    ///
    /// Detection is done by [`tzdb::local_tz`], and if it fails will return a GMT timezone
    ///
    /// [`tzdb::local_tz`]: https://docs.rs/tzdb/latest/tzdb/fn.local_tz.html
    #[inline]
    #[must_use]
    pub fn local() -> Self {
        Self::Tz(local_time_zone())
    }

    /// Generate an offset with a number of seconds from UTC.
    #[inline]
    #[must_use]
    pub fn fixed(offset: i32) -> Self {
        let offset_name = offset_hhmm_from_seconds(offset);
        let local_time_type =
            LocalTimeType::new(offset, false, Some(offset_name.as_bytes())).expect("Couldn't create fixed offset");
        Self::Fixed([local_time_type])
    }

    /// Generate an offset based on a provided [`tz::timezone::TimeZoneRef`]
    ///
    /// This can be combined with [`tzdb`] to generate offsets based on predefined iana time zones
    ///
    /// ```
    /// use spinoso_time::tzrs::Offset;
    /// use tzdb::time_zone::pacific::AUCKLAND;
    /// let offset = Offset::tz(AUCKLAND);
    /// ```
    ///
    /// [`tz:timezone::TimeZoneRef`]: https://docs.rs/tz-rs/0.6.9/tz/timezone/struct.TimeZoneRef.html
    /// [`tzdb`]: https://docs.rs/tzdb/latest/tzdb/index.html
    #[inline]
    #[must_use]
    pub fn tz(tz: TimeZoneRef<'static>) -> Self {
        Self::Tz(tz)
    }

    /// Returns a `TimeZoneRef` which can be used to generate and project _time_.
    #[inline]
    #[must_use]
    pub fn time_zone_ref(&'a self) -> TimeZoneRef<'a> {
        match self {
            Self::Utc => TimeZoneRef::utc(),
            Self::Fixed(local_time_types) => match TimeZoneRef::new(&[], local_time_types, &[], &None) {
                Ok(tz) => tz,
                Err(_) => GMT,
            },

            Self::Tz(zone) => *zone,
        }
    }
}

impl TryFrom<&str> for Offset {
    type Error = TzStringError;

    /// Construct a Offset based on the [accepted MRI values]
    ///
    /// Accepts:
    ///
    /// - `[+/-]HH[:]MM`
    /// - A-I representing +01:00 to +09:00
    /// - K-M representing +10:00 to +12:00
    /// - N-Y representing -01:00 to -12:00
    /// - Z representing 0 offset
    ///
    /// [accepted MRI values]: https://ruby-doc.org/core-2.6.3/Time.html#method-c-new
    #[inline]
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        match input {
            "A" => Ok(Self::fixed(1)),
            "B" => Ok(Self::fixed(2)),
            "C" => Ok(Self::fixed(3)),
            "D" => Ok(Self::fixed(4)),
            "E" => Ok(Self::fixed(5)),
            "F" => Ok(Self::fixed(6)),
            "G" => Ok(Self::fixed(7)),
            "H" => Ok(Self::fixed(8)),
            "I" => Ok(Self::fixed(9)),
            "K" => Ok(Self::fixed(10)),
            "L" => Ok(Self::fixed(11)),
            "M" => Ok(Self::fixed(12)),
            "N" => Ok(Self::fixed(-1)),
            "O" => Ok(Self::fixed(-2)),
            "P" => Ok(Self::fixed(-3)),
            "Q" => Ok(Self::fixed(-4)),
            "R" => Ok(Self::fixed(-5)),
            "S" => Ok(Self::fixed(-6)),
            "T" => Ok(Self::fixed(-7)),
            "U" => Ok(Self::fixed(-8)),
            "V" => Ok(Self::fixed(-9)),
            "W" => Ok(Self::fixed(-10)),
            "X" => Ok(Self::fixed(-11)),
            "Y" => Ok(Self::fixed(-12)),
            "Z" | "UTC" => Ok(Self::utc()),
            _ => {
                lazy_static! {
                    static ref HH_MM_MATCHER: Regex = Regex::new(r"^([\-\+]{1})(\d{2})(\d{2})$").unwrap();
                }
                if HH_MM_MATCHER.is_match(input) {
                    let caps = HH_MM_MATCHER.captures(input).unwrap();

                    let sign = if caps.get(1).unwrap().as_str() == "+" { 1 } else { -1 };
                    let hours = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
                    let minutes = caps.get(3).unwrap().as_str().parse::<i32>().unwrap();

                    let offset_seconds: i32 = sign * ((hours * SECONDS_IN_HOUR) + (minutes * SECONDS_IN_MINUTE));
                    Ok(Self::fixed(offset_seconds))
                } else {
                    Err(TzStringError(input.to_string()))
                }
            }
        }
    }
}

impl TryFrom<String> for Offset {
    type Error = TzStringError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        Offset::try_from(input.as_str())
    }
}

impl From<TimeZoneRef<'static>> for Offset {
    #[inline]
    #[must_use]
    fn from(tz: TimeZoneRef<'static>) -> Self {
        Self::tz(tz)
    }
}

impl From<i32> for Offset {
    /// Construct a Offset with the offset in seconds from UTC
    #[inline]
    #[must_use]
    fn from(seconds: i32) -> Self {
        Self::fixed(seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offset_seconds_from_fixed_offset(input: &str) -> i32 {
        let offset = Offset::try_from(input).unwrap();
        let local_time_type = offset.time_zone_ref().local_time_types()[0];
        local_time_type.ut_offset()
    }

    fn offset_name(offset: &Offset) -> &str {
        match offset {
            Offset::Utc => "UTC",
            Offset::Fixed(ltt) => ltt[0].time_zone_designation(),
            Offset::Tz(_) => "Ambiguous timezone name",
        }
    }

    #[test]
    fn fixed_zero_is_not_utc() {
        let offset = Offset::from(0);
        assert!(matches!(offset, Offset::Fixed(_)));
    }

    #[test]
    fn z_is_utc() {
        let offset = Offset::try_from("Z").unwrap();
        assert!(matches!(offset, Offset::Utc));
    }

    #[test]
    fn from_str_hh_mm() {
        assert_eq!(0, offset_seconds_from_fixed_offset("+0000"));
        assert_eq!(0, offset_seconds_from_fixed_offset("-0000"));
        assert_eq!(60, offset_seconds_from_fixed_offset("+0001"));
        assert_eq!(-60, offset_seconds_from_fixed_offset("-0001"));
        assert_eq!(3600, offset_seconds_from_fixed_offset("+0100"));
        assert_eq!(-3600, offset_seconds_from_fixed_offset("-0100"));
        assert_eq!(7320, offset_seconds_from_fixed_offset("+0202"));
        assert_eq!(-7320, offset_seconds_from_fixed_offset("-0202"));
        assert_eq!(362_340, offset_seconds_from_fixed_offset("+9999"));
        assert_eq!(-362_340, offset_seconds_from_fixed_offset("-9999"));
        assert_eq!(3660, offset_seconds_from_fixed_offset("+0061"));
    }

    #[test]
    fn from_str_hh_mm_strange() {
        assert_eq!(3660, offset_seconds_from_fixed_offset("+0061"));
    }

    #[test]
    fn fixed_time_zone_designation() {
        assert_eq!("+0000", offset_name(&Offset::from(0)));
        assert_eq!("+0000", offset_name(&Offset::from(59)));
        assert_eq!("+0001", offset_name(&Offset::from(60)));
        assert_eq!("-0001", offset_name(&Offset::from(-60)));
        assert_eq!("+0100", offset_name(&Offset::from(3600)));
        assert_eq!("-0100", offset_name(&Offset::from(-3600)));
        assert_eq!("+0202", offset_name(&Offset::from(7320)));
        assert_eq!("-0202", offset_name(&Offset::from(-7320)));
        assert_eq!("+9959", offset_name(&Offset::from(359_940)));
        assert_eq!("-9959", offset_name(&Offset::from(-359_940)));

        // Unexpected cases
        assert_eq!("-0000", offset_name(&Offset::from(-59)));

        // FIXME: Should error instead
        assert_eq!("+10000", offset_name(&Offset::from(360_000)));
    }
}
