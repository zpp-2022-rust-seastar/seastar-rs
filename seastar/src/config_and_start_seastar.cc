#include "config_and_start_seastar.hh"

namespace seastar_ffi {
namespace config_and_start_seastar {

std::unique_ptr<seastar_options> new_options() {
    return std::make_unique<seastar_options>();
}

rust::Str get_name(const seastar_options& opts) {
    return rust::Str(opts.name.begin(), opts.name.size());
}

rust::Str get_description(const seastar_options& opts) {
    return rust::Str(opts.description.begin(), opts.description.size());
}

uint32_t get_smp(const seastar_options& opts) {
    if (opts.smp_opts.smp) {
        return (uint32_t)opts.smp_opts.smp.get_value();
    } else {
        return (uint32_t)seastar::get_current_cpuset().size();
    }
}

void set_name(seastar_options& opts, const rust::Str name) {
    opts.name = seastar::sstring(name.begin(), name.size());
}

void set_description(seastar_options& opts, const rust::Str description) {
    opts.description = seastar::sstring(description.begin(), description.size());
}

void set_smp(seastar_options& opts, const uint32_t smp) {
    opts.smp_opts.smp.set_value((unsigned)smp);
}

std::unique_ptr<app_template> new_app_template_from_options(seastar_options& opts) {
    return std::make_unique<app_template>(std::move(opts));
}

int32_t run_void(app_template& app, int32_t argc, char** argv, VoidFuture fut) {
    int32_t exit_value = app.run((int)argc, argv, [&]() -> seastar::future<> {
        co_await std::move(fut);
    });
    return exit_value;
}

int32_t run_int(app_template& app, int32_t argc, char** argv, IntFuture fut) {
    int32_t exit_value = app.run((int)argc, argv, [&]() -> seastar::future<int> {
        co_return co_await std::move(fut);
    });
    return exit_value;
}

} // namespace config_and_start_seastar
} // namespace seastar
