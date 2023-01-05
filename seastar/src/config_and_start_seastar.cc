#include "config_and_start_seastar.hh"

namespace seastar {

std::unique_ptr<seastar_options> new_options() {
    return std::make_unique<seastar_options>();
}

rust::Str get_name(const std::unique_ptr<seastar_options>& opts) {
    return rust::Str(&*opts->name.begin(), opts->name.size());
}

rust::Str get_description(const std::unique_ptr<seastar_options>& opts) {
    return rust::Str(&*opts->description.begin(), opts->description.size());
}

uint32_t get_smp(const std::unique_ptr<seastar_options>& opts) {
    return (uint32_t)opts->smp_opts.smp;
}

void set_name(const std::unique_ptr<seastar_options>& opts, const rust::Str name) {
    opts->name = sstring(name.begin(), name.size());
}

void set_description(const std::unique_ptr<seastar_options>& opts, const rust::Str description) {
    opts->description = std::string(description.begin(), description.size());;
}

void set_smp(const std::unique_ptr<seastar_options>& opts, const uint32_t smp) {
    opts->smp_opts.smp.set_value((unsigned)smp);
}

std::unique_ptr<seastar::app_template> new_app_template_from_options(const std::unique_ptr<seastar_options>& opts) {
    return std::make_unique<seastar::app_template>(std::move(*opts));
}

//int run_void(std::unique_ptr<seastar::app_template>& app, int ac, int av, rust::Fn<void()> func) {
//    return app->run(ac, av, [] {
//        return seastar::make_ready_future<>().then([] {
//            func();
//        })
//    });
//}
//
//int run_int(std::unique_ptr<seastar::app_template>& app, int ac, int av, rust::Fn<int()> func) {
//    return app->run(ac, av, [] {
//        return seastar::make_ready_future<>().then([] {
//            return func();
//        })
//    });
//}

} // namespace seastar