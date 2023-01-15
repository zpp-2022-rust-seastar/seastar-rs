#pragma once

#include <seastar/core/lowres_clock.hh>
#include <seastar/core/manual_clock.hh>

namespace seastar_ffi {
namespace clocks {

int64_t steady_clock_now();

int64_t lowres_clock_now();

int64_t manual_clock_now();

void manual_clock_advance(int64_t duration);

using nanos = std::chrono::nanoseconds;

template<typename Duration>
nanos to_nanos(Duration dur) {
    return std::chrono::duration_cast<nanos>(dur);
}

seastar::steady_clock_type::duration to_sc_duration(int64_t duration);

seastar::steady_clock_type::time_point to_sc_time_point(int64_t tp);

seastar::lowres_clock::duration to_lc_duration(int64_t duration);

seastar::lowres_clock::time_point to_lc_time_point(int64_t tp);

seastar::manual_clock::duration to_mc_duration(int64_t duration);

seastar::manual_clock::time_point to_mc_time_point(int64_t tp);

} // namespace clocks
} // namespace seastar_ffi
