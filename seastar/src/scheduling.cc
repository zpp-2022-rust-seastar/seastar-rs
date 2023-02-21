#include "scheduling.hh"

namespace seastar_ffi {
namespace scheduling {

std::shared_ptr<scheduling_group> new_sg() {
    return std::make_unique<scheduling_group>();
}

bool sg_active(const scheduling_group& sg) {
    return sg.active();
}

rust::str sg_name(const scheduling_group& sg) {
    const seastar::sstring& sg_name = sg.name();
    return rust::Str(sg_name.begin(), sg_name.size());
}

bool sg_is_main(const scheduling_group& sg) {
    return sg.is_main();
}

void sg_set_shares(const std::shared_ptr<scheduling_group>& sg, float shares) {
    sg->set_shares(shares);
}

bool sg_equal(const scheduling_group& sg1, const scheduling_group& sg2) {
    return sg1 == sg2;
}

} // namespace scheduling
} // namespace seastar_ffi
