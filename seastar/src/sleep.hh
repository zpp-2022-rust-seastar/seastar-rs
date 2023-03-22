#pragma once

#include "cxx_async_futures.hh"
#include <seastar/core/sleep.hh>
#include <seastar/core/manual_clock.hh>

namespace seastar_ffi {
namespace sleep {

VoidFuture steady_sleep(int64_t nanos);

VoidFuture lowres_sleep(int64_t nanos);

VoidFuture manual_sleep(int64_t nanos);

} // namespace sleep
} // namespace seastar_ffi
