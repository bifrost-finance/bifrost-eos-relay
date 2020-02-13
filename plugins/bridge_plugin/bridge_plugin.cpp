#include "bifrost_rpc.h"
#include <eosio/bridge_plugin/bridge_plugin.hpp>
#include <eosio/bridge_plugin/ffi_types.hpp>
#include <eosio/chain/exceptions.hpp>
#include <eosio/chain/merkle.hpp>
#include <eosio/chain/types.hpp>

#include <boost/multi_index_container.hpp>
#include <boost/asio/steady_timer.hpp>
#include <fc/io/fstream.hpp>
#include <fstream>
#include <fc/log/logger_config.hpp>
#include <fc/io/json.hpp>

namespace eosio {
   using boost::multi_index_container;
   using namespace boost::multi_index;

   static appbase::abstract_plugin &_bridge_plugin = app().register_plugin<bridge_plugin>();

   struct by_status;
   digest_type digest(const action &act) { return digest_type::hash(act); }

   typedef multi_index_container<
           bridge_blocks,
           indexed_by<
              ordered_unique<tag<by_id>,
              member<bridge_blocks, block_id_type, &bridge_blocks::id>>
           >
   > bridge_block_index;

   typedef multi_index_container<
           bridge_change_schedule,
           indexed_by<
              ordered_unique<
                 tag<by_id>,
                 member<bridge_change_schedule, uint32_t, &bridge_change_schedule::block_num>
              >,
              ordered_non_unique<
                 tag<by_status>,
                 member<bridge_change_schedule, uint8_t, &bridge_change_schedule::status>
              >
           >
   > bridge_change_schedule_index;

   typedef multi_index_container<
           bridge_prove_action,
           indexed_by<
              ordered_unique<
                 tag<by_id>,
                 member<bridge_prove_action, block_id_type, &bridge_prove_action::act_receipt_digest>
              >,
              ordered_non_unique<
                 tag<by_status>,
                 member<bridge_prove_action, uint8_t, &bridge_prove_action::status>
              >
           >
    > bridge_prove_action_index;

   struct bifrost_config {
      std::string bifrost_addr;
      std::string bifrost_account;
   };

   class bridge_plugin_impl {
   public:
      chain_plugin *chain_plug = nullptr;

      unique_ptr<boost::asio::steady_timer> change_schedule_timer;
      unique_ptr<boost::asio::steady_timer> prove_action_timer;

      boost::asio::steady_timer::duration change_schedule_timeout{std::chrono::milliseconds{1000}};
      boost::asio::steady_timer::duration prove_action_timeout{std::chrono::milliseconds{1000}};

      bridge_block_index            block_index;
      bridge_change_schedule_index  change_schedule_index;
      bridge_prove_action_index     prove_action_index;

      bifrost_config config;

      fc::path datadir;

      void change_schedule_timer_tick();
      void prove_action_timer_tick();

      void collect_blocks_timer_tick();

      void irreversible_block(const chain::block_state_ptr &);
      void apply_action_receipt(std::tuple<const transaction_trace_ptr &, const std::vector<action_receipt>&>);

      void open_db();
      void close_db();

      std::tuple<std::vector<signed_block_header>, std::vector<std::vector<block_id_type>>> collect_incremental_merkle_and_blocks(bridge_change_schedule_index::iterator &);
      std::tuple<std::vector<signed_block_header>, std::vector<std::vector<block_id_type>>> collect_incremental_merkle_and_blocks(bridge_prove_action_index::iterator &);

      void filter_action(const std::string &contract, const std::vector<action_trace> &, const std::vector<action_receipt> &);
   };

   std::tuple<std::vector<signed_block_header>, std::vector<std::vector<block_id_type>>> bridge_plugin_impl::collect_incremental_merkle_and_blocks(bridge_prove_action_index::iterator &ti) {
      auto bl_state = block_state();
      std::vector<signed_block_header> block_headers; // can reserve a buffer to store id
      block_headers.reserve(15);
      for (auto bls: ti->bs) {
         if (ti->block_num == bls.block_num) { // which block is need to be verified
            block_headers.push_back(bls.header);
            bl_state = bls;
            break;
         }
      }

      std::vector<std::vector<block_id_type>> block_id_lists; // can reserve a buffer to store id
      block_id_lists.reserve(15);

      auto reserved = std::vector<block_id_type>();
      reserved.reserve(10);
      block_id_lists.push_back(std::vector<block_id_type>());
      block_id_lists.push_back(reserved);
      for (auto bls: ti->bs) {
         if (bls.block_num <= ti->block_num) continue;
         if (bls.block_num - block_headers.back().block_num() == 12) {
            block_headers.push_back(bls.header);
            if (block_headers.size() >= 15) break;
            block_id_lists.push_back(std::vector<block_id_type>());
         } else {
            auto block_ids = block_id_lists.back();
            if (block_ids.size() < 10) block_id_lists.back().push_back(bls.id);
         }
         if (block_id_lists.size() >= 15 && block_id_lists.back().size() >= 10 && block_headers.size() >= 15) break;
      }

      // get incremental_merkle
      auto pre_block_state = block_index.find(bl_state.header.previous);
      auto blockroot_merkle = pre_block_state->bls.blockroot_merkle;
      if (blockroot_merkle._node_count != 0)  {
         prove_action_index.modify(ti, [&](auto &entry) {
            entry.imcre_merkle = blockroot_merkle;
         });
      }

      return std::make_tuple(block_headers, block_id_lists);
   }

