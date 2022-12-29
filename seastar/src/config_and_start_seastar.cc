#include "config_and_start_seastar.hh"

std::unique_ptr<seastar_opts> new_options() {
    return std::make_unique<seastar_opts>(seastar_opts());
}

std::string get_name(const std::unique_ptr<const seastar_opts> opts) {
    return opts->name;
}

std::string get_description(const std::unique_ptr<const seastar_opts> opts) {
    return opts->description;
}

unsigned get_smp(const std::unique_ptr<const seastar_opts> opts) {
    return opts->smp_opts.smp;
}

void set_name(const std::unique_ptr<seastar_opts> opts, const rust::Str name) {
    opts->name = std::string(name.begin(), name.size());
}

void set_description(const std::unique_ptr<seastar_opts> opts, const rust::Str description) {
    opts->description = std::string(description.begin(), description.size());;
}

void set_smp(const std::unique_ptr<seastar_opts> opts, const unsigned smp) {
    opts->smp_opts.smp.set_value(smp);
}

std::unique_ptr<seastar::app_template> new_app_template_from_options(const std::unique_ptr<const seastar_opts>& opts) {
    return std::make_unique<seastar::app_template>(std::move(*opts));
}