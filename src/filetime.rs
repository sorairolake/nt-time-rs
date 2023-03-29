//
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Copyright (C) 2023 Shun Sakai
//

//! The [Windows NT system time][file-time-docs-url].
//!
//! [file-time-docs-url]: https://docs.microsoft.com/en-us/windows/win32/sysinfo/file-times

use core::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use time::{macros::datetime, OffsetDateTime};

use crate::error;

#[cfg(feature = "std")]
static SYSTEM_TIME_NT_EPOCH: once_cell::sync::Lazy<std::time::SystemTime> =
    once_cell::sync::Lazy::new(|| {
        std::time::SystemTime::UNIX_EPOCH - std::time::Duration::from_secs(11_644_473_600)
    });

/// `FileTime` is a type that represents the [Windows NT system
/// time][file-time-docs-url].
///
/// This is a 64-bit unsigned integer value that represents the number of
/// 100-nanosecond intervals that have elapsed since "1601-01-01 00:00:00 UTC".
///
/// [file-time-docs-url]: https://docs.microsoft.com/en-us/windows/win32/sysinfo/file-times
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileTime(u64);

impl FileTime {
    /// The NT time epoch.
    ///
    /// This is defined as "1601-01-01 00:00:00 UTC".
    ///
    /// # Examples
    ///
    /// ```
    /// # use nt_time::FileTime;
    /// # use time::{macros::datetime, OffsetDateTime};
    /// #
    /// assert_eq!(
    ///     OffsetDateTime::try_from(FileTime::NT_EPOCH).unwrap(),
    ///     datetime!(1601-01-01 00:00 UTC)
    /// );
    /// ```
    pub const NT_EPOCH: Self = Self::new(u64::MIN);

    /// The largest value that can be represented by the Windows NT system time.
    ///
    /// This is "+60056-05-28 05:36:10.955161500 UTC".
    ///
    /// # Examples
    ///
    /// ```
    /// # use nt_time::FileTime;
    /// # use time::{macros::datetime, OffsetDateTime};
    /// #
    /// # #[cfg(feature = "large-dates")]
    /// assert_eq!(
    ///     OffsetDateTime::try_from(FileTime::MAX).unwrap(),
    ///     datetime!(+60056-05-28 05:36:10.955_161_500 UTC)
    /// );
    /// ```
    pub const MAX: Self = Self::new(u64::MAX);

    /// Creates a new `FileTime` with the given Windows NT system time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(FileTime::new(u64::MIN), FileTime::NT_EPOCH);
    /// assert_eq!(FileTime::new(u64::MAX), FileTime::MAX);
    /// ```
    #[must_use]
    #[inline]
    pub const fn new(time: u64) -> Self {
        Self(time)
    }

