use crate::{Clock, Duration};

/// Returns a future which completes after a specified duration has elapsed.
///
/// Uses `ClockType` as a clock.
pub async fn sleep<ClockType: Clock>(duration: Duration<ClockType>) {
    crate::assert_runtime_is_running();
    ClockType::sleep(duration.nanos).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as seastar;
    use crate::{LowresClock, ManualClock, SteadyClock};
    use std::time::SystemTime;

    #[seastar::test]
    async fn test_steady_clock_sleep() {
        let now = SystemTime::now();
        let duration = Duration::from_millis(100);
        sleep::<SteadyClock>(duration).await;
        let elapsed = now.elapsed().unwrap().as_millis() as i64;
        assert!(elapsed >= duration.as_millis());
    }

    #[seastar::test]
    async fn test_lowres_clock_sleep() {
        let now = SystemTime::now();
        let duration = Duration::from_millis(100);
        sleep::<LowresClock>(duration).await;
        let elapsed = now.elapsed().unwrap().as_millis() as i64;
        assert!(elapsed >= duration.as_millis());
    }

    #[seastar::test]
    async fn test_manual_clock_sleep() {
        let millis = 100;
        let duration = Duration::from_millis(millis);
        let advance_clock_future = seastar::spawn(async move {
            for _ in 0..2 * millis {
                sleep::<SteadyClock>(Duration::from_millis(1)).await;
                ManualClock::advance(Duration::from_millis(1));
            }
        });

        let before_sleep = ManualClock::now();
        sleep::<ManualClock>(duration).await;
        let after_sleep = ManualClock::now();
        assert!(after_sleep - before_sleep >= duration);
        advance_clock_future.await;
    }
}
