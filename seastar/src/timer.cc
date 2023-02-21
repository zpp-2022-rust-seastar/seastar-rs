#include "timer.hh"

namespace seastar_ffi {
namespace timer {

using seastar_ffi::clocks::to_nanos;

// This class was created to drop callbacks passed to `timer::set_calback` at the right
// time. We pass an instance of` callback_object` instead of a pure callback and the
// callback is dropped when the `callback_object`'s destructor is called.
class callback_object {
    uint8_t* _callback;
    rust::Fn<void(uint8_t*)> _caller;
    rust::Fn<void(uint8_t*)> _dropper;
    bool _valid = true;

public:
    callback_object(
        uint8_t* callback,
        rust::Fn<void(uint8_t*)> caller,
        rust::Fn<void(uint8_t*)> dropper
    ) : _callback(callback), _caller(caller), _dropper(dropper) {}

    callback_object(const callback_object&) = delete;

    callback_object& operator=(const callback_object&) = delete;

    callback_object(
        callback_object&& other
    ) : _callback(other._callback), _caller(other._caller), _dropper(other._dropper) {
        other._valid = false;
    }

    callback_object& operator=(callback_object&& other) {
        if (this != &other) {
            _callback = other._callback;
            _caller = other._caller;
            _dropper = other._dropper;
            other._valid = false;
        }

        return *this;
    }

    // We want to drop `_callback` only if an instance of `callback_object` wasn't move.
    // Without this, the destructor of a temporary instance created in `sct_set_callback`
    // would drop `_callback` too early and there would be a double drop.
    ~callback_object() {
        if (_valid) {
            _dropper(_callback);
        }
    }

    void operator()() {
        _caller(_callback);
    }
};

namespace steady_clock {

using sc = seastar::steady_clock_type;
using seastar_ffi::clocks::to_sc_duration;
using seastar_ffi::clocks::to_sc_time_point;

std::unique_ptr<steady_clock_timer> new_sct() {
    return std::make_unique<steady_clock_timer>();
}

void sct_set_callback(
    steady_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper
) {
    timer.set_callback(callback_object(callback, caller, dropper));
}

void sct_set_callback_under_group(
    steady_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper,
    const scheduling_group& sg
) {
    timer.set_callback(sg, callback_object(callback, caller, dropper));
}

void sct_arm_at(steady_clock_timer& timer, int64_t at) {
    timer.arm(to_sc_time_point(at));
}

void sct_arm_at_periodic(steady_clock_timer& timer, int64_t at, int64_t period) {
    timer.arm(to_sc_time_point(at), std::optional(to_sc_duration(period)));
}

void sct_rearm_at(steady_clock_timer& timer, int64_t at) {
    timer.rearm(to_sc_time_point(at));
}

void sct_rearm_at_periodic(steady_clock_timer& timer, int64_t at, int64_t period) {
    timer.rearm(to_sc_time_point(at), std::optional(to_sc_duration(period)));
}

bool sct_armed(const steady_clock_timer& timer) {
    return timer.armed();
}

bool sct_cancel(steady_clock_timer& timer) {
    return timer.cancel();
}

int64_t sct_get_timeout(const steady_clock_timer& timer) {
    return to_nanos(timer.get_timeout().time_since_epoch()).count();
}

} // namespace steady_clock

namespace lowres_clock {

using lc = seastar::lowres_clock;
using seastar_ffi::clocks::to_lc_duration;
using seastar_ffi::clocks::to_lc_time_point;

std::unique_ptr<lowres_clock_timer> new_lct() {
    return std::make_unique<lowres_clock_timer>();
}

void lct_set_callback(
    lowres_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper
) {
    timer.set_callback(callback_object(callback, caller, dropper));
}

void lct_set_callback_under_group(
    lowres_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper,
    const scheduling_group& sg
) {
    timer.set_callback(sg, callback_object(callback, caller, dropper));
}

void lct_arm_at(lowres_clock_timer& timer, int64_t at) {
    timer.arm(to_lc_time_point(at));
}

void lct_arm_at_periodic(lowres_clock_timer& timer, int64_t at, int64_t period) {
    timer.arm(to_lc_time_point(at), std::optional(to_lc_duration(period)));
}

void lct_rearm_at(lowres_clock_timer& timer, int64_t at) {
    timer.rearm(to_lc_time_point(at));
}

void lct_rearm_at_periodic(lowres_clock_timer& timer, int64_t at, int64_t period) {
    timer.rearm(to_lc_time_point(at), std::optional(to_lc_duration(period)));
}

bool lct_armed(const lowres_clock_timer& timer) {
    return timer.armed();
}

bool lct_cancel(lowres_clock_timer& timer) {
    return timer.cancel();
}

int64_t lct_get_timeout(const lowres_clock_timer& timer) {
    return to_nanos(timer.get_timeout().time_since_epoch()).count();
}

} // namespace lowres_clock

namespace manual_clock {

using mc = seastar::manual_clock;
using seastar_ffi::clocks::to_mc_duration;
using seastar_ffi::clocks::to_mc_time_point;

std::unique_ptr<manual_clock_timer> new_mct() {
    return std::make_unique<manual_clock_timer>();
}

void mct_set_callback(
    manual_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper
) {
    timer.set_callback(callback_object(callback, caller, dropper));
}

void mct_set_callback_under_group(
    manual_clock_timer& timer,
    uint8_t* callback,
    rust::Fn<void(uint8_t*)> caller,
    rust::Fn<void(uint8_t*)> dropper,
    const scheduling_group& sg
) {
    timer.set_callback(sg, callback_object(callback, caller, dropper));
}

void mct_arm_at(manual_clock_timer& timer, int64_t at) {
    timer.arm(to_mc_time_point(at));
}

void mct_arm_at_periodic(manual_clock_timer& timer, int64_t at, int64_t period) {
    timer.arm(to_mc_time_point(at), std::optional(to_mc_duration(period)));
}

void mct_rearm_at(manual_clock_timer& timer, int64_t at) {
    timer.rearm(to_mc_time_point(at));
}

void mct_rearm_at_periodic(manual_clock_timer& timer, int64_t at, int64_t period) {
    timer.rearm(to_mc_time_point(at), std::optional(to_mc_duration(period)));
}

bool mct_armed(const manual_clock_timer& timer) {
    return timer.armed();
}

bool mct_cancel(manual_clock_timer& timer) {
    return timer.cancel();
}

int64_t mct_get_timeout(const manual_clock_timer& timer) {
    return to_nanos(timer.get_timeout().time_since_epoch()).count();
}

} // namespace manual_clock

} // namespace timer
} // namespace seastar_ffi
