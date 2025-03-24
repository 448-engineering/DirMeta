#[cfg(feature = "time")]
use chrono::{DateTime, Utc};
#[cfg(feature = "time")]
use std::time::Duration;

#[cfg(feature = "time")]
use std::time::SystemTime;

#[cfg(feature = "time")]
use tai64::Tai64N;

/// Reusable Clone-on-Write str with lifetime of `'a`
pub type CowStr<'a> = std::borrow::Cow<'a, str>;

/// A convenience struct to access utilities
pub struct FsUtils;

impl FsUtils {
    /// Returns [Option::None] if time query is not supported
    #[cfg(feature = "time")]
    pub fn maybe_time(time_result: Option<SystemTime>) -> Option<Tai64N> {
        time_result.map(|time| Tai64N::from_system_time(&time))
    }

    /// Calculate the size in bytes
    #[cfg(feature = "size")]
    pub fn size_to_bytes(bytes: usize) -> String {
        byte_prefix::calc_bytes(bytes as f32)
    }

    /// Convert TAI64N to local time in 24 hour format
    #[cfg(feature = "time")]
    pub fn tai64_to_local_hrs<'a>(time: &Tai64N) -> DateTimeString<'a> {
        let date_time: DateTime<Utc> = time.to_system_time().into();
        let date = date_time
            .date_naive()
            .format("%A, %-d %B, %C%y")
            .to_string();
        let date = CowStr::Owned(date);
        let time = date_time.format("%H:%M:%S").to_string();
        let time = CowStr::Owned(time);

        DateTimeString { date, time }
    }

    /// Convert TAI64N to local time in 12 hour format
    #[cfg(feature = "time")]
    pub fn tai64_to_local_am_pm<'a>(time: &Tai64N) -> DateTimeString<'a> {
        let date_time: DateTime<Utc> = time.to_system_time().into();
        let date = date_time
            .date_naive()
            .format("%A, %-d %B, %C%y")
            .to_string();
        let date = CowStr::Owned(date);
        let time = date_time.format("%-I:%M %p").to_string();
        let time = CowStr::Owned(time);

        DateTimeString { date, time }
    }

    /// Convert duration since UNIX EPOCH to humantime
    #[cfg(feature = "time")]
    pub fn tai64_to_humantime_with_epoch(time: &Tai64N) -> Option<String> {
        FsUtils::tai64_duration_since_epoch(time)
            .map(|duration| humantime::format_duration(duration).to_string())
    }

    /// Convert duration since two TAI64N timestamps to humantime
    #[cfg(feature = "time")]
    pub fn tai64_to_humantime(earlier_time: &Tai64N, current_time: &Tai64N) -> Option<String> {
        FsUtils::tai64_duration(earlier_time, current_time)
            .map(|duration| humantime::format_duration(duration).to_string())
    }

    /// Convert duration between current time and earlier TAI64N timestamp to humantime
    #[cfg(feature = "time")]
    pub fn tai64_now_duration_to_humantime(earlier_time: &Tai64N) -> Option<String> {
        FsUtils::tai64_duration_from_now(earlier_time)
            .map(|duration| humantime::format_duration(duration).to_string())
    }

    /// Get the duration between two TAI64N timestamps
    #[cfg(feature = "time")]
    pub fn tai64_duration(earlier_time: &Tai64N, current_time: &Tai64N) -> Option<Duration> {
        match earlier_time.duration_since(current_time) {
            Ok(valid_time) => Some(valid_time),
            Err(_) => Option::None,
        }
    }

    /// Get the duration since UNIX EPOCH
    #[cfg(feature = "time")]
    pub fn tai64_duration_since_epoch(time: &Tai64N) -> Option<Duration> {
        match time.duration_since(&Tai64N::UNIX_EPOCH) {
            Ok(valid_time) => Some(valid_time),
            Err(_) => Option::None,
        }
    }

    /// Get the duration since UNIX EPOCH
    #[cfg(feature = "time")]
    pub fn tai64_duration_from_now(earlier_time: &Tai64N) -> Option<Duration> {
        match Tai64N::now().duration_since(earlier_time) {
            Ok(valid_time) => Some(valid_time),
            Err(_) => Option::None,
        }
    }
}

/// The data and time in human readable [String]
#[cfg(feature = "time")]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Default)]
pub struct DateTimeString<'a> {
    /// The data without a timestamp
    pub date: CowStr<'a>,
    /// A timestamp without a date
    pub time: CowStr<'a>,
}
