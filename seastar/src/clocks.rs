use crate::cxx_async_futures::VoidFuture;
use core::cmp::Ordering;
use cxx::UniquePtr;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[cxx::bridge]
mod ffi {
    #[namespace = "seastar_ffi"]
    unsafe extern "C++" {
        type VoidFuture = crate::cxx_async_futures::VoidFuture;
    }

    #[namespace = "seastar_ffi::clocks"]
    unsafe extern "C++" {
        include!("seastar/src/clocks.hh");

        fn steady_clock_now() -> i64;

        fn lowres_clock_now() -> i64;

        fn manual_clock_now() -> i64;

        fn manual_clock_advance(duration: i64);
    }

    #[namespace = "seastar_ffi::sleep"]
    unsafe extern "C++" {
        include!("seastar/src/sleep.hh");

        fn steady_sleep(nanos: i64) -> VoidFuture;

        fn lowres_sleep(nanos: i64) -> VoidFuture;

        fn manual_sleep(nanos: i64) -> VoidFuture;
    }

    #[namespace = "seastar_ffi::timer::steady_clock"]
    unsafe extern "C++" {
        include!("seastar/src/timer.hh");

        type steady_clock_timer;

        fn new_sct() -> UniquePtr<steady_clock_timer>;

        unsafe fn sct_set_callback(
            timer: Pin<&mut steady_clock_timer>,
            callback: *mut u8, // u8 is a substitute for c_void that isn't supported by cxx.
            caller: unsafe fn(*mut u8),
            dropper: unsafe fn(*mut u8),
        );

        fn sct_arm_at(timer: Pin<&mut steady_clock_timer>, at: i64);
        fn sct_arm_at_periodic(timer: Pin<&mut steady_clock_timer>, at: i64, period: i64);

        fn sct_rearm_at(timer: Pin<&mut steady_clock_timer>, at: i64);
        fn sct_rearm_at_periodic(timer: Pin<&mut steady_clock_timer>, at: i64, period: i64);

        fn sct_armed(timer: &steady_clock_timer) -> bool;

        fn sct_cancel(timer: Pin<&mut steady_clock_timer>) -> bool;

        fn sct_get_timeout(timer: &steady_clock_timer) -> i64;
    }

    #[namespace = "seastar_ffi::timer::lowres_clock"]
    unsafe extern "C++" {
        include!("seastar/src/timer.hh");

        type lowres_clock_timer;

        fn new_lct() -> UniquePtr<lowres_clock_timer>;

        unsafe fn lct_set_callback(
            timer: Pin<&mut lowres_clock_timer>,
            callback: *mut u8, // u8 is a substitute for c_void that isn't supported by cxx.
            caller: unsafe fn(*mut u8),
            dropper: unsafe fn(*mut u8),
        );

        fn lct_arm_at(timer: Pin<&mut lowres_clock_timer>, at: i64);
        fn lct_arm_at_periodic(timer: Pin<&mut lowres_clock_timer>, at: i64, period: i64);

        fn lct_rearm_at(timer: Pin<&mut lowres_clock_timer>, at: i64);
        fn lct_rearm_at_periodic(timer: Pin<&mut lowres_clock_timer>, at: i64, period: i64);

        fn lct_armed(timer: &lowres_clock_timer) -> bool;

        fn lct_cancel(timer: Pin<&mut lowres_clock_timer>) -> bool;

        fn lct_get_timeout(timer: &lowres_clock_timer) -> i64;
    }

