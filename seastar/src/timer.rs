use crate::assert_runtime_is_running;
use crate::ffi_utils::{get_dropper, get_fn_mut_void_caller};
use crate::{Clock, Duration, Instant};
use std::marker::PhantomData;

/// Runs a callback at a certain time point in the future.
///
/// Timer callbacks should execute quickly. If involved processing is required,
/// a timer can launch a continuation.
///
/// Timer has 3 specializations:
/// - `Timer<SteadyClock>` − has relatively high accuracy but is quite expensive.
/// - `Timer<LowresClock>` − has very coarse resolution (~10 ms) but is quite efficient.
/// - `Timer<ManualClock>` − used mainly for testing.
pub struct Timer<ClockType: Clock> {
    inner: ClockType::CppTimer,
    _phantom: PhantomData<ClockType>,
}

impl<ClockType: Clock> Default for Timer<ClockType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<ClockType: Clock> Timer<ClockType> {
    /// Constructs a timer with no callback set and no expiration time.
    pub fn new() -> Self {
        Self {
            inner: ClockType::new(),
            _phantom: PhantomData,
        }
    }

    /// Sets the callback function to be called when the timer expires.
    ///
    /// # Arguments
    /// * `callback` - The callback to be executed when the timer expires.
    pub fn set_callback<Func: FnMut() + 'static>(&mut self, callback: Func) {
        let caller = get_fn_mut_void_caller(&callback);
        let dropper = get_dropper(&callback);
        let boxed_callback = Box::into_raw(Box::new(callback)) as *mut u8;
        ClockType::set_callback(&mut self.inner, boxed_callback, caller, dropper);
    }

    /// Sets the timer expiration time.
    ///
    /// It is illegal to arm a timer that has already been armed (and not disarmed by
    /// expiration or [`Timer<ClockType>::cancel`]). In the current implementation, this
    /// will result in an assertion failure. See [`Timer<ClockType>::rearm_at`].
    ///
    /// # Arguments
    /// * `at` - The time when the timer expires.
    pub fn arm_at(&mut self, at: Instant<ClockType>) {
        assert_runtime_is_running();
        ClockType::arm_at(&mut self.inner, at.nanos);
    }

    /// Sets the timer expiration time with automatic rearming.
    ///
    /// It is illegal to arm a timer that has already been armed (and not disarmed by
    /// expiration or [`Timer<ClockType>::cancel`]). In the current implementation, this
    /// will result in an assertion failure. See [`Timer<ClockType>::rearm_at_periodic`].
    ///
    /// # Arguments
    /// * `at` - The time when the timer expires.
    /// * `period` - Automatic rearm duration.
    pub fn arm_at_periodic(&mut self, at: Instant<ClockType>, period: Duration<ClockType>) {
        assert_runtime_is_running();
        ClockType::arm_at_periodic(&mut self.inner, at.nanos, period.nanos);
    }

    /// Sets the timer expiration time. If the timer was already armed, it is
    /// canceled first.
    ///
    /// # Arguments
    /// * `at` - The time when the timer expires.
    pub fn rearm_at(&mut self, at: Instant<ClockType>) {
        assert_runtime_is_running();
        ClockType::rearm_at(&mut self.inner, at.nanos);
    }

    /// Sets the timer expiration time with automatic rearming. If the timer was
    /// already armed, it is canceled first.
    ///
    /// # Arguments
    /// * `at` - The time when the timer expires.
    /// * `period` - Automatic rearm duration.
    pub fn rearm_at_periodic(&mut self, at: Instant<ClockType>, period: Duration<ClockType>) {
        assert_runtime_is_running();
        ClockType::rearm_at_periodic(&mut self.inner, at.nanos, period.nanos);
    }

    /// Sets the timer expiration time relatively to now.
    ///
    /// It is illegal to arm a timer that has already been armed (and not disarmed by
    /// expiration or [`Timer<ClockType>::cancel`]). In the current implementation, this
    /// will result in an assertion failure. See [`Timer<ClockType>::rearm`].
    ///
    /// # Arguments
    /// * `delta` - The time when the timer expires, relative to now.
    pub fn arm(&mut self, delta: Duration<ClockType>) {
        self.arm_at(ClockType::now() + delta);
    }

    /// Sets the timer expiration time relatively to now. The timer will rearm
    /// automatically with a period equal to `delta`.
    ///
    /// It is illegal to arm a timer that has already been armed (and not disarmed by
    /// expiration or [`Timer<ClockType>::cancel`]). In the current implementation, this
    /// will result in an assertion failure. See [`Timer<ClockType>::rearm_periodic`].
    ///
    /// # Arguments
    /// * `delta` - The time when the timer expires, relative to now.
    pub fn arm_periodic(&mut self, delta: Duration<ClockType>) {
        self.arm_at_periodic(ClockType::now() + delta, delta);
    }

    /// Sets the timer expiration time relatively to now. If the timer was already armed,
    /// it is canceled first.
    ///
    /// # Arguments
    /// * `delta` - The time when the timer expires, relative to now.
    pub fn rearm(&mut self, delta: Duration<ClockType>) {
        self.rearm_at(ClockType::now() + delta);
    }

    /// Sets the timer expiration time relatively to now. If the timer was already armed,
    ///  it is canceled first. The timer will rearm automatically with a period equal
    /// to `delta`.
    ///
    /// # Arguments
    /// * `delta` - The time when the timer expires, relative to now.
    pub fn rearm_periodic(&mut self, delta: Duration<ClockType>) {
        self.rearm_at_periodic(ClockType::now() + delta, delta);
    }

    /// Returns whether the timer is armed.
    pub fn armed(&self) -> bool {
        ClockType::armed(&self.inner)
    }

    /// Cancels an armed timer.
    ///
    /// If the timer was armed, it is disarmed. If the timer was not armed, does nothing.
    ///
    /// Returns `true` if the timer was armed before the call.
    pub fn cancel(&mut self) -> bool {
        ClockType::cancel(&mut self.inner)
    }

    /// Gets the expiration time of a timer if it is armed. Otherwise, returns None.
    pub fn get_timeout(&self) -> Option<Instant<ClockType>> {
        if !self.armed() {
            return None;
        }

        return Some(Instant::new(ClockType::get_timeout(&self.inner)));
    }
}
