use core::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[cxx::bridge]
mod ffi {
    #[namespace = "seastar_ffi::clocks"]
    unsafe extern "C++" {
        include!("seastar/src/clocks.hh");

        fn steady_clock_now() -> i64;

        fn lowres_clock_now() -> i64;

        fn manual_clock_now() -> i64;

        fn manual_clock_advance(duration: i64);
    }
}

use ffi::*;

/// Type used by the `ClockType` clock to represent duration.
///
/// Note that, in contrast to `std::time::Duration`, values of this type
/// can be negative beacuse underlying implementation of
/// `std::chrono::duration` is expected tu use signed integers.
pub struct Duration<ClockType> {
    pub(crate) nanos: i64,
    _phantom: PhantomData<ClockType>,
}

impl<ClockType> PartialEq for Duration<ClockType> {
    fn eq(&self, other: &Self) -> bool {
        self.nanos == other.nanos
    }
}

impl<ClockType> Eq for Duration<ClockType> {}

impl<ClockType> Ord for Duration<ClockType> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nanos.cmp(&other.nanos)
    }
}

impl<ClockType> PartialOrd for Duration<ClockType> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<ClockType> Hash for Duration<ClockType> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nanos.hash(state);
    }
}

impl<ClockType> Clone for Duration<ClockType> {
    fn clone(&self) -> Self {
        Self {
            nanos: self.nanos,
            _phantom: PhantomData,
        }
    }
}

impl<ClockType> Copy for Duration<ClockType> {}

impl<ClockType> Default for Duration<ClockType> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<ClockType> fmt::Debug for Duration<ClockType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Duration")
            .field("nanos", &self.nanos)
            .finish()
    }
}

impl<ClockType> Duration<ClockType> {
    pub const MAX: Self = Self::from_nanos(i64::MAX);
    pub const MIN: Self = Self::from_nanos(i64::MIN);

    pub const NANOSECOND: Self = Self::from_nanos(1);
    pub const MICROSECOND: Self = Self::from_micros(1);
    pub const MILLISECOND: Self = Self::from_millis(1);
    pub const SECOND: Self = Self::from_secs(1);

    pub const ZERO: Self = Self::from_nanos(0);

    /// Returns the total number of nanoseconds contained by this duration.
    pub const fn as_nanos(&self) -> i64 {
        self.nanos
    }

    /// Returns the total number of microseconds contained by this duration.
    pub const fn as_micros(&self) -> i64 {
        self.nanos / 1000
    }

    /// Returns the total number of milliseconds contained by this duration.
    pub const fn as_millis(&self) -> i64 {
        self.nanos / 1_000_000
    }

    /// Returns the total number of seconds contained by this duration.
    pub const fn as_secs(&self) -> i64 {
        self.nanos / 1_000_000_000
    }

    /// Creates a new duration from the specified number of nanoseconds.
    pub const fn from_nanos(nanos: i64) -> Self {
        Self {
            nanos,
            _phantom: PhantomData,
        }
    }

    /// Creates a new duration from the specified number of microseconds.
    pub const fn from_micros(micros: i32) -> Self {
        Self {
            nanos: micros as i64 * 1000,
            _phantom: PhantomData,
        }
    }

    /// Creates a new duration from the specified number of milliseconds.
    pub const fn from_millis(millis: i32) -> Self {
        Self {
            nanos: millis as i64 * 1_000_000,
            _phantom: PhantomData,
        }
    }

    /// Creates a new duration from the specified number of seconds.
    pub const fn from_secs(secs: i32) -> Self {
        Self {
            nanos: secs as i64 * 1_000_000_000,
            _phantom: PhantomData,
        }
    }

    /// Returns true if this duration spans no time.
    pub const fn is_zero(&self) -> bool {
        self.nanos == 0
    }

    /// Checked duration addition. Computes `self + rhs`,
    /// returning `None` if overflow occurred.
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.nanos.checked_add(rhs.nanos) {
            Some(nanos) => Some(Self::from_nanos(nanos)),
            None => None,
        }
    }

    /// Checked duration substraction. Computes `self - rhs`,
    /// returning `None` if overflow occurred.
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        match self.nanos.checked_sub(rhs.nanos) {
            Some(nanos) => Some(Self::from_nanos(nanos)),
            None => None,
        }
    }

    /// Checked duration substraction. Computes `self * rhs`,
    /// returning `None` if overflow occurred.
    pub const fn checked_mul(self, rhs: i64) -> Option<Self> {
        match self.nanos.checked_mul(rhs) {
            Some(nanos) => Some(Self::from_nanos(nanos)),
            None => None,
        }
    }

    /// Checked duration substraction. Computes `self / rhs`,
    /// returning `None` if `rhs == 0`.
    pub const fn checked_div(self, rhs: i64) -> Option<Self> {
        match self.nanos.checked_div(rhs) {
            Some(nanos) => Some(Self::from_nanos(nanos)),
            None => None,
        }
    }
}

