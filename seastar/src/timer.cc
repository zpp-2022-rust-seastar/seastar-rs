#include "timer.hh"

namespace seastar_ffi {
namespace timer {

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

} // namespace steady_clock

} // namespace timer
} // namespace seastar_ffi
