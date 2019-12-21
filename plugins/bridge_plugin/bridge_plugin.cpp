#include <eosio/bridge_plugin/bridge_plugin.hpp>
#include <eosio/chain/exceptions.hpp>

#include <boost/multi_index_container.hpp>
#include <boost/asio/steady_timer.hpp>
#include <fc/io/fstream.hpp>
#include <fstream>
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

      fc::path datadir;

      void change_schedule_timer_tick();
      void prove_action_timer_tick();

      void irreversible_block(const chain::block_state_ptr &);
      void applied_transaction(std::tuple<const transaction_trace_ptr &, const signed_transaction &>);

      void open_db();
      void close_db();
   };

   void bridge_plugin_impl::change_schedule_timer_tick() {
      change_schedule_timer->expires_from_now(change_schedule_timeout);
      change_schedule_timer->async_wait([&](boost::system::error_code ec) {
         auto status_iter = change_schedule_index.get<by_status>().find( 1 );
         auto it = change_schedule_index.project<0>(status_iter);
         for (; it != change_schedule_index.end(); ++it) {
            // TODO send change_schedule transaction
         }

         change_schedule_timer_tick();
      });
   }

   void bridge_plugin_impl::prove_action_timer_tick() {
      prove_action_timer->expires_from_now(prove_action_timeout);
      prove_action_timer->async_wait([&](boost::system::error_code ec) {
         auto status_iter = prove_action_index.get<by_status>().find( 1 );
         auto it = prove_action_index.project<0>(status_iter);
         for (; it != prove_action_index.end(); ++it) {
            // TODO send prove_action transaction
         }

         prove_action_timer_tick();
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
      ilog("block_index size: ${bi}", ("bi", block_index.size()));

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

      // TODO check change_schedule_index
      auto cs_status_iter = change_schedule_index.get<by_status>().find(0);
      auto cs_it = change_schedule_index.project<0>(cs_status_iter);
      for (; cs_it != change_schedule_index.end(); ++cs_it) {

      }

      // TODO check prove_action_index
      auto pa_status_iter = prove_action_index.get<by_status>().find(0);
      auto pa_it = prove_action_index.project<0>(pa_status_iter);
      for (; pa_it != prove_action_index.end(); ++pa_it) {

      }
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

   void bridge_plugin_impl::open_db() {
      ilog("bridge_plugin_impl::open_db()");

      datadir = app().data_dir() / "bridge";
      if (!fc::is_directory(datadir))
         fc::create_directories(datadir);

      auto bridge_db_dat = datadir / config::bridgedb_filename;
      if (fc::exists(bridge_db_dat)) {
         try {
            string content;
            fc::read_file_contents(bridge_db_dat, content);
            fc::datastream<const char *> ds(content.data(), content.size());

            block_index.clear();
            change_schedule_index.clear();
            prove_action_index.clear();

            unsigned_int block_index_size;
            fc::raw::unpack(ds, block_index_size);
            for (uint32_t i = 0, n = block_index_size.value; i < n; ++i) {
               bridge_blocks bb;
               fc::raw::unpack(ds, bb);
               block_index.insert(bb);
            }

            unsigned_int change_schedule_index_size;
            fc::raw::unpack(ds, change_schedule_index_size);
            for (uint32_t i = 0, n = change_schedule_index_size.value; i < n; ++i) {
               bridge_change_schedule bcs;
               fc::raw::unpack(ds, bcs);
               change_schedule_index.insert(bcs);
            }

            unsigned_int prove_action_index_size;
            fc::raw::unpack(ds, prove_action_index_size);
            for (uint32_t i = 0, n = prove_action_index_size.value; i < n; ++i) {
               bridge_prove_action bpa;
               fc::raw::unpack(ds, bpa);
               prove_action_index.insert(bpa);
            }

         } FC_CAPTURE_AND_RETHROW((bridge_db_dat))

         fc::remove(bridge_db_dat);
      }
   }

   void bridge_plugin_impl::close_db() {
      ilog("bridge_plugin_impl::close_db()");
      auto bridge_db_dat = datadir / config::bridgedb_filename;

      std::ofstream out(bridge_db_dat.generic_string().c_str(), std::ios::out | std::ios::binary | std::ofstream::trunc);

      uint32_t block_index_size = block_index.size();
      fc::raw::pack(out, unsigned_int{block_index_size});
      auto block_iter = block_index.get<by_id>().begin();
      auto blk_it = block_index.project<0>(block_iter);
      for (; blk_it != block_index.end(); ++blk_it) {
         fc::raw::pack(out, *blk_it);
      }

      uint32_t change_schedule_index_size = change_schedule_index.size();
      fc::raw::pack(out, unsigned_int{change_schedule_index_size});
      auto cs_iter = change_schedule_index.get<by_id>().begin();
      auto cs_it = change_schedule_index.project<0>(cs_iter);
      for (; cs_it != change_schedule_index.end(); ++cs_it) {
         fc::raw::pack(out, *cs_it);
      }

      uint32_t prove_action_index_size = prove_action_index.size();
      fc::raw::pack(out, unsigned_int{prove_action_index_size});
      auto pa_iter = prove_action_index.get<by_id>().begin();
      auto pa_it = prove_action_index.project<0>(pa_iter);
      for (; pa_it != prove_action_index.end(); ++pa_it) {
         fc::raw::pack(out, *pa_it);
      }

      block_index.clear();
      change_schedule_index.clear();
      prove_action_index.clear();
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

         my->open_db();

         my->chain_plug = app().find_plugin<chain_plugin>();
         chain::controller &cc = my->chain_plug->chain();
         cc.irreversible_block.connect(boost::bind(&bridge_plugin_impl::irreversible_block, my.get(), _1));
         cc.applied_transaction.connect(boost::bind(&bridge_plugin_impl::applied_transaction, my.get(), _1));

         // init timer tick
         my->change_schedule_timer = std::make_unique<boost::asio::steady_timer>(app().get_io_service());
         my->prove_action_timer = std::make_unique<boost::asio::steady_timer>(app().get_io_service());

      }
      FC_LOG_AND_RETHROW()
   }

   void bridge_plugin::plugin_startup() {
      // Make the magic happen
      ilog("bridge_plugin::plugin_startup.");

      // start timer tick
      my->change_schedule_timer_tick();
      my->prove_action_timer_tick();
   }

   void bridge_plugin::plugin_shutdown() {
      // OK, that's enough magic
      ilog("bridge_plugin::plugin_shutdown.");

      my->close_db();
   }
}