impl<ClockType> Add for Duration<ClockType> {
    type Output = Self;

    /// # Panics
    /// Panics if result overflows.
    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs).unwrap()
    }
}

impl<ClockType> AddAssign for Duration<ClockType> {
    /// # Panics
    /// Panics if result overflows.
    fn add_assign(&mut self, rhs: Self) {
        *self = self.checked_add(rhs).unwrap()
    }
}

impl<ClockType> Sub for Duration<ClockType> {
    type Output = Self;

    /// # Panics
    /// Panics if result overflows.
    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).unwrap()
    }
}

impl<ClockType> SubAssign for Duration<ClockType> {
    /// # Panics
    /// Panics if result overflows.
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.checked_sub(rhs).unwrap()
    }
}

impl<ClockType> Mul<i64> for Duration<ClockType> {
    type Output = Self;

    /// # Panics
    /// Panics if result overflows.
    fn mul(self, rhs: i64) -> Self {
        self.checked_mul(rhs).unwrap()
    }
}

impl<ClockType> Mul<Duration<ClockType>> for i64 {
    type Output = Duration<ClockType>;

    /// # Panics
    /// Panics if result overflows.
    fn mul(self, rhs: Duration<ClockType>) -> Duration<ClockType> {
        rhs.checked_mul(self).unwrap()
    }
}

impl<ClockType> MulAssign<i64> for Duration<ClockType> {
    /// # Panics
    /// Panics if result overflows.
    fn mul_assign(&mut self, rhs: i64) {
        *self = self.checked_mul(rhs).unwrap()
    }
}

impl<ClockType> Div<i64> for Duration<ClockType> {
    type Output = Self;

    /// # Panics
    /// Panics if `rhs == 0`.
    fn div(self, rhs: i64) -> Self {
        self.checked_div(rhs).unwrap()
    }
}

impl<ClockType> DivAssign<i64> for Duration<ClockType> {
    /// # Panics
    /// Panics if `rhs == 0`.
    fn div_assign(&mut self, rhs: i64) {
        *self = self.checked_div(rhs).unwrap()
    }
}

impl<ClockType> Neg for Duration<ClockType> {
    type Output = Self;

    /// # Panics
    /// Panics if overflow happens.
    fn neg(self) -> Self {
        match 0_i64.checked_sub(self.nanos) {
            Some(nanos) => Self::from_nanos(nanos),
            None => panic!(),
        }
    }
}

/// Type used by the `ClockType` clock to represent points in time.
///
/// `Instant<ClockType>` is implemented as if it stores a value of
/// [`Duration<ClockType>`] indicating the time interval from the start of the
/// `ClockType`'s epoch.
pub struct Instant<ClockType> {
    pub(crate) nanos: i64,
    _phantom: PhantomData<ClockType>,
}

impl<ClockType> PartialEq for Instant<ClockType> {
    fn eq(&self, other: &Self) -> bool {
        self.nanos == other.nanos
    }
}

impl<ClockType> Eq for Instant<ClockType> {}

impl<ClockType> Ord for Instant<ClockType> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nanos.cmp(&other.nanos)
    }
}

impl<ClockType> PartialOrd for Instant<ClockType> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<ClockType> Hash for Instant<ClockType> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nanos.hash(state);
    }
}

impl<ClockType> Clone for Instant<ClockType> {
    fn clone(&self) -> Self {
        Self {
            nanos: self.nanos,
            _phantom: PhantomData,
        }
    }
}

impl<ClockType> Copy for Instant<ClockType> {}

impl<ClockType> Default for Instant<ClockType> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<ClockType> fmt::Debug for Instant<ClockType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instant")
            .field("nanos", &self.nanos)
            .finish()
    }
}

impl<ClockType> Instant<ClockType> {
    pub(crate) const fn new(nanos: i64) -> Self {
        Self {
            nanos,
            _phantom: PhantomData,
        }
    }

    /// Returns `Some(t)` where `t` is equal to `self + duration` if `t` is
    /// inside the bounds of the underlying data structure, None otherwise.
    pub const fn checked_add(&self, duration: Duration<ClockType>) -> Option<Self> {
        match self.nanos.checked_add(duration.nanos) {
            Some(nanos) => Some(Self::new(nanos)),
            None => None,
        }
    }

    /// Returns `Some(t)` where `t` is equal to `self - duration` if `t` is
    /// inside the bounds of the underlying data structure, None otherwise.
    pub const fn checked_sub(&self, duration: Duration<ClockType>) -> Option<Self> {
        match self.nanos.checked_sub(duration.nanos) {
            Some(nanos) => Some(Self::new(nanos)),
            None => None,
        }
    }