   std::tuple<std::vector<signed_block_header>, std::vector<std::vector<block_id_type>>> bridge_plugin_impl::collect_incremental_merkle_and_blocks(bridge_change_schedule_index::iterator &ti) {
      auto bl_state = block_state();
      std::vector<signed_block_header> block_headers; // can reserve a buffer to store id
      block_headers.reserve(15);
      for (auto bls: ti->bs) {
         if (ti->block_num == bls.block_num) {
            block_headers.push_back(bls.header);
            bl_state = bls;
            break;
         }
      }

      std::vector<std::vector<block_id_type>>   block_id_lists; // can reserve a buffer to store id
      block_id_lists.reserve(15);
      block_id_lists.push_back(std::vector<block_id_type>());
      block_id_lists.push_back(std::vector<block_id_type>());
      for (auto bls: ti->bs) {
         if (bls.block_num <= ti->block_num) continue;
         if (bls.block_num - block_headers.back().block_num() == 12) {
            block_headers.push_back(bls.header);
            if (block_headers.size() >= 15) break;
            block_id_lists.push_back(std::vector<block_id_type>());
         } else {
            auto block_ids = block_id_lists.back();
            if (block_ids.size() < 10) block_id_lists.back().push_back(bls.id);
         }
         if (block_id_lists.size() >= 15 && block_id_lists.back().size() >= 10 && block_headers.size() >= 15) break;
      }

      // get incremental_merkle
      auto pre_block_state = block_index.find(bl_state.header.previous);
      auto blockroot_merkle = pre_block_state->bls.blockroot_merkle;
      if (blockroot_merkle._node_count != 0) {
         change_schedule_index.modify(ti, [&](auto &entry) {
            entry.imcre_merkle = blockroot_merkle;
         });
      }

      return std::make_tuple(block_headers, block_id_lists);
   }

   void bridge_plugin_impl::change_schedule_timer_tick() {
      change_schedule_timer->expires_from_now(change_schedule_timeout);
      change_schedule_timer->async_wait([&](boost::system::error_code ec) {
         for (auto ti = change_schedule_index.begin(); ti != change_schedule_index.end(); ++ti) {
            if (ti->status != 1) continue;

            auto tuple = collect_incremental_merkle_and_blocks(ti);
            incremental_merkle blockroot_merkle = ti->imcre_merkle;
            auto block_headers = std::get<0>(tuple);
            auto block_id_lists = std::get<1>(tuple);

            signed_block_header_ffi *blocks_ffi = new signed_block_header_ffi[block_headers.size()];
            for (size_t i = 0; i < block_headers.size(); ++i) {
               auto p = new signed_block_header_ffi(block_headers[i]);
               blocks_ffi[i] = *p;
            }

            auto merkle_ptr = convert_ffi(blockroot_merkle);

            block_id_type_list *ids_list = new block_id_type_list[block_id_lists.size()];
            for (size_t i = 0; i < block_id_lists.size(); ++i) {
               if (block_id_lists[i].empty()) {
                  ids_list[i] = block_id_type_list();
                  continue;
               }
               ids_list[i] = convert_ffi(block_id_lists[i]);
            }

            rpc_result *result = change_schedule(
               config.bifrost_addr.data(),
               config.bifrost_account.data(),
               &merkle_ptr,
               blocks_ffi,
               block_headers.size(),
               ids_list,
               block_id_lists.size()
            );

            if (result) { // not null
               if (result->success) {
                  change_schedule_index.modify(ti, [&](auto &entry) {
                     entry.status = 2; // sent successfully
                  });
                  ilog("sent data to bifrost for changing schedule.");
                  ilog("Transaction got finalized. Hash: ${hash}.", ("hash", std::string(result->msg)));
               } else {
                  ilog("failed to send data to bifrost for changing schedule due to: ${err}.", ("err", std::string(result->msg)));
               }
            }

            delete []blocks_ffi;
            delete []ids_list;
         }

         change_schedule_timer_tick();
      });
   }

