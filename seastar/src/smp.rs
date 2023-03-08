#[cxx::bridge]
mod ffi {
    #[namespace = "seastar"]
    unsafe extern "C++" {
        include!("seastar/core/smp.hh");

        /// Returns the `shard_id` of the current shard.
        fn this_shard_id() -> u32;
    }

    #[namespace = "seastar_ffi::smp"]
    unsafe extern "C++" {
        include!("seastar/src/smp.hh");

        /// Returns the total number of shards.
        fn get_count() -> u32;
    }
}

pub use ffi::{get_count, this_shard_id};

#[cfg(test)]
mod tests {
    use super::*;
    use crate as seastar;

    #[test]
    fn test_this_shard_id() {
        assert_eq!(this_shard_id(), 0);
    }

    #[seastar::test]
    async fn test_get_count_within_runtime() {
        // `num_cpus::get` is inconsistent with regards to cpu affinity,
        // see the discussion here: https://github.com/seanmonstar/num_cpus/pull/38
        // Thus, checking if *any* shards were created has to do for now.
        assert_ne!(get_count(), 0);
        // Outside of `AppTemplate::run{int, void}`, the function will yield 0,
        // but this is not tested here, as there's no control over the order of tests,
        // and Seastar doesn't clean up the variable that stores the cpu count (`seastar::smp::count`).
    }
}
