#include "clocks.hh"

namespace seastar_ffi {
namespace clocks {

using sc = seastar::steady_clock_type;
using lc = seastar::lowres_clock;
using mc = seastar::manual_clock;

using nanos = std::chrono::nanoseconds;

using std::chrono::duration_cast;

int64_t steady_clock_now() {
    static_assert(std::is_same<int64_t, sc::rep>::value);

    sc::duration d = sc::now().time_since_epoch();
    return duration_cast<nanos>(d).count();
}

int64_t lowres_clock_now() {
    static_assert(std::is_same<int64_t, lc::rep>::value);

    lc::duration d = lc::now().time_since_epoch();
    return duration_cast<nanos>(d).count();
}

int64_t manual_clock_now() {
    static_assert(std::is_same<int64_t, mc::rep>::value);

    mc::duration d = mc::now().time_since_epoch();
    return duration_cast<nanos>(d).count();
}

void manual_clock_advance(int64_t duration) {
    nanos d(duration);
    mc::advance(duration_cast<mc::duration>(d));
}

} // namespace clocks
} // namespace seastar_ffi
