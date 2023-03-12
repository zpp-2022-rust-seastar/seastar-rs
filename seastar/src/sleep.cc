#include "sleep.hh"

namespace seastar_ffi {
namespace sleep {

VoidFuture steady_sleep(int64_t nanos) {
    co_await seastar::sleep(std::chrono::nanoseconds(nanos));
}

VoidFuture lowres_sleep(int64_t nanos) {
    co_await seastar::sleep<seastar::lowres_clock>(std::chrono::nanoseconds(nanos));
}

VoidFuture manual_sleep(int64_t nanos) {
    co_await seastar::sleep<seastar::manual_clock>(std::chrono::nanoseconds(nanos));
}

} // namespace sleep
} // namespace seastar_ffi
