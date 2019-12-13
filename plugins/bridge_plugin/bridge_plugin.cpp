#include <eosio/bridge_plugin/bridge_plugin.hpp>
#include <eosio/chain/exceptions.hpp>
#include <boost/asio/steady_timer.hpp>
#include <fc/log/logger_config.hpp>

namespace eosio {
    static appbase::abstract_plugin &_bridge_plugin = app().register_plugin<bridge_plugin>();

    class bridge_plugin_impl {
    public:
        chain_plugin *chain_plug = nullptr;

        unique_ptr<boost::asio::steady_timer> block_timer;

        boost::asio::steady_timer::duration block_timeout{std::chrono::milliseconds{1000}};

        void block_timer_tick();
    };

    void bridge_plugin_impl::block_timer_tick() {
        block_timer->expires_from_now(block_timeout);
        block_timer->async_wait([&](boost::system::error_code ec) {
            block_timer_tick();

            chain_plug = app().find_plugin<chain_plugin>();
            uint32_t lib_block_num = chain_plug->chain().last_irreversible_block_num();
            ilog("lib_block_num: ${lib_block_num}", ("lib_block_num", lib_block_num));
        });
    }

    bridge_plugin::bridge_plugin() : my(new bridge_plugin_impl()) {}

    bridge_plugin::~bridge_plugin() {}

    void bridge_plugin::set_program_options(options_description &, options_description &cfg) {
        cfg.add_options()
                ("option-name", bpo::value<string>()->default_value("default value"),
                 "Option Description");
        ilog("bridge_plugin::set_program_options.");
    }

    void bridge_plugin::plugin_initialize(const variables_map &options) {
        ilog("bridge_plugin::plugin_initializ.");

        try {
            if (options.count("option-name")) {
                // Handle the option
            }

            my->block_timer = std::make_unique<boost::asio::steady_timer>(app().get_io_service());

        }
        FC_LOG_AND_RETHROW()
    }

    void bridge_plugin::plugin_startup() {
        // Make the magic happen
        ilog("bridge_plugin::plugin_startup.");

        my->block_timer_tick();
    }

    void bridge_plugin::plugin_shutdown() {
        // OK, that's enough magic
        ilog("bridge_plugin::plugin_shutdown.");
    }
}
