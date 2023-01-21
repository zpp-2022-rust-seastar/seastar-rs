#include "config_and_start_seastar.hh"

namespace seastar_ffi {
namespace config_and_start_seastar {

// Copies rust::Slice<const rust::Str> to char**. Returns nullptr if fails.
// Function free_args should be called on the returned pointer to avoid a memory leak.
static char** args_as_ptr(rust::Slice<const rust::Str> args) {
    char** av = new char*[args.size() + 1]();
    if (av == nullptr) {
        return nullptr;
    }

    for (size_t i = 0; i < args.size(); i++) {
        const auto& arg = *(args.begin() + i * sizeof(char));
        av[i] = new char[arg.size() + 1];
        if (av[i] == nullptr) {
            for (size_t j = 0; j < i; j++) {
                delete av[j];
            }
            return nullptr;
        }
        strncpy(av[i], arg.data(), arg.size());
    }

    return av;
}

static void free_args(char** av, size_t ac) {
    for (size_t i = 0; i < ac; i++) {
        delete av[i];
    }
    delete av;
}

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

int32_t run_void(app_template& app, rust::Slice<const rust::Str> args, VoidFuture fut) {
    int ac = args.size();
    char** av = args_as_ptr(args);
    if (av == nullptr) {
        return 1;
    }

    int32_t exit_value = app.run(ac, av, [&]() -> seastar::future<> {
         co_await std::move(fut);
    });

    free_args(av, args.size());
    return exit_value;
}

int32_t run_int(app_template& app, rust::Slice<const rust::Str> args, IntFuture fut) {
    int ac = args.size();
    char** av = args_as_ptr(args);
    if (av == nullptr) {
        return 1;
    }

    int32_t exit_value = app.run(ac, av, [&]() -> seastar::future<int> {
         co_return co_await std::move(fut);
    });

    free_args(av, args.size());
    return exit_value;
}

} // namespace config_and_start_seastar
} // namespace seastar