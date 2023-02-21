#pragma once

#include <seastar/core/timer.hh>
#include "rust/cxx.h"
#include "clocks.hh"

namespace seastar_ffi {
namespace timer {

using scheduling_group = seastar::scheduling_group;

namespace steady_clock {

using steady_clock_timer = seastar::timer<seastar::steady_clock_type>;

std::unique_ptr<steady_clock_timer> new_sct();

void sct_set_callback(
    steady_clock_timer& timer,
    uint8_t* callback, // uint8_t is a substitute for void that isn't supported by cxx.
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper
);

void sct_set_callback_under_group(
    steady_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper,
    const scheduling_group& sg
);

void sct_arm_at(steady_clock_timer& timer, int64_t at);

void sct_arm_at_periodic(steady_clock_timer& timer, int64_t at, int64_t period);

void sct_rearm_at(steady_clock_timer& timer, int64_t at);

void sct_rearm_at_periodic(steady_clock_timer& timer, int64_t at, int64_t period);

bool sct_armed(const steady_clock_timer& timer);

bool sct_cancel(steady_clock_timer& timer);

int64_t sct_get_timeout(const steady_clock_timer& timer);

} // namespace steady_clock

namespace lowres_clock {

using lowres_clock_timer = seastar::timer<seastar::lowres_clock>;

std::unique_ptr<lowres_clock_timer> new_lct();

void lct_set_callback(
    lowres_clock_timer& timer,
    uint8_t* callback, // uint8_t is a substitute for void that isn't supported by cxx.
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper
);

void lct_set_callback_under_group(
    lowres_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper,
    const scheduling_group& sg
);

void lct_arm_at(lowres_clock_timer& timer, int64_t at);

void lct_arm_at_periodic(lowres_clock_timer& timer, int64_t ay, int64_t period);

void lct_rearm_at(lowres_clock_timer& timer, int64_t at);

void lct_rearm_at_periodic(lowres_clock_timer& timer, int64_t at, int64_t period);

bool lct_armed(const lowres_clock_timer& timer);

bool lct_cancel(lowres_clock_timer& timer);

int64_t lct_get_timeout(const lowres_clock_timer& timer);

} // namespace lowres_clock

namespace manual_clock {

using manual_clock_timer = seastar::timer<seastar::manual_clock>;

std::unique_ptr<manual_clock_timer> new_mct();

void mct_set_callback(
    manual_clock_timer& timer,
    uint8_t* callback, // uint8_t is a substitute for void that isn't supported by cxx.
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper
);

void mct_set_callback_under_group(
    manual_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper,
    const scheduling_group& sg
);

void mct_arm_at(manual_clock_timer& timer, int64_t at);

void mct_arm_at_periodic(manual_clock_timer& timer, int64_t at, int64_t period);

void mct_rearm_at(manual_clock_timer& timer, int64_t at);

void mct_rearm_at_periodic(manual_clock_timer& timer, int64_t at, int64_t period);

bool mct_armed(const manual_clock_timer& timer);

bool mct_cancel(manual_clock_timer& timer);

int64_t mct_get_timeout(const manual_clock_timer& timer);

} // namespace manual_clock

} // namespace timer
} // namespace seastar_ffi
