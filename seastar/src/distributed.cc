#include "distributed.hh"
#include <seastar/util/defer.hh>

namespace seastar_ffi {
namespace distributed {

rust_service::rust_service(
    const uint8_t* raw_service_maker,
    rust::Fn<uint8_t*(const uint8_t*)> raw_service_maker_caller,
    rust::Fn<VoidFuture(uint8_t*)> stop_caller,
    rust::Fn<void(uint8_t*)> dropper)
: _stop_caller(stop_caller), _dropper(dropper)
{
    _inner = raw_service_maker_caller(raw_service_maker);
}

seastar::future<> rust_service::stop() {
    co_await _stop_caller(_inner);
}

rust_service::~rust_service() {
    _dropper(_inner);
}

std::shared_ptr<distributed> new_distributed() {
    return std::make_shared<distributed>();
}

const uint8_t* local(const distributed& distr) {
    return distr.local()._inner;
}

VoidFuture start(
    const distributed& distr,
    const uint8_t* raw_service_maker,
    rust::Fn<uint8_t*(const uint8_t*)> raw_service_maker_caller,
    rust::Fn<void(const uint8_t*)> raw_service_maker_dropper,
    rust::Fn<VoidFuture(uint8_t*)> stop_caller,
    rust::Fn<void(uint8_t*)> dropper
) {
    auto cleanup_maker = seastar::defer([&] {
        raw_service_maker_dropper(raw_service_maker);
    });
    co_await const_cast<distributed&>(distr).start(raw_service_maker, raw_service_maker_caller, stop_caller, dropper);
}

VoidFuture start_single(
    const distributed& distr,
    const uint8_t* raw_service_maker,
    rust::Fn<uint8_t*(const uint8_t*)> raw_service_maker_caller,
    rust::Fn<void(const uint8_t*)> raw_service_maker_dropper,
    rust::Fn<VoidFuture(uint8_t*)> stop_caller,
    rust::Fn<void(uint8_t*)> dropper
) {
    auto cleanup_maker = seastar::defer([&] {
        raw_service_maker_dropper(raw_service_maker);
    });
    co_await const_cast<distributed&>(distr).start_single(raw_service_maker, raw_service_maker_caller, stop_caller, dropper);
}

VoidFuture stop(const distributed& distr) {
    co_await const_cast<distributed&>(distr).stop();
}

} // namespace distributed
} // namespace seastar_ffi
