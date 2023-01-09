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
    if (opts->smp_opts.smp) {
        return (uint32_t)opts->smp_opts.smp.get_value();
    } else {
        return (uint32_t)get_current_cpuset().size();
    }
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

// Copies rust::Vec<rust::String> as char**. Returns nullptr if fails.
// Function free_args should be called on returned pointer to avoid memory leak.
static char** args_as_ptr(const rust::Vec<rust::String>& args) {
    char** av = (char**) calloc(args.size() + 1, sizeof(char*));
    if (av == nullptr) {
        return nullptr;
    }

    for (size_t i = 0; i < args.size(); i++) {
        const rust::String& arg = args[i];
        av[i] = (char*) calloc(arg.size() + 1, sizeof(char));
        if (av[i] == nullptr) {
            return nullptr;
        }
        strncpy(av[i], arg.data(), arg.size());
    }

    return av;
}

static void free_args(char** av, size_t ac) {
    for (size_t i = 0; i < ac; i++) {
        free(av[i]);
    }
    free(av);
}

int32_t run_void(const std::unique_ptr<seastar::app_template>& app, const rust::Vec<rust::String>& args, rust::Fn<void()> func) {
    char** av = args_as_ptr(args);
    if (av == nullptr) {
        return 1;
    }

    int32_t exit_value = app->run(args.size(), av, [&] {
        return seastar::make_ready_future<>().then([&] {
            func();
        });
    });

    free_args(av, args.size());

    return exit_value;
}

int32_t run_int(const std::unique_ptr<seastar::app_template>& app, const rust::Vec<rust::String>& args, rust::Fn<int()> func) {
    char** av = args_as_ptr(args);
    if (av == nullptr) {
        return 1;
    }

    int32_t exit_value = app->run(args.size(), av, [&] {
        return seastar::make_ready_future<>().then([&] {
            return func();
        });
    });

    free_args(av, args.size());

    return exit_value;
}

} // namespace seastar