   void bridge_plugin_impl::prove_action_timer_tick() {
      prove_action_timer->expires_from_now(prove_action_timeout);
      prove_action_timer->async_wait([&](boost::system::error_code ec) {
         for (auto ti = prove_action_index.begin(); ti != prove_action_index.end(); ++ti) {
            ilog("headers length: ${header_len}. status: ${status}", ("header_len", ti->bs.size())("status", ti->status));
            if (ti->status != 1) continue;

            auto tuple = collect_incremental_merkle_and_blocks(ti);
            incremental_merkle blockroot_merkle = ti->imcre_merkle;
            auto block_headers = std::get<0>(tuple);
            auto block_id_lists = std::get<1>(tuple);

            signed_block_header_ffi *blocks_ffi = new signed_block_header_ffi[block_headers.size()];
            for (size_t i = 0; i < block_headers.size(); ++i) {
               auto p = new signed_block_header_ffi(block_headers[i]);
               blocks_ffi[i] = *p;
            }

            auto receipts = action_receipt_ffi(ti->receipt);
            auto act_ffi = action_ffi(ti->act);
            auto merkle_ptr = convert_ffi(blockroot_merkle);

            std::vector<block_id_type> act_receipts_digs;
            int j = -1;
            for (size_t i = 0; i < ti->act_receipts.size(); ++i) {
               auto dig = ti->act_receipts[i].digest();
               if (dig == ti->act_receipt_digest) j = i;
               act_receipts_digs.push_back(dig);
            }
            if (j < 0) {
               ilog("This is an invalid transaction due to wrong action receipt");
               continue;
            }
            auto paths = get_proof(j, act_receipts_digs);
            auto merkle_paths = convert_ffi(paths);

            block_id_type_list *ids_list = new block_id_type_list[block_id_lists.size()];
            for (size_t i = 0; i < block_id_lists.size(); ++i) {
               ids_list[i] = convert_ffi(block_id_lists[i]);
            }

            rpc_result *result = prove_action(
               config.bifrost_addr.data(),
               config.bifrost_account.data(),
               &act_ffi,
               &merkle_ptr,
               &receipts,
               &merkle_paths,
               blocks_ffi,
               block_headers.size(),
               ids_list,
               block_id_lists.size()
            );

            if (result) { // not null
               if (result->success) {
                  prove_action_index.modify(ti, [&](auto &entry) {
                     entry.status = 2; // sent successfully
                  });
                  ilog("sent data to bifrost for proving action.");
                  ilog("Transaction got finalized. Hash: ${hash}.", ("hash", std::string(result->msg)));
               } else {
                  ilog("failed to send data to bifrost for proving action due to: ${err}.", ("err", std::string(result->msg)));
               }
            }

            delete []ids_list;
            delete []blocks_ffi;
         }

         prove_action_timer_tick();
      });
   }

   void bridge_plugin_impl::irreversible_block(const chain::block_state_ptr &block) {
      // flush buffer
      uint64_t block_index_max_size = 10240;
      if (prove_action_index.size() >= block_index_max_size && prove_action_index.begin()->status == 2) {
         prove_action_index.erase(prove_action_index.begin());
      }

      if (change_schedule_index.size() >= block_index_max_size && change_schedule_index.begin()->status == 2) {
         change_schedule_index.erase(change_schedule_index.begin());
      }

      ilog("irreversible_block: ${n}, id: ${id}, action_mroot: ${root}", ("n", block->block_num)("id", block->id)("root", block->header.action_mroot));
      auto bb = bridge_blocks{block->id, *block};
      if (block_index.size() >= block_index_max_size) {
         block_index.erase(block_index.begin());
      }
      block_index.insert(bb);

      // collect blocks for prove_action
      for (auto iter = prove_action_index.begin(); iter !=prove_action_index.end(); ++iter) {
         if (iter->status == 0 && iter->bs.size() <= 12 * 16) {
            prove_action_index.modify(iter, [=](auto &entry) {
                entry.bs.push_back(*block);
            });
         }
         if (iter->status != 2 && iter->block_num != 0 && iter->bs.size() >= 12 * 16) {
            prove_action_index.modify(iter, [=](auto &entry) {
                entry.status = 1; // full
            });
         }
      }

      // check if block has new producers, and collect blocks for change_schedule
      auto blk = block->block;
      if (blk->new_producers) {
         // insert blocks
         auto trace = bridge_change_schedule { block->block_num, incremental_merkle(), std::vector<block_state>(), 0};
         change_schedule_index.insert(trace);
      }

      for (auto iter = change_schedule_index.begin(); iter !=change_schedule_index.end(); ++iter) {
         if (iter->status == 0 && iter->bs.size() <= 12 * 16) {
            change_schedule_index.modify(iter, [=](auto &entry) {
               entry.bs.push_back(*block);
            });
         }
         if (iter->status != 2 && iter->block_num != 0 && iter->bs.size() >= 12 * 16) {
            change_schedule_index.modify(iter, [=](auto &entry) {
               entry.status = 1; // full
            });
         }
      }
   }

