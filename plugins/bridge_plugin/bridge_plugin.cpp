#include <eosio/chain/types.hpp>

#include <eosio/bridge_plugin/bridge_plugin.hpp>
#include <eosio/chain/block.hpp>
#include <eosio/chain/exceptions.hpp>
#include <boost/asio/steady_timer.hpp>
#include <fc/log/logger_config.hpp>

namespace eosio {
   static appbase::abstract_plugin &_bridge_plugin = app().register_plugin<bridge_plugin>();

   class bridge_plugin_impl {
   public:
      chain_plugin *chain_plug = nullptr;

      unique_ptr<boost::asio::steady_timer> block_timer;
      unique_ptr<boost::asio::steady_timer> change_schedule_timer;
      unique_ptr<boost::asio::steady_timer> prove_action_timer;

      boost::asio::steady_timer::duration block_timeout{std::chrono::milliseconds{1000}};
      boost::asio::steady_timer::duration change_schedule_timeout{std::chrono::milliseconds{1000}};
      boost::asio::steady_timer::duration prove_action_timeout{std::chrono::milliseconds{1000}};

      void block_timer_tick();

      void change_schedule_timer_tick();

      void prove_action_timer_tick();

      void irreversible_block(const chain::block_state_ptr &);
   };

   void bridge_plugin_impl::block_timer_tick() {
      block_timer->expires_from_now(block_timeout);
      block_timer->async_wait([&](boost::system::error_code ec) {
         uint32_t lib_block_num = chain_plug->chain().last_irreversible_block_num();
         ilog("block_timer_tick: ${lib_block_num}", ("lib_block_num", lib_block_num));
         block_timer_tick();

         // TODO retrieve start_block_id and end_block_id
         // for block from start_block_id to end_block_id
         //    1. if is new_producers
         //      save new_producers and relevant info to local storage
         //    2. if is deposite/withdraw action
         //      record action and relevant info to local storage
      });
   }

   void bridge_plugin_impl::change_schedule_timer_tick() {
      change_schedule_timer->expires_from_now(change_schedule_timeout);
      change_schedule_timer->async_wait([&](boost::system::error_code ec) {
         change_schedule_timer_tick();

         // TODO read new_producers data
         // for new_producer in new_producers:
         //   if active_schedule is on chain（bifrost）&& irreversible block exceeded 15 * 12 blocks:
         //     retrieve relevant data from local storage
         //     send change_schedule transaction to bifrost
      });
   }

   void bridge_plugin_impl::prove_action_timer_tick() {
      prove_action_timer->expires_from_now(prove_action_timeout);
      prove_action_timer->async_wait([&](boost::system::error_code ec) {
         prove_action_timer_tick();

         // TODO read prove_actions data
         // for prove_action in prove_actions:
         //   if active_schedule is on chain（bifrost）&& irreversible block exceeded 15 * 12 blocks:
         //     retrieve relevant data from local storage
         //     send prove_action transaction to bifrost
      });
   }

   void bridge_plugin_impl::irreversible_block(const chain::block_state_ptr &block) {
      ilog("signaled, block: ${n}, id: ${id}", ("n", block->block_num)("id", block->id));
      // TODO read blocks info to local storage
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

         my->chain_plug = app().find_plugin<chain_plugin>();
         chain::controller &cc = my->chain_plug->chain();
         cc.irreversible_block.connect(boost::bind(&bridge_plugin_impl::irreversible_block, my.get(), _1));

         // init timer tick
         my->block_timer = std::make_unique<boost::asio::steady_timer>(app().get_io_service());
         my->change_schedule_timer = std::make_unique<boost::asio::steady_timer>(app().get_io_service());
         my->prove_action_timer = std::make_unique<boost::asio::steady_timer>(app().get_io_service());

      }
      FC_LOG_AND_RETHROW()
   }

   void bridge_plugin::plugin_startup() {
      // Make the magic happen
      ilog("bridge_plugin::plugin_startup.");

      // start timer tick
      my->block_timer_tick();
      my->change_schedule_timer_tick();
      my->prove_action_timer_tick();
   }

   void bridge_plugin::plugin_shutdown() {
      // OK, that's enough magic
      ilog("bridge_plugin::plugin_shutdown.");
   }
}
