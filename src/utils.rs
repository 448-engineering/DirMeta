use chrono::{DateTime, Utc};
use std::{
    borrow::Cow,
    time::{Duration, SystemTime},
};
use tai64::Tai64N;

pub type CowStr<'a> = Cow<'a, str>;

pub struct FsUtils;

impl FsUtils {
    /// Returns [Option::None] if time query is not supported
    pub fn maybe_time(time_result: Option<SystemTime>) -> Option<Tai64N> {
        if let Some(time) = time_result {
            Some(Tai64N::from_system_time(&time))
        } else {
            Option::None
        }
    }

    /// Calculate the size in bytes
    pub fn size_to_bytes(bytes: usize) -> String {
        byte_prefix::calc_bytes(bytes as f32)
    }

    /// Convert TAI64N to local time in 24 hour format
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
    pub fn tai64_to_humantime_with_epoch(time: &Tai64N) -> Option<String> {
        if let Some(duration) = FsUtils::tai64_duration_since_epoch(time) {
            Some(humantime::format_duration(duration).to_string())
        } else {
            None
        }
    }

    /// Convert duration since two TAI64N timestamps to humantime
    pub fn tai64_to_humantime(earlier_time: &Tai64N, current_time: &Tai64N) -> Option<String> {
        if let Some(duration) = FsUtils::tai64_duration(earlier_time, current_time) {
            Some(humantime::format_duration(duration).to_string())
        } else {
            None
        }
    }

    /// Convert duration between current time and earlier TAI64N timestamp to humantime
    pub fn tai64_now_duration_to_humantime(earlier_time: &Tai64N) -> Option<String> {
        if let Some(duration) = FsUtils::tai64_duration_from_now(earlier_time) {
            Some(humantime::format_duration(duration).to_string())
        } else {
            None
        }
    }

    /// Get the duration between two TAI64N timestamps
    pub fn tai64_duration(earlier_time: &Tai64N, current_time: &Tai64N) -> Option<Duration> {
        match earlier_time.duration_since(&Tai64N::UNIX_EPOCH) {
            Ok(valid_time) => Some(valid_time),
            Err(_) => return Option::None,
        }
    }

    /// Get the duration since UNIX EPOCH
    pub fn tai64_duration_since_epoch(time: &Tai64N) -> Option<Duration> {
        match time.duration_since(&Tai64N::UNIX_EPOCH) {
            Ok(valid_time) => Some(valid_time),
            Err(_) => return Option::None,
        }
    }

    /// Get the duration since UNIX EPOCH
    pub fn tai64_duration_from_now(earlier_time: &Tai64N) -> Option<Duration> {
        match Tai64N::now().duration_since(&earlier_time) {
            Ok(valid_time) => Some(valid_time),
            Err(_) => return Option::None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Default)]
pub struct DateTimeString<'a> {
    date: CowStr<'a>,
    time: CowStr<'a>,
}