   void bridge_plugin_impl::filter_action(
      const std::string &contract,
      const std::vector<action_trace> &action_traces,
      const std::vector<action_receipt> &receipts
   ) {
      int index = -1;
      for (size_t i = 0; i < action_traces.size(); ++i) {
         // in case action traces has errors
         if (action_traces[i].except) {
            ilog("An invalid action occured due to: ${reason}", ("reason", action_traces[i].except));
            return;
         }

         auto act = action_traces[i].act;
         auto receiver = action_traces[i].receiver;
         if (act.account == name("eosio.token") && act.name == name("transfer") && receiver == name("eosio.token")) {
            action_transfer der_act;
            fc::raw::unpack<action_transfer>(act.data, der_act);
            ilog("money from: ${from}", ("from", der_act.from));
            ilog("money to: ${to}", ("to", der_act.to));
            ilog("action_transfer: ${to}", ("to", der_act));
            ilog("action traces from: ${to}", ("to", action_traces[i]));

            // deposit operation mean asset will be bridged to bifrost, it needs to verify action.
            // but withdraw operation, do not need to verify action.
            if (der_act.from == contract) return; // withdraw operation, do not need to verify action
            if (!action_traces[i].receipt) return;
            if (der_act.from == name(contract) || der_act.to == name(contract)) {
               index = action_traces[i].action_ordinal;
            }
         }
      }

      if (index < 0) return;

      auto receipt = action_traces[index].receipt;
      auto receipt_dig = action_traces[index].receipt->digest(); // this can be unique as index

      auto bt = bridge_prove_action {
         action_traces[index].block_num,
         action_traces[index].act,
         *action_traces[index].receipt,
         receipts,
         receipt_dig,
         incremental_merkle(),
         std::vector<block_state>(),
         0
      };
      prove_action_index.insert(bt);

      // replace the latest action receipts while more than one action in a single block
      for (auto ti = prove_action_index.begin(); ti != prove_action_index.end(); ++ti) {
         if (ti->block_num == bt.block_num) {
            prove_action_index.modify(ti, [=](auto &entry) {
               entry.act_receipts = receipts;
            });
         }
      }
   }

   void bridge_plugin_impl::apply_action_receipt(std::tuple<const transaction_trace_ptr&, const std::vector<action_receipt>&> t) {
      auto tt = std::get<0>(t);
      auto acts = std::get<1>(t);

      auto action_traces = tt->action_traces;
      filter_action(config.bifrost_account, action_traces, acts);
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
              ("bifrost-node", bpo::value<string>()->default_value("127.0.0.1"),
               "This is sopposed to be a bifrost node address like: 127.0.0.1");
      cfg.add_options()
              ("bifrost-account", bpo::value<string>()->default_value("bob"),
               "This is sopposed to be a bifrost account like: alice or bob");
      cfg.add_options()
              ("delete-relay-history", bpo::bool_switch()->default_value(false),
               "This is sopposed to delete all realy data history");
      ilog("bridge_plugin::set_program_options.");
   }

   void bridge_plugin::plugin_initialize(const variables_map &options) {
      ilog("bridge_plugin::plugin_initialize.");

      try {
         if (options.count("bifrost-node") && options.count("bifrost-account")) {
            // Handle the option
            auto address = options.at("bifrost-node").as<std::string>();
            auto account = options.at("bifrost-account").as<std::string>();
            ilog("address: ${addr}.", ("addr", address));
            ilog("account: ${addr}.", ("addr", account));
            my->config.bifrost_addr = address;
            my->config.bifrost_account = account;
         } else {
            my->config.bifrost_addr = "127.0.0.1:9944";
            my->config.bifrost_account = "bob";
         }

         if (options.at("delete-relay-history").as<bool>()) {
            // Todo, delete relay data
            ilog("delete relay data history. ${h}", ("h", my->datadir));
            boost::filesystem::remove_all(my->datadir);
         }

         my->open_db();

         my->chain_plug = app().find_plugin<chain_plugin>();
         chain::controller &cc = my->chain_plug->chain();
         cc.irreversible_block.connect(boost::bind(&bridge_plugin_impl::irreversible_block, my.get(), _1));
         cc.apply_action_receipt.connect(boost::bind(&bridge_plugin_impl::apply_action_receipt, my.get(), _1));

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
