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
///
/// # Examples
///
/// ### `Timer<SteadyClock>`
///
/// ```rust
/// #[seastar::test]
/// async fn steady_clock_example () {
///     let mut timer = seastar::Timer::<SteadyClock>::new();
///
///     let finished = std::rc::Rc::new(std::cell::RefCell::new(false));
///     let callback = || {
///         *finished.borrow_mut() = true;
///     };
///     timer.set_callback(callback);
///
///     let delta = seastar::Duration::from_secs(1);
///     timer.arm(delta);
///
///     seastar::steady_sleep(delta / 2).await;
///     assert!(!*finished.borrow());
///     seastar::steady_sleep(delta).await;
///     assert!(*finished.borrow());
/// }
/// ```
///
/// `Timer<LowresClock>` works exactly the same.
///
/// ### `Timer<ManualClock>`
///
/// ```rust
/// #[seastar::test]
/// async fn manual_clock_example () {
///     let mut timer = seastar::Timer::<ManualClock>::new();
///
///     let finished = std::rc::Rc::new(std::cell::RefCell::new(false));
///     let callback = || {
///         *finished.borrow_mut() = true;
///     };
///     timer.set_callback(callback);
///
///     let delta = seastar::Duration::from_secs(1);
///     timer.arm(delta);
///
///     seastar::ManualClock::advance(delta / 2);
///     assert!(!*finished.borrow());
///     seastar::ManualClock::advance(delta);
///     assert!(*finished.borrow());
/// }
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate as seastar;
    use crate::sleep;
    use crate::{LowresClock, ManualClock, SteadyClock};
    use std::cell::RefCell;
    use std::rc::Rc;

    async fn steady_clock_timer_wait(duration: Duration<SteadyClock>) {
        sleep(duration).await;
    }

    async fn lowres_clock_timer_wait(duration: Duration<LowresClock>) {
        sleep(duration).await;
    }

    async fn manual_clock_timer_wait(duration: Duration<ManualClock>) {
        ManualClock::advance(duration);
    }

    // Macro used to generate tests for the timer specializations.
    // - `Clock` - name of the corresponding clock type,
    // - `timer` - infix used to define test names,
    // - `wait` - name of the corresponding wait function (look above).
    macro_rules! test_timer {
        ($Clock:ty, $timer:ident, $wait:ident) => {
            paste::paste! {
                // Funtion doing generic start of every arm/ream timer's test.
                fn [<set_up_ $timer _test>]() -> (Timer<$Clock>, Duration<$Clock>, Rc<RefCell<u32>>) {
                    let mut timer = Timer::new();

                    let calls = Rc::new(RefCell::new(0));
                    let calls_cloned = calls.clone();
                    let callback = move || {
                        *calls_cloned.borrow_mut() += 1;
                    };
                    timer.set_callback(callback);

                    let duration = Duration::from_millis(100);

                    (timer, duration, calls)
                }

                // Funtion doing generic start of every non-periodic arm/ream timer's test.
                async fn [<check_ $timer>](
                    timer: &mut Timer<$Clock>,
                    duration: Duration<$Clock>,
                    calls: Rc<RefCell<u32>>,
                ) {
                    $wait(duration / 2).await;
                    assert!(*calls.borrow() == 0);
                    $wait(duration).await;
                    assert!(*calls.borrow() == 1);
                    $wait(duration).await;
                    assert!(*calls.borrow() == 1);
                    timer.cancel();
                }

                // Funtion doing generic start of every periodic arm/ream timer's test.
                async fn [<check_ $timer _periodic>](
                    timer: &mut Timer<$Clock>,
                    duration: Duration<$Clock>,
                    calls: Rc<RefCell<u32>>,
                ) {
                    $wait(duration / 2).await;
                    assert!(*calls.borrow() == 0);
                    $wait(duration).await;
                    assert!(*calls.borrow() == 1);
                    $wait(duration).await;
                    assert!(*calls.borrow() == 2);
                    timer.cancel();
                    $wait(duration).await;
                    assert!(*calls.borrow() == 2);
                }

                #[seastar::test]
                async fn [<test_ $timer _arm_at>]() {
                    let (mut timer, duration, calls) = [<set_up_ $timer _test>]();

                    let now = $Clock::now();
                    timer.arm_at(now + duration);

                    [<check_ $timer>](&mut timer, duration, calls).await
                }

                #[seastar::test]
                async fn [<test_ $timer _arm_at_periodic>]() {
                    let (mut timer, duration, calls) = [<set_up_ $timer _test>]();

                    let now = $Clock::now();
                    timer.arm_at_periodic(now + duration, duration);

                    [<check_ $timer _periodic>](&mut timer, duration, calls).await
                }

                #[seastar::test]
                async fn [<test_ $timer _rearm_at>]() {
                    let (mut timer, duration, calls) = [<set_up_ $timer _test>]();

                    let now = $Clock::now();
                    timer.arm(10 * duration);
                    timer.rearm_at(now + duration);

                    [<check_ $timer>](&mut timer, duration, calls).await
                }

                #[seastar::test]
                async fn [<test_ $timer _rearm_at_periodic>]() {
                    let (mut timer, duration, calls) = [<set_up_ $timer _test>]();

                    let now = $Clock::now();
                    timer.arm(10 * duration);
                    timer.rearm_at_periodic(now + duration, duration);

                    [<check_ $timer _periodic>](&mut timer, duration, calls).await
                }

                #[seastar::test]
                async fn [<test_ $timer _arm>]() {
                    let (mut timer, duration, calls) = [<set_up_ $timer _test>]();

                    timer.arm(duration);

                    [<check_ $timer>](&mut timer, duration, calls).await
                }

                #[seastar::test]
                async fn [<test_ $timer _arm_periodic>]() {
                    let (mut timer, duration, calls) = [<set_up_ $timer _test>]();

                    timer.arm_periodic(duration);

                    [<check_ $timer _periodic>](&mut timer, duration, calls).await;
                }

                #[seastar::test]
                async fn [<test_ $timer _rearm>]() {
                    let (mut timer, duration, calls) = [<set_up_ $timer _test>]();

                    timer.arm(10 * duration);
                    timer.rearm(duration);

                    [<check_ $timer>](&mut timer, duration, calls).await
                }

                #[seastar::test]
                async fn [<test_ $timer _rearm_periodic>]() {
                    let (mut timer, duration, calls) = [<set_up_ $timer _test>]();

                    timer.arm(10 * duration);
                    timer.rearm_periodic(duration);

                    [<check_ $timer _periodic>](&mut timer, duration, calls).await;
                }

                #[seastar::test]
                async fn [<test_ $timer _armed>]() {
                    let mut timer = Timer::<$Clock>::new();
                    assert!(!timer.armed());

                    timer.set_callback(|| {});
                    let duration = Duration::from_millis(100);
                    timer.arm(duration);
                    assert!(timer.armed());

                    $wait(duration * 2).await;
                    assert!(!timer.armed());
                }

                #[seastar::test]
                async fn [<test_ $timer _get_timeout>]() {
                    let mut timer = Timer::<$Clock>::new();
                    assert!(timer.get_timeout().is_none());

                    let now = $Clock::now();
                    let duration = Duration::from_secs(1);
                    timer.arm(duration);

                    let timeout = timer.get_timeout().unwrap();
                    let diff = timeout - now;
                    assert!(diff >= duration);
                }
            }
        };
    }

    test_timer!(SteadyClock, steady_clock_timer, steady_clock_timer_wait);

    test_timer!(LowresClock, lowres_clock_timer, lowres_clock_timer_wait);

    test_timer!(ManualClock, manual_clock_timer, manual_clock_timer_wait);
}