    /// Converts a `FileTime` to the Windows NT system time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(FileTime::NT_EPOCH.as_u64(), u64::MIN);
    /// assert_eq!(FileTime::MAX.as_u64(), u64::MAX);
    /// ```
    #[must_use]
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Computes `self + duration`, returning [`None`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::time::Duration;
    /// #
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(
    ///     FileTime::NT_EPOCH.checked_add(Duration::from_nanos(1)),
    ///     Some(FileTime::NT_EPOCH)
    /// );
    /// assert_eq!(
    ///     FileTime::NT_EPOCH.checked_add(Duration::from_nanos(100)),
    ///     Some(FileTime::new(1))
    /// );
    ///
    /// assert_eq!(FileTime::MAX.checked_add(Duration::from_nanos(100)), None);
    /// ```
    #[must_use]
    pub fn checked_add(self, duration: core::time::Duration) -> Option<Self> {
        let duration = u64::try_from(duration.as_nanos() / 100).ok()?;
        let ft = self.as_u64().checked_add(duration)?;
        Some(Self::new(ft))
    }

    /// Computes `self - duration`, returning [`None`] if the result would be
    /// negative or if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::time::Duration;
    /// #
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(
    ///     FileTime::MAX.checked_sub(Duration::from_nanos(1)),
    ///     Some(FileTime::MAX)
    /// );
    /// assert_eq!(
    ///     FileTime::MAX.checked_sub(Duration::from_nanos(100)),
    ///     Some(FileTime::new(u64::MAX - 1))
    /// );
    ///
    /// assert_eq!(
    ///     FileTime::NT_EPOCH.checked_sub(Duration::from_nanos(100)),
    ///     None
    /// );
    /// ```
    #[must_use]
    pub fn checked_sub(self, duration: core::time::Duration) -> Option<Self> {
        let duration = u64::try_from(duration.as_nanos() / 100).ok()?;
        let ft = self.as_u64().checked_sub(duration)?;
        Some(Self::new(ft))
    }

    /// Computes `self + duration`, returning [`FileTime::MAX`] if overflow
    /// occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::time::Duration;
    /// #
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(
    ///     FileTime::NT_EPOCH.saturating_add(Duration::from_nanos(1)),
    ///     FileTime::NT_EPOCH
    /// );
    /// assert_eq!(
    ///     FileTime::NT_EPOCH.saturating_add(Duration::from_nanos(100)),
    ///     FileTime::new(1)
    /// );
    ///
    /// assert_eq!(
    ///     FileTime::MAX.saturating_add(Duration::from_nanos(100)),
    ///     FileTime::MAX
    /// );
    /// ```
    #[must_use]
    pub fn saturating_add(self, duration: core::time::Duration) -> Self {
        let duration = u64::try_from(duration.as_nanos() / 100).unwrap_or(u64::MAX);
        let ft = self.as_u64().saturating_add(duration);
        Self::new(ft)
    }

    /// Computes `self - duration`, returning [`FileTime::NT_EPOCH`] if the
    /// result would be negative or if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::time::Duration;
    /// #
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(
    ///     FileTime::MAX.saturating_sub(Duration::from_nanos(1)),
    ///     FileTime::MAX
    /// );
    /// assert_eq!(
    ///     FileTime::MAX.saturating_sub(Duration::from_nanos(100)),
    ///     FileTime::new(u64::MAX - 1)
    /// );
    ///
    /// assert_eq!(
    ///     FileTime::NT_EPOCH.saturating_sub(Duration::from_nanos(100)),
    ///     FileTime::NT_EPOCH
    /// );
    /// ```
    #[must_use]
    pub fn saturating_sub(self, duration: core::time::Duration) -> Self {
        let duration = u64::try_from(duration.as_nanos() / 100).unwrap_or(u64::MAX);
        let ft = self.as_u64().saturating_sub(duration);
        Self::new(ft)
    }
}

impl Default for FileTime {
    /// Returns the default value of "1601-01-01 00:00:00 UTC".
    ///
    /// # Examples
    ///
    /// ```
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(FileTime::default(), FileTime::NT_EPOCH);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::NT_EPOCH
    }
}

impl fmt::Display for FileTime {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        u64::from(*self).fmt(f)
    }
}

impl Add<core::time::Duration> for FileTime {
    type Output = Self;

    fn add(self, rhs: core::time::Duration) -> Self::Output {
        let ft = self.as_u64()
            + u64::try_from(rhs.as_nanos() / 100)
                .expect("duration should be in the range of `u64`");
        Self::new(ft)
    }
}

impl AddAssign<core::time::Duration> for FileTime {
    fn add_assign(&mut self, rhs: core::time::Duration) {
        let ft = self.as_u64()
            + u64::try_from(rhs.as_nanos() / 100)
                .expect("duration should be in the range of `u64`");
        *self = Self::new(ft);
    }
}

impl Sub for FileTime {
    type Output = core::time::Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let duration = (self.as_u64() - rhs.as_u64()) * 100;
        Self::Output::from_nanos(duration)
    }
}

impl Sub<core::time::Duration> for FileTime {
    type Output = Self;

    fn sub(self, rhs: core::time::Duration) -> Self::Output {
        let ft = self.as_u64()
            - u64::try_from(rhs.as_nanos() / 100)
                .expect("duration should be in the range of `u64`");
        Self::new(ft)
    }
}

impl SubAssign<core::time::Duration> for FileTime {
    fn sub_assign(&mut self, rhs: core::time::Duration) {
        let ft = self.as_u64()
            - u64::try_from(rhs.as_nanos() / 100)
                .expect("duration should be in the range of `u64`");
        *self = Self::new(ft);
    }
}

impl From<FileTime> for u64 {
    /// Converts a `FileTime` to the Windows NT system time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(u64::from(FileTime::NT_EPOCH), u64::MIN);
    /// assert_eq!(u64::from(FileTime::MAX), u64::MAX);
    /// ```
    #[inline]
    fn from(time: FileTime) -> Self {
        time.as_u64()
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl From<FileTime> for std::time::SystemTime {
    /// Converts a `FileTime` to a [`SystemTime`](std::time::SystemTime).
    fn from(time: FileTime) -> Self {
        use std::time::Duration;

        let duration = Duration::new(
            time.as_u64() / 10_000_000,
            u32::try_from((time.as_u64() % 10_000_000) * 100)
                .expect("the number of nanoseconds should be in the range of `u32`"),
        );
        *SYSTEM_TIME_NT_EPOCH + duration
    }
}

impl TryFrom<FileTime> for OffsetDateTime {
    type Error = error::OffsetDateTimeRangeError;

    /// Converts a `FileTime` to a [`OffsetDateTime`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the `large-dates` feature is disabled and `time` is
    /// out of range of [`OffsetDateTime`].
    fn try_from(time: FileTime) -> Result<Self, Self::Error> {
        const NT_EPOCH: OffsetDateTime = datetime!(1601-01-01 00:00 UTC);
        let duration = time::Duration::new(
            i64::try_from(time.0 / 10_000_000)
                .expect("the number of seconds should be in the range of `i64`"),
            i32::try_from((time.0 % 10_000_000) * 100)
                .expect("the number of nanoseconds should be in the range of `i32`"),
        );
        NT_EPOCH
            .checked_add(duration)
            .ok_or(error::OffsetDateTimeRangeError)
    }
}

impl From<u64> for FileTime {
    /// Converts the Windows NT system time to a `FileTime`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nt_time::FileTime;
    /// #
    /// assert_eq!(FileTime::from(u64::MIN), FileTime::NT_EPOCH);
    /// assert_eq!(FileTime::from(u64::MAX), FileTime::MAX);
    /// ```
    #[inline]
    fn from(time: u64) -> Self {
        Self::new(time)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl TryFrom<std::time::SystemTime> for FileTime {
    type Error = error::FileTimeRangeError;

    /// Converts a [`SystemTime`](std::time::SystemTime) to a `FileTime`.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if `time` is out of range of the Windows NT system time.
    fn try_from(time: std::time::SystemTime) -> Result<Self, Self::Error> {
        let elapsed = time
            .duration_since(*SYSTEM_TIME_NT_EPOCH)
            .map(|d| d.as_nanos())
            .map_err(|_| error::FileTimeRangeError::new(error::FileTimeRangeErrorKind::Negative))?;
        let ft = u64::try_from(elapsed / 100)
            .map_err(|_| error::FileTimeRangeError::new(error::FileTimeRangeErrorKind::Overflow))?;
        Ok(Self::new(ft))
    }
}

impl TryFrom<OffsetDateTime> for FileTime {
    type Error = error::FileTimeRangeError;

    /// Converts a [`OffsetDateTime`] to a `FileTime`.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if `dt` is out of range of the Windows NT system time.
    fn try_from(dt: OffsetDateTime) -> Result<Self, Self::Error> {
        const NT_EPOCH: OffsetDateTime = datetime!(1601-01-01 00:00 UTC);
        let elapsed = core::time::Duration::try_from(dt - NT_EPOCH)
            .map(|d| d.as_nanos())
            .map_err(|_| error::FileTimeRangeError::new(error::FileTimeRangeErrorKind::Negative))?;
        let ft = u64::try_from(elapsed / 100)
            .map_err(|_| error::FileTimeRangeError::new(error::FileTimeRangeErrorKind::Overflow))?;
        Ok(Self::new(ft))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone() {
        assert_eq!(FileTime::NT_EPOCH.clone(), FileTime::NT_EPOCH);
    }

    #[test]
    fn copy() {
        let a = FileTime::NT_EPOCH;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", FileTime::NT_EPOCH), "FileTime(0)");
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash() {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };

        assert_ne!(
            {
                let mut hasher = DefaultHasher::new();
                FileTime::NT_EPOCH.hash(&mut hasher);
                hasher.finish()
            },
            {
                let mut hasher = DefaultHasher::new();
                FileTime::MAX.hash(&mut hasher);
                hasher.finish()
            }
        );
    }

    #[test]
    fn ord() {
        use core::cmp::Ordering;

        assert_eq!(FileTime::NT_EPOCH.cmp(&FileTime::MAX), Ordering::Less);
        assert_eq!(FileTime::NT_EPOCH.cmp(&FileTime::NT_EPOCH), Ordering::Equal);
        assert_eq!(FileTime::MAX.cmp(&FileTime::NT_EPOCH), Ordering::Greater);
    }

    #[test]
    fn partial_eq() {
        assert!(FileTime::NT_EPOCH == FileTime::NT_EPOCH);
    }

    #[test]
    fn partial_ord() {
        use core::cmp::Ordering;

        assert_eq!(
            FileTime::NT_EPOCH.partial_cmp(&FileTime::MAX),
            Some(Ordering::Less)
        );
        assert_eq!(
            FileTime::NT_EPOCH.partial_cmp(&FileTime::NT_EPOCH),
            Some(Ordering::Equal)
        );
        assert_eq!(
            FileTime::MAX.partial_cmp(&FileTime::NT_EPOCH),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn nt_epoch() {
        assert_eq!(
            OffsetDateTime::try_from(FileTime::NT_EPOCH).unwrap(),
            datetime!(1601-01-01 00:00 UTC)
        );
    }

    #[cfg(feature = "large-dates")]
    #[test]
    fn max() {
        assert_eq!(
            OffsetDateTime::try_from(FileTime::MAX).unwrap(),
            datetime!(+60056-05-28 05:36:10.955_161_500 UTC)
        );
    }

    #[test]
    fn new() {
        assert_eq!(FileTime::new(u64::MIN), FileTime::NT_EPOCH);
        assert_eq!(FileTime::new(u64::MAX), FileTime::MAX);
    }

    #[test]
    fn as_u64() {
        assert_eq!(FileTime::NT_EPOCH.as_u64(), u64::MIN);
        assert_eq!(FileTime::MAX.as_u64(), u64::MAX);
    }

    #[test]
    fn checked_add() {
        use core::time::Duration;

        assert_eq!(
            FileTime::NT_EPOCH.checked_add(Duration::from_nanos(1)),
            Some(FileTime::NT_EPOCH)
        );
        assert_eq!(
            FileTime::NT_EPOCH.checked_add(Duration::from_nanos(99)),
            Some(FileTime::NT_EPOCH)
        );
        assert_eq!(
            FileTime::NT_EPOCH.checked_add(Duration::from_nanos(100)),
            Some(FileTime::new(1))
        );

        assert_eq!(
            FileTime::MAX.checked_add(Duration::from_nanos(1)),
            Some(FileTime::MAX)
        );
        assert_eq!(
            FileTime::MAX.checked_add(Duration::from_nanos(99)),
            Some(FileTime::MAX)
        );
        assert_eq!(FileTime::MAX.checked_add(Duration::from_nanos(100)), None);
    }

    #[test]
    fn checked_sub() {
        use core::time::Duration;

        assert_eq!(
            FileTime::MAX.checked_sub(Duration::from_nanos(1)),
            Some(FileTime::MAX)
        );
        assert_eq!(
            FileTime::MAX.checked_sub(Duration::from_nanos(99)),
            Some(FileTime::MAX)
        );
        assert_eq!(
            FileTime::MAX.checked_sub(Duration::from_nanos(100)),
            Some(FileTime::new(u64::MAX - 1))
        );

        assert_eq!(
            FileTime::NT_EPOCH.checked_sub(Duration::from_nanos(1)),
            Some(FileTime::NT_EPOCH)
        );
        assert_eq!(
            FileTime::NT_EPOCH.checked_sub(Duration::from_nanos(99)),
            Some(FileTime::NT_EPOCH)
        );
        assert_eq!(
            FileTime::NT_EPOCH.checked_sub(Duration::from_nanos(100)),
            None
        );
    }

    #[test]
    fn saturating_add() {
        use core::time::Duration;

        assert_eq!(
            FileTime::NT_EPOCH.saturating_add(Duration::from_nanos(1)),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::NT_EPOCH.saturating_add(Duration::from_nanos(99)),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::NT_EPOCH.saturating_add(Duration::from_nanos(100)),
            FileTime::new(1)
        );

        assert_eq!(
            FileTime::MAX.saturating_add(Duration::from_nanos(1)),
            FileTime::MAX
        );
        assert_eq!(
            FileTime::MAX.saturating_add(Duration::from_nanos(99)),
            FileTime::MAX
        );
        assert_eq!(
            FileTime::MAX.saturating_add(Duration::from_nanos(100)),
            FileTime::MAX
        );
    }

    #[test]
    fn saturating_sub() {
        use core::time::Duration;

        assert_eq!(
            FileTime::MAX.saturating_sub(Duration::from_nanos(1)),
            FileTime::MAX
        );
        assert_eq!(
            FileTime::MAX.saturating_sub(Duration::from_nanos(99)),
            FileTime::MAX
        );
        assert_eq!(
            FileTime::MAX.saturating_sub(Duration::from_nanos(100)),
            FileTime::new(u64::MAX - 1)
        );

        assert_eq!(
            FileTime::NT_EPOCH.saturating_sub(Duration::from_nanos(1)),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::NT_EPOCH.saturating_sub(Duration::from_nanos(99)),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::NT_EPOCH.saturating_sub(Duration::from_nanos(100)),
            FileTime::NT_EPOCH
        );
    }

    #[test]
    fn default() {
        assert_eq!(FileTime::default(), FileTime::NT_EPOCH);
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", FileTime::NT_EPOCH), "0");
        assert_eq!(format!("{}", FileTime::MAX), "18446744073709551615");
    }

    #[test]
    fn add_duration() {
        use core::time::Duration;

        assert_eq!(
            FileTime::NT_EPOCH + Duration::from_nanos(1),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::NT_EPOCH + Duration::from_nanos(99),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::NT_EPOCH + Duration::from_nanos(100),
            FileTime::new(1)
        );

        assert_eq!(FileTime::MAX + Duration::from_nanos(1), FileTime::MAX);
        assert_eq!(FileTime::MAX + Duration::from_nanos(99), FileTime::MAX);
    }

    #[test]
    #[should_panic]
    fn add_duration_with_overflow() {
        use core::time::Duration;

        let _ = FileTime::MAX + Duration::from_nanos(100);
    }

    #[test]
    fn add_assign_duration() {
        use core::time::Duration;

        {
            let mut ft = FileTime::NT_EPOCH;
            ft += Duration::from_nanos(1);
            assert_eq!(ft, FileTime::NT_EPOCH);
        }
        {
            let mut ft = FileTime::NT_EPOCH;
            ft += Duration::from_nanos(99);
            assert_eq!(ft, FileTime::NT_EPOCH);
        }
        {
            let mut ft = FileTime::NT_EPOCH;
            ft += Duration::from_nanos(100);
            assert_eq!(ft, FileTime::new(1));
        }

        {
            let mut ft = FileTime::MAX;
            ft += Duration::from_nanos(1);
            assert_eq!(ft, FileTime::MAX);
        }
        {
            let mut ft = FileTime::MAX;
            ft += Duration::from_nanos(99);
            assert_eq!(ft, FileTime::MAX);
        }
    }

    #[test]
    #[should_panic]
    fn add_assign_duration_with_overflow() {
        use core::time::Duration;

        let mut ft = FileTime::MAX;
        ft += Duration::from_nanos(100);
    }

    #[test]
    fn sub_file_time() {
        use core::time::Duration;

        assert_eq!(
            FileTime::MAX - (FileTime::MAX - Duration::from_nanos(1)),
            Duration::ZERO
        );
        assert_eq!(
            FileTime::MAX - (FileTime::MAX - Duration::from_nanos(99)),
            Duration::ZERO
        );
        assert_eq!(
            FileTime::MAX - (FileTime::MAX - Duration::from_nanos(100)),
            Duration::from_nanos(100)
        );
    }

    #[test]
    #[should_panic]
    fn sub_file_time_with_overflow() {
        use core::time::Duration;

        let _ = (FileTime::MAX - Duration::from_nanos(100)) - FileTime::MAX;
    }

    #[test]
    fn sub_duration() {
        use core::time::Duration;

        assert_eq!(FileTime::MAX - Duration::from_nanos(1), FileTime::MAX);
        assert_eq!(FileTime::MAX - Duration::from_nanos(99), FileTime::MAX);
        assert_eq!(
            FileTime::MAX - Duration::from_nanos(100),
            FileTime::new(u64::MAX - 1)
        );

        assert_eq!(
            FileTime::NT_EPOCH - Duration::from_nanos(1),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::NT_EPOCH - Duration::from_nanos(99),
            FileTime::NT_EPOCH
        );
    }

    #[test]
    #[should_panic]
    fn sub_duration_with_overflow() {
        use core::time::Duration;

        let _ = FileTime::NT_EPOCH - Duration::from_nanos(100);
    }

    #[test]
    fn sub_assign_duration() {
        use core::time::Duration;

        {
            let mut ft = FileTime::MAX;
            ft -= Duration::from_nanos(1);
            assert_eq!(ft, FileTime::MAX);
        }
        {
            let mut ft = FileTime::MAX;
            ft -= Duration::from_nanos(99);
            assert_eq!(ft, FileTime::MAX);
        }
        {
            let mut ft = FileTime::MAX;
            ft -= Duration::from_nanos(100);
            assert_eq!(ft, FileTime::new(u64::MAX - 1));
        }

        {
            let mut ft = FileTime::NT_EPOCH;
            ft -= Duration::from_nanos(1);
            assert_eq!(ft, FileTime::NT_EPOCH);
        }
        {
            let mut ft = FileTime::NT_EPOCH;
            ft -= Duration::from_nanos(99);
            assert_eq!(ft, FileTime::NT_EPOCH);
        }
    }

    #[test]
    #[should_panic]
    fn sub_assign_duration_with_overflow() {
        use core::time::Duration;

        let mut ft = FileTime::NT_EPOCH;
        ft -= Duration::from_nanos(100);
    }

    #[test]
    fn from_file_time_to_u64() {
        assert_eq!(u64::from(FileTime::NT_EPOCH), u64::MIN);
        assert_eq!(u64::from(FileTime::MAX), u64::MAX);
    }

    #[cfg(feature = "std")]
    #[test]
    fn from_file_time_to_system_time() {
        use std::time::{Duration, SystemTime};

        assert_eq!(
            SystemTime::from(FileTime::NT_EPOCH),
            SystemTime::UNIX_EPOCH - Duration::from_secs(11_644_473_600)
        );
        assert_eq!(
            SystemTime::from(FileTime::new(116_444_736_000_000_000)),
            SystemTime::UNIX_EPOCH
        );
        assert_eq!(
            SystemTime::from(FileTime::new(2_650_467_743_999_999_999)),
            SystemTime::UNIX_EPOCH + Duration::new(253_402_300_799, 999_999_900)
        );
        assert_eq!(
            SystemTime::from(FileTime::new(2_650_467_744_000_000_000)),
            SystemTime::UNIX_EPOCH + Duration::from_secs(253_402_300_800)
        );
        #[cfg(not(windows))]
        assert_eq!(
            SystemTime::from(FileTime::MAX),
            SystemTime::UNIX_EPOCH + Duration::new(1_833_029_933_770, 955_161_500)
        );
    }

    #[test]
    fn try_from_file_time_to_offset_date_time() {
        assert_eq!(
            OffsetDateTime::try_from(FileTime::NT_EPOCH).unwrap(),
            datetime!(1601-01-01 00:00 UTC)
        );
        assert_eq!(
            OffsetDateTime::try_from(FileTime::new(116_444_736_000_000_000)).unwrap(),
            OffsetDateTime::UNIX_EPOCH
        );
        assert_eq!(
            OffsetDateTime::try_from(FileTime::new(2_650_467_743_999_999_999)).unwrap(),
            datetime!(9999-12-31 23:59:59.999_999_900 UTC)
        );
    }

    #[cfg(not(feature = "large-dates"))]
    #[test]
    fn try_from_file_time_to_offset_date_time_with_invalid_file_time() {
        let dt = OffsetDateTime::try_from(FileTime::new(2_650_467_744_000_000_000)).unwrap_err();
        assert_eq!(dt, error::OffsetDateTimeRangeError);
    }

    #[cfg(feature = "large-dates")]
    #[test]
    fn try_from_file_time_to_offset_date_time_with_large_dates() {
        assert_eq!(
            OffsetDateTime::try_from(FileTime::new(2_650_467_744_000_000_000)).unwrap(),
            datetime!(+10000-01-01 00:00 UTC)
        );
        assert_eq!(
            OffsetDateTime::try_from(FileTime::MAX).unwrap(),
            datetime!(+60056-05-28 05:36:10.955_161_500 UTC)
        );
    }

    #[test]
    fn from_u64_to_file_time() {
        assert_eq!(FileTime::from(u64::MIN), FileTime::NT_EPOCH);
        assert_eq!(FileTime::from(u64::MAX), FileTime::MAX);
    }

    #[cfg(feature = "std")]
    #[test]
    fn try_from_system_time_to_file_time_before_epoch() {
        use std::time::{Duration, SystemTime};

        let st = if cfg!(windows) {
            SystemTime::UNIX_EPOCH - Duration::from_nanos(11_644_473_600_000_000_100)
        } else {
            SystemTime::UNIX_EPOCH - Duration::from_nanos(11_644_473_600_000_000_001)
        };
        let ft = FileTime::try_from(st).unwrap_err();
        assert_eq!(
            ft,
            error::FileTimeRangeError::new(error::FileTimeRangeErrorKind::Negative)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn try_from_system_time_to_file_time() {
        use std::time::{Duration, SystemTime};

        assert_eq!(
            FileTime::try_from(SystemTime::UNIX_EPOCH - Duration::from_secs(11_644_473_600))
                .unwrap(),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::try_from(SystemTime::UNIX_EPOCH).unwrap(),
            FileTime::new(116_444_736_000_000_000)
        );
        assert_eq!(
            FileTime::try_from(
                SystemTime::UNIX_EPOCH + Duration::new(253_402_300_799, 999_999_900)
            )
            .unwrap(),
            FileTime::new(2_650_467_743_999_999_999)
        );
        assert_eq!(
            FileTime::try_from(SystemTime::UNIX_EPOCH + Duration::from_secs(253_402_300_800))
                .unwrap(),
            FileTime::new(2_650_467_744_000_000_000)
        );
        #[cfg(not(windows))]
        assert_eq!(
            FileTime::try_from(
                SystemTime::UNIX_EPOCH + Duration::new(1_833_029_933_770, 955_161_500)
            )
            .unwrap(),
            FileTime::MAX
        );
    }

    #[cfg(all(feature = "std", not(windows)))]
    #[test]
    fn try_from_system_time_to_file_time_with_too_big_system_time() {
        use std::time::{Duration, SystemTime};

        let st = FileTime::try_from(
            SystemTime::UNIX_EPOCH + Duration::new(1_833_029_933_770, 955_161_600),
        )
        .unwrap_err();
        assert_eq!(
            st,
            error::FileTimeRangeError::new(error::FileTimeRangeErrorKind::Overflow)
        );
    }

    #[test]
    fn try_from_offset_date_time_to_file_time_before_epoch() {
        let ft = FileTime::try_from(datetime!(1601-01-01 00:00 UTC) - time::Duration::NANOSECOND)
            .unwrap_err();
        assert_eq!(
            ft,
            error::FileTimeRangeError::new(error::FileTimeRangeErrorKind::Negative)
        );
    }

    #[test]
    fn try_from_offset_date_time_to_file_time() {
        assert_eq!(
            FileTime::try_from(datetime!(1601-01-01 00:00 UTC)).unwrap(),
            FileTime::NT_EPOCH
        );
        assert_eq!(
            FileTime::try_from(OffsetDateTime::UNIX_EPOCH).unwrap(),
            FileTime::new(116_444_736_000_000_000)
        );
        assert_eq!(
            FileTime::try_from(datetime!(9999-12-31 23:59:59.999_999_999 UTC)).unwrap(),
            FileTime::new(2_650_467_743_999_999_999)
        );
    }

    #[cfg(feature = "large-dates")]
    #[test]
    fn try_from_offset_date_time_to_file_time_with_large_dates() {
        assert_eq!(
            FileTime::try_from(datetime!(+10000-01-01 00:00 UTC)).unwrap(),
            FileTime::new(2_650_467_744_000_000_000)
        );
        assert_eq!(
            FileTime::try_from(datetime!(+60056-05-28 05:36:10.955_161_500 UTC)).unwrap(),
            FileTime::MAX
        );
    }

    #[cfg(feature = "large-dates")]
    #[test]
    fn try_from_offset_date_time_to_file_time_with_too_big_date_time() {
        let ft = FileTime::try_from(
            datetime!(+60056-05-28 05:36:10.955_161_500 UTC) + time::Duration::nanoseconds(100),
        )
        .unwrap_err();
        assert_eq!(
            ft,
            error::FileTimeRangeError::new(error::FileTimeRangeErrorKind::Overflow)
        );
    }
}
