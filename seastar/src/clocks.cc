#include "clocks.hh"

namespace seastar_ffi {
namespace clocks {

using sc = seastar::steady_clock_type;
using lc = seastar::lowres_clock;
using mc = seastar::manual_clock;

using std::chrono::duration_cast;

int64_t steady_clock_now() {
    static_assert(std::is_same<int64_t, sc::rep>::value);

    sc::duration d = sc::now().time_since_epoch();
    return to_nanos(d).count();
}

int64_t lowres_clock_now() {
    static_assert(std::is_same<int64_t, lc::rep>::value);

    lc::duration d = lc::now().time_since_epoch();
    return to_nanos(d).count();
}

int64_t manual_clock_now() {
    static_assert(std::is_same<int64_t, mc::rep>::value);

    mc::duration d = mc::now().time_since_epoch();
    return to_nanos(d).count();
}

void manual_clock_advance(int64_t duration) {
    mc::advance(to_mc_duration(duration));
}

sc::duration to_sc_duration(int64_t duration) {
    return duration_cast<sc::duration>(nanos(duration));
}

sc::time_point to_sc_time_point(int64_t tp) {
    return sc::time_point(to_sc_duration(tp));
}

lc::duration to_lc_duration(int64_t duration) {
    return duration_cast<lc::duration>(nanos(duration));
}

lc::time_point to_lc_time_point(int64_t tp) {
    return lc::time_point(to_lc_duration(tp));
}

mc::duration to_mc_duration(int64_t duration) {
    return duration_cast<mc::duration>(nanos(duration));
}

mc::time_point to_mc_time_point(int64_t tp) {
    return mc::time_point(to_mc_duration(tp));
}

} // namespace clocks
} // namespace seastar_ffi
