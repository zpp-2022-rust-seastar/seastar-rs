#pragma once

#include "cxx_async_futures.hh"
#include <seastar/core/scheduling.hh>

namespace seastar_ffi {
namespace scheduling {

using scheduling_group = seastar::scheduling_group;

std::shared_ptr<scheduling_group> new_sg();

bool sg_active(const scheduling_group& sg);

rust::str sg_name(const scheduling_group& sg);

bool sg_is_main(const scheduling_group& sg);

void sg_set_shares(const std::shared_ptr<scheduling_group>& sg, float shares);

bool sg_equal(const scheduling_group& sg1, const scheduling_group& sg2);

} // namespace scheduling
} // namespace seastar_ffi
