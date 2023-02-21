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

VoidFuture create_sg(std::shared_ptr<scheduling_group>& sg, rust::str name, float shares) {
    seastar::sstring s_name(name.begin(), name.size());
    scheduling_group new_sg = co_await seastar::create_scheduling_group(s_name, shares);
    sg = std::make_shared<scheduling_group>(std::move(new_sg));
}

VoidFuture destroy_sg(const std::shared_ptr<scheduling_group>& sg) {
    co_await seastar::destroy_scheduling_group(*sg);
}

VoidFuture rename_sg(const std::shared_ptr<scheduling_group>& sg, rust::str new_name) {
    seastar::sstring s_name(new_name.begin(), new_name.size());
    co_await seastar::rename_scheduling_group(*sg, s_name);
}

} // namespace scheduling
} // namespace seastar_ffi