    #[namespace = "seastar_ffi::timer::manual_clock"]
    unsafe extern "C++" {
        include!("seastar/src/timer.hh");

        type manual_clock_timer;

        fn new_mct() -> UniquePtr<manual_clock_timer>;

        unsafe fn mct_set_callback(
            timer: Pin<&mut manual_clock_timer>,
            callback: *mut u8, // u8 is a substitute for c_void that isn't supported by cxx.
            caller: unsafe fn(*mut u8),
            dropper: unsafe fn(*mut u8),
        );

        fn mct_arm_at(timer: Pin<&mut manual_clock_timer>, at: i64);
        fn mct_arm_at_periodic(timer: Pin<&mut manual_clock_timer>, at: i64, period: i64);

        fn mct_rearm_at(timer: Pin<&mut manual_clock_timer>, at: i64);
        fn mct_rearm_at_periodic(timer: Pin<&mut manual_clock_timer>, at: i64, period: i64);

        fn mct_armed(timer: &manual_clock_timer) -> bool;

        fn mct_cancel(timer: Pin<&mut manual_clock_timer>) -> bool;

        fn mct_get_timeout(timer: &manual_clock_timer) -> i64;
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

mod clock_implementation {
    use super::*;

    // Hidden trait containing all clock specific ffi functions.
    pub trait ClockImpl: Sized {
        type CppTimer;

        fn sleep(nanos: i64) -> VoidFuture;

        fn new() -> Self::CppTimer;

        fn set_callback(
            timer: &mut Self::CppTimer,
            callback: *mut u8,
            caller: fn(*mut u8),
            dropper: fn(*mut u8),
        );

        fn arm_at(timer: &mut Self::CppTimer, at: i64);

        fn arm_at_periodic(timer: &mut Self::CppTimer, at: i64, period: i64);

        fn rearm_at(timer: &mut Self::CppTimer, at: i64);

        fn rearm_at_periodic(timer: &mut Self::CppTimer, at: i64, period: i64);

        fn armed(timer: &Self::CppTimer) -> bool;

        fn cancel(timer: &mut Self::CppTimer) -> bool;

        fn get_timeout(timer: &Self::CppTimer) -> i64;
    }
}

// Macro used to generate the implementation of the part of `ClockImpl`
// responsible for timer's ffi.
// - `cpp_timer` - timer from the ffi corresponding to the clock.
// - `ffi_pref` - prefix used by that variant's ffi functions.
macro_rules! timer_impl {
    ($cpp_timer:ident, $ffi_pref:ident) => {
        paste::paste! {
            type CppTimer = UniquePtr<$cpp_timer>;

            fn new() -> Self::CppTimer {
                [<new_ $ffi_pref>]()
            }

            fn set_callback(
                timer: &mut Self::CppTimer,
                callback: *mut u8,
                caller: fn(*mut u8),
                dropper: fn(*mut u8),
            ) {
                unsafe {
                    [<$ffi_pref _set_callback>](
                        timer.pin_mut(),
                        callback,
                        caller,
                        dropper,
                    );
                }
            }

            fn arm_at(timer: &mut Self::CppTimer, at: i64) {
                [<$ffi_pref _arm_at>](timer.pin_mut(), at);
            }

            fn arm_at_periodic(timer: &mut Self::CppTimer, at: i64, period: i64) {
                [<$ffi_pref _arm_at_periodic>](timer.pin_mut(), at, period);
            }

            fn rearm_at(timer: &mut Self::CppTimer, at: i64) {
                [<$ffi_pref _rearm_at>](timer.pin_mut(), at);
            }

            fn rearm_at_periodic(timer: &mut Self::CppTimer, at: i64, period: i64) {
                [<$ffi_pref _rearm_at_periodic>](timer.pin_mut(), at, period);
            }

            fn armed(timer: &Self::CppTimer) -> bool {
                [<$ffi_pref _armed>](timer)
            }

            fn cancel(timer: &mut Self::CppTimer) -> bool {
                [<$ffi_pref _cancel>](timer.pin_mut())
            }

            fn get_timeout(timer: &Self::CppTimer) -> i64 {
                [<$ffi_pref _get_timeout>](timer)
            }
        }
    };
}

/// Trait implemented by: [`SteadyClock`], [`LowresClock`], [`ManualClock`].
pub trait Clock: clock_implementation::ClockImpl {
    /// Returns an instant representing the current value of the clock.
    fn now() -> Instant<Self>;
}

/// Wrapper on `std::chrono::steady_clock`.
pub struct SteadyClock;

impl clock_implementation::ClockImpl for SteadyClock {
    fn sleep(nanos: i64) -> VoidFuture {
        steady_sleep(nanos)
    }

    timer_impl!(steady_clock_timer, sct);
}

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

impl clock_implementation::ClockImpl for LowresClock {
    fn sleep(nanos: i64) -> VoidFuture {
        lowres_sleep(nanos)
    }

    timer_impl!(lowres_clock_timer, lct);
}

impl Clock for LowresClock {
    fn now() -> Instant<LowresClock> {
        Instant::new(lowres_clock_now())
    }
}

/// Clock used mainly for testing.
///
/// Equivalent of `seastar::manual_clock`.
pub struct ManualClock;

impl clock_implementation::ClockImpl for ManualClock {
    fn sleep(nanos: i64) -> VoidFuture {
        manual_sleep(nanos)
    }

    timer_impl!(manual_clock_timer, mct);
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::catch_unwind;

    #[test]
    fn test_clocks_smoke_test() {
        // Clocks are also used in tests of the timer module.
        let _ = SteadyClock::now();
        let _ = LowresClock::now();
        let _ = ManualClock::now();
        ManualClock::advance(Duration::from_nanos(1000));
    }

    // Tests below test only `Instant<SteadyClock>` and `Duration<SteadyClock>`.
    // All instant and duration types have the same definition so it suffices.

    fn get_instants() -> (
        Instant<SteadyClock>,
        Instant<SteadyClock>,
        Instant<SteadyClock>,
    ) {
        let i1 = Instant::new(2);
        let i2 = Instant::new(-2);
        let i3 = Instant::new(i64::MAX);
        (i1, i2, i3)
    }

    #[test]
    fn test_instant_checked_operations() {
        let (i1, i2, _) = get_instants(); // (2, -2, _)
        let d1 = Duration::from_nanos(2);
        let d2 = Duration::from_nanos(i64::MAX);

        let i = i1.checked_add(d1).unwrap();
        assert_eq!(4, i.nanos);
        assert!(i1.checked_add(d2).is_none()); // 2 + i64::MAX (overflow)

        let i = i2.checked_sub(d1).unwrap();
        assert_eq!(-4, i.nanos);
        assert!(i2.checked_sub(d2).is_none()); // -2 - i64::MAX (overflow)
    }

    #[test]
    fn test_instant_since_methods() {
        let (i1, i2, i3) = get_instants(); // (2, -2, i64::MAX)

        let d = i1.duration_since(i2);
        assert_eq!(4, d.nanos);
        assert!(catch_unwind(|| i3.duration_since(i2)).is_err()); // i64::MAX - (-2) (overflow)

        let d = i1.checked_duration_since(i2).unwrap();
        assert_eq!(4, d.nanos);
        assert!(i3.checked_duration_since(i2).is_none()); // i64::MAX - (-2) (overflow)

        let d = i1.duration_since_epoch();
        assert_eq!(2, d.nanos);
    }

    #[test]
    fn test_instant_add() {
        let i1 = Instant::<SteadyClock>::new(2);
        let d1 = Duration::from_nanos(2);
        let d2 = Duration::from_nanos(i64::MAX);

        let i = i1 + d1; // 2 + 2
        assert_eq!(4, i.nanos);
        assert!(catch_unwind(|| i1 + d2).is_err()); // 2 + i64::MAX (overflow)

        let mut i = i1;
        i += d1; // 2 += 2
        assert_eq!(4, i.nanos);
        assert!(catch_unwind(|| {
            let mut i = i1;
            i += d2; // 2 + i64::MAX (overflow)
        })
        .is_err());
    }

    #[test]
    fn test_instant_sub() {
        let (i1, i2, i3) = get_instants(); // (2, -2, i64::MAX)
        let d1 = Duration::from_nanos(2);
        let d2 = Duration::from_nanos(i64::MAX);

        let i = i2 - d1;
        assert_eq!(-4, i.nanos);
        assert!(catch_unwind(|| i2 - d2).is_err()); // -2 - i64::MAX (overflow)

        let mut i = i2;
        i -= d1;
        assert_eq!(-4, i.nanos);
        assert!(catch_unwind(|| {
            let mut i = i2;
            i -= d2; // -2 - i64::MAX (overflow)
        })
        .is_err());

        let d = i1 - i2;
        assert_eq!(4, d.nanos);
        assert!(catch_unwind(|| i3 - i2).is_err()); // i64::MAX - (-2) (overflow)
    }

    fn get_durations() -> (
        Duration<SteadyClock>,
        Duration<SteadyClock>,
        Duration<SteadyClock>,
    ) {
        let d1 = Duration::from_nanos(1);
        let d2 = Duration::from_nanos(-2);
        let d3 = Duration::from_nanos(i64::MAX);
        (d1, d2, d3)
    }

    #[test]
    fn test_duration_as_and_from() {
        let secs = 123;
        let d_secs = Duration::<SteadyClock>::from_secs(secs);
        let d_millis = Duration::<SteadyClock>::from_millis(secs * 1000);
        let d_micros = Duration::<SteadyClock>::from_micros(secs * 1_000_000);
        let d_nanos = Duration::<SteadyClock>::from_nanos(secs as i64 * 1_000_000_000);

        assert_eq!(d_secs, d_millis);
        assert_eq!(d_millis, d_micros);
        assert_eq!(d_micros, d_nanos);

        assert_eq!(secs as i64, d_secs.as_secs());
        assert_eq!(secs as i64 * 1000, d_secs.as_millis());
        assert_eq!(secs as i64 * 1_000_000, d_secs.as_micros());
        assert_eq!(secs as i64 * 1_000_000_000, d_secs.as_nanos());
    }

    #[test]
    fn test_duration_is_zero() {
        let zero = Duration::<SteadyClock>::ZERO;
        assert!(zero.is_zero());
        let non_zero = Duration::<SteadyClock>::NANOSECOND;
        assert!(!non_zero.is_zero());
    }

    #[test]
    fn test_duration_checked_operations() {
        let (d1, d2, d3) = get_durations(); // (1, -2, i64::MAX)

        let d = d1.checked_add(d2).unwrap();
        assert_eq!(-1, d.nanos);
        assert!(d1.checked_add(d3).is_none()); // 1 + i64::MAX (overflow)

        let d = d1.checked_sub(d2).unwrap();
        assert_eq!(3, d.nanos);
        assert!(d1.checked_add(d3).is_none()); // -2 - i64::MAX (overflow)

        let d = d1.checked_mul(2).unwrap();
        assert_eq!(2, d.nanos);
        assert!(d3.checked_mul(2).is_none()); // i64::MAX * 2 (overflow)

        let d = d2.checked_div(2).unwrap();
        assert_eq!(-1, d.nanos);
        assert!(d1.checked_div(0).is_none()); // division by 0
    }

    #[test]
    fn test_duration_arithmetical_add() {
        let (d1, d2, d3) = get_durations(); // (1, -2, i64::MAX)

        let d = d1 + d2;
        assert_eq!(-1, d.nanos);
        assert!(catch_unwind(|| d1 + d3).is_err()); // 1 + i64::MAX (overflow)

        let mut d = d1;
        d += d2;
        assert_eq!(-1, d.nanos);
        assert!(catch_unwind(|| {
            let mut d = d1;
            d += d3; // 1 + i64::MAX (overflow)
        })
        .is_err());
    }

    #[test]
    fn test_duration_arithmetical_sub() {
        let (d1, d2, d3) = get_durations(); // (1, -2, i64::MAX)

        let d = d1 - d2;
        assert_eq!(3, d.nanos);
        assert!(catch_unwind(|| d2 - d3).is_err()); // -2 - i64::MAX (overflow)

        let mut d = d1;
        d -= d2;
        assert_eq!(3, d.nanos);
        assert!(catch_unwind(|| {
            let mut d = d2;
            d -= d3; // -2 - i64::MAX (overflow)
        })
        .is_err());
    }

    #[test]
    fn test_duration_arithmetical_mul() {
        let (d1, _, d3) = get_durations(); // (1, _, i64::MAX)

        let d = d1 * 2;
        assert_eq!(2, d.nanos);
        let d = 2 * d1;
        assert_eq!(2, d.nanos);
        assert!(catch_unwind(|| d3 * 2).is_err()); // i64::MAX * 2 (overflow)
        assert!(catch_unwind(|| 2 * d3).is_err()); // 2 * i64::MAX (overflow)

        let mut d = d1;
        d *= 2;
        assert_eq!(2, d.nanos);
        assert!(catch_unwind(|| {
            let mut d = d3;
            d *= 2; // i64::MAX * 2 (overflow)
        })
        .is_err());
    }

    #[test]
    fn test_duration_arithmetical_div() {
        let (d1, d2, _) = get_durations(); // (1, -2, _)

        let d = d2 / 2;
        assert_eq!(-1, d.nanos);
        assert!(catch_unwind(|| d1 / 0).is_err()); // division by 0

        let mut d = d2;
        d /= 2;
        assert_eq!(-1, d.nanos);
        assert!(catch_unwind(|| {
            let mut d = d1;
            d /= 0; // division by 0
        })
        .is_err());
    }

    #[test]
    fn test_duration_arithmetical_neg() {
        let d = -Duration::<SteadyClock>::from_nanos(1);
        assert_eq!(-1, d.nanos);
        let d = Duration::<SteadyClock>::MIN;
        assert!(catch_unwind(|| -d).is_err()); // -i64::MIN == i64::MAX + 1 (overflow)
    }
}
