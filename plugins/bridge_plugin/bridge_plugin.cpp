#include <eosio/bridge_plugin/bridge_plugin.hpp>
#include <eosio/chain/exceptions.hpp>

#include <boost/multi_index_container.hpp>
#include <boost/asio/steady_timer.hpp>

#include <fc/log/logger_config.hpp>

namespace eosio {
   using boost::multi_index_container;
   using namespace boost::multi_index;

   static appbase::abstract_plugin &_bridge_plugin = app().register_plugin<bridge_plugin>();

   struct by_status;

   typedef multi_index_container<
           bridge_blocks,
           indexed_by<
                   ordered_unique<
                           tag<by_id>,
                           member<bridge_blocks,
                                  block_id_type,
                                  &bridge_blocks::id> >
           >
   > bridge_block_index;

   typedef multi_index_container<
           bridge_change_schedule,
           indexed_by<
                   ordered_unique<
                           tag<by_id>,
                           member<bridge_change_schedule,
                                  block_id_type,
                                  &bridge_change_schedule::id> >,
                   ordered_non_unique<
                           tag<by_status>,
                           member<bridge_change_schedule,
                                  uint8_t,
                                  &bridge_change_schedule::status> >
           >
   > bridge_change_schedule_index;

   typedef multi_index_container<
           bridge_prove_action,
           indexed_by<
                   ordered_unique<
                           tag<by_id>,
                           member<bridge_prove_action,
                                  block_id_type,
                                  &bridge_prove_action::id> >,
                   ordered_non_unique<
                           tag<by_status>,
                           member<bridge_prove_action,
                                  uint8_t,
                                  &bridge_prove_action::status > >
           >
   > bridge_prove_action_index;

   class bridge_plugin_impl {
   public:
      chain_plugin *chain_plug = nullptr;

      unique_ptr<boost::asio::steady_timer> block_timer;
      unique_ptr<boost::asio::steady_timer> change_schedule_timer;
      unique_ptr<boost::asio::steady_timer> prove_action_timer;

      boost::asio::steady_timer::duration block_timeout{std::chrono::milliseconds{1000}};
      boost::asio::steady_timer::duration change_schedule_timeout{std::chrono::milliseconds{1000}};
      boost::asio::steady_timer::duration prove_action_timeout{std::chrono::milliseconds{1000}};

      bridge_block_index            block_index;
      bridge_change_schedule_index  change_schedule_index;
      bridge_prove_action_index     prove_action_index;

      void block_timer_tick();

      void change_schedule_timer_tick();

      void prove_action_timer_tick();

      void irreversible_block(const chain::block_state_ptr &);
      void applied_transaction(std::tuple<const transaction_trace_ptr &, const signed_transaction &>);
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
      // TODO read blocks info to local storage
      ilog("irreversible_block: ${n}, id: ${id}", ("n", block->block_num)("id", block->id));
      auto bb = bridge_blocks{block->id, *block};

      // record block
      uint64_t block_index_max_size = 10;
      if (block_index.size() >= block_index_max_size) {
         block_index.erase(block_index.begin());
      }
      block_index.insert(bb);

      // check if block has new producers
      auto blk = block->block;
      if (blk->new_producers) {
         ilog("blk.new_producers: ${np}", ("np", blk->new_producers));
         auto cs = bridge_change_schedule{
                 block->id,
                 block->blockroot_merkle,
                 std::vector<signed_block_header>(),
                 std::vector<std::vector<block_id_type>>(),
                 0,
         };
         change_schedule_index.insert(cs);
      }

      ilog("block_index size: ${bi}", ("bi", block_index.size()));
   }

   void bridge_plugin_impl::applied_transaction(std::tuple<const transaction_trace_ptr &, const signed_transaction &> t) {
      auto tt = std::get<0>(t);
      ilog("applied_transaction => transaction_trace_ptr: ${tt},", ("tt", tt));

      std::vector<action_receipt> act_receipts;
      auto action_traces = tt->action_traces;
      for (auto &at : action_traces) {
         // TODO check if has withdraw/deposite transaction
         auto action = at.act;

         auto receipt = at.receipt;
         if (receipt) {
            act_receipts.push_back(*receipt);
         }
      }

      // TODO get merkle path

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
         cc.applied_transaction.connect(boost::bind(&bridge_plugin_impl::applied_transaction, my.get(), _1));

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
