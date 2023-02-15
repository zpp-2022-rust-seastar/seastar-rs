use crate::{Clock, Duration};

/// Returns a future which completes after a specified duration has elapsed.
///
/// Uses `ClockType` as a clock.
pub async fn sleep<ClockType: Clock>(duration: Duration<ClockType>) {
    crate::assert_runtime_is_running();
    ClockType::sleep(duration.nanos).await.unwrap();
}
