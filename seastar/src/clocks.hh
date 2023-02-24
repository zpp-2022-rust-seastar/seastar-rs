#pragma once

#include <seastar/core/lowres_clock.hh>
#include <seastar/core/manual_clock.hh>

namespace seastar_ffi {
namespace clocks {

int64_t steady_clock_now();

int64_t lowres_clock_now();

int64_t manual_clock_now();

void manual_clock_advance(int64_t duration);

} // namespace clocks
} // namespace seastar_ffi