    /// Returns the amount of time elapsed from another instant to this one.
    /// If `other` is later that `&self` the returned value is negative.
    ///
    /// # Panics
    /// Panics if an overflow in the underlying data structure happens.
    pub const fn duration_since(&self, other: Self) -> Duration<ClockType> {
        match self.nanos.checked_sub(other.nanos) {
            Some(nanos) => Duration::from_nanos(nanos),
            None => panic!(),
        }
    }

    /// Returns the amount of time elapsed from another instant to this one
    /// or None when overflow in the underlying data structure happens.
    pub const fn checked_duration_since(&self, other: Self) -> Option<Duration<ClockType>> {
        match self.nanos.checked_sub(other.nanos) {
            Some(nanos) => Some(Duration::from_nanos(nanos)),
            None => None,
        }
    }

    /// Returns a duration representing the amount of time between `&self` and
    /// the clock's epoch.
    ///
    /// Equivalent of `std::chrono::time_since_epoch`.
    pub const fn duration_since_epoch(&self) -> Duration<ClockType> {
        Duration::from_nanos(self.nanos)
    }
}

impl<ClockType> Add<Duration<ClockType>> for Instant<ClockType> {
    type Output = Self;

    /// # Panics
    /// Panics if an overflow in the underlying data structure happens.
    fn add(self, duration: Duration<ClockType>) -> Self {
        self.checked_add(duration).unwrap()
    }
}

impl<ClockType> AddAssign<Duration<ClockType>> for Instant<ClockType> {
    /// # Panics
    /// Panics if an overflow in the underlying data structure happens.
    fn add_assign(&mut self, duration: Duration<ClockType>) {
        *self = self.checked_add(duration).unwrap()
    }
}

impl<ClockType> Sub<Duration<ClockType>> for Instant<ClockType> {
    type Output = Self;

    /// # Panics
    /// Panics if an overflow in the underlying data structure happens.
    fn sub(self, duration: Duration<ClockType>) -> Self {
        self.checked_sub(duration).unwrap()
    }
}

impl<ClockType> SubAssign<Duration<ClockType>> for Instant<ClockType> {
    /// # Panics
    /// Panics if an overflow in the underlying data structure happens.
    fn sub_assign(&mut self, duration: Duration<ClockType>) {
        *self = self.checked_sub(duration).unwrap()
    }
}

impl<ClockType> Sub for Instant<ClockType> {
    type Output = Duration<ClockType>;

    /// Returns the amount of time elapsed from another instant to this one.
    /// If `other` is later that `self` the returned value is negative.
    ///
    /// # Panics
    /// Panics if an overflow in the underlying data structure happens.
    fn sub(self, other: Instant<ClockType>) -> Duration<ClockType> {
        self.duration_since(other)
    }
}

mod clock_implemantation {
    use super::*;

    // Hidden trait containing all clock specific ffi functions.
    pub trait ClockImpl: Sized {}
}

/// Trait implemented by: [`SteadyClock`], [`LowresClock`], [`ManualClock`].
pub trait Clock: clock_implemantation::ClockImpl {
    /// Returns an instant representing the current value of the clock.
    fn now() -> Instant<Self>;
}

/// Wrapper on `std::chrono::steady_clock`.
pub struct SteadyClock;

impl clock_implemantation::ClockImpl for SteadyClock {}

impl Clock for SteadyClock {
    fn now() -> Instant<SteadyClock> {
        Instant::new(steady_clock_now())
    }
}

/// Low-resolution and efficient steady clock.
///
/// Equivalent of `seastar::lowres_clock`.
///
/// This is a monotonic clock with a granularity of ~task_quota. Time points from this
/// this clock do not correspond to system time.
///
/// The primary benefit of this clock is that invoking [`Clock::now`] is inexpensive
/// compared to [`SteadyClock`].
pub struct LowresClock;

impl clock_implemantation::ClockImpl for LowresClock {}

impl Clock for LowresClock {
    fn now() -> Instant<LowresClock> {
        Instant::new(lowres_clock_now())
    }
}

/// Clock used mainly for testing.
///
/// Equivalent of `seastar::manual_clock`.
pub struct ManualClock;

impl clock_implemantation::ClockImpl for ManualClock {}

impl Clock for ManualClock {
    fn now() -> Instant<ManualClock> {
        Instant::new(manual_clock_now())
    }
}

impl ManualClock {
    /// Advances `ManualClock` by `duration`.
    ///
    /// Equivalent of `seastar::manual_clock::advance`.
    ///
    /// # Arguments
    /// * `duration` - The duration that the clock is advanced by.
    pub fn advance(duration: Duration<ManualClock>) {
        manual_clock_advance(duration.nanos);
    }
}
