#include "smp.hh"
#include <seastar/core/smp.hh>

namespace seastar_ffi {
namespace smp {

uint32_t get_count() {
    return (uint32_t)seastar::smp::count;
}

} // namespace smp
} // namespace seastar_ffi
