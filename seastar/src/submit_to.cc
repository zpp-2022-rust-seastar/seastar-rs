#include "submit_to.hh"
#include <seastar/core/smp.hh>

namespace seastar_ffi {

namespace submit_to {

VoidFuture submit_to(const uint32_t shard_id, uint8_t* closure, rust::Fn<VoidFuture(uint8_t*)> caller) {
    co_await ::seastar::smp::submit_to(shard_id, [&] () -> seastar::future<> {
        co_await caller(closure);
    });
}

} // submit_to

} // seastar_ffi
