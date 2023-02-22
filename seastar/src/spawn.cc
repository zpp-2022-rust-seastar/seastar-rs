#include <seastar/core/task.hh>
#include <seastar/core/future.hh>
#include <seastar/core/make_task.hh>
#include <seastar/core/coroutine.hh>

#include "spawn.hh"

namespace seastar_ffi {
namespace spawn {

VoidFuture cpp_spawn(VoidFuture future) {
    seastar::promise<> p;
    auto f = p.get_future();
    auto t = seastar::make_task([&]() -> seastar::future<> {
        auto local_p = std::move(p);
        co_await std::move(future);
        local_p.set_value(); 
    });
    seastar::schedule(t);
    co_await std::move(f);
}

}
}
