#pragma once

#include "cxx_async_futures.hh"
#include <seastar/core/distributed.hh>

namespace seastar_ffi {
namespace distributed {

class rust_service {
private:
    rust::Fn<VoidFuture(uint8_t*)> _stop_caller;
    rust::Fn<void(uint8_t*)> _dropper;
public:
    uint8_t* _inner;

    rust_service(
        const uint8_t* raw_service_maker,
        rust::Fn<uint8_t*(const uint8_t*)> raw_service_maker_caller,
        rust::Fn<VoidFuture(uint8_t*)> stop_caller,
        rust::Fn<void(uint8_t*)> dropper
    );
    ~rust_service();

    seastar::future<> stop();
};

using distributed = seastar::distributed<rust_service>;

std::shared_ptr<distributed> new_distributed();

const uint8_t* local(const distributed& distr);

VoidFuture start(
    const distributed& distr,
    const uint8_t* raw_service_maker,
    rust::Fn<uint8_t*(const uint8_t*)> raw_service_maker_caller,
    rust::Fn<void(const uint8_t*)> raw_service_maker_dropper,
    rust::Fn<VoidFuture(uint8_t*)> stop_caller,
    rust::Fn<void(uint8_t*)> dropper
);

VoidFuture start_single(
    const distributed& distr,
    const uint8_t* raw_service_maker,
    rust::Fn<uint8_t*(const uint8_t*)> raw_service_maker_caller,
    rust::Fn<void(const uint8_t*)> raw_service_maker_dropper,
    rust::Fn<VoidFuture(uint8_t*)> stop_caller,
    rust::Fn<void(uint8_t*)> dropper
);

VoidFuture stop(const distributed &distr);

} // namespace distributed
} // namespace seastar_ffi
