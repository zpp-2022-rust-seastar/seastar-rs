#include "scheduling.hh"

namespace seastar_ffi {
namespace scheduling {

std::shared_ptr<scheduling_group> new_sg() {
    return std::make_unique<scheduling_group>();
}

} // namespace scheduling
} // namespace seastar_ffi
