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
                   ordered_unique<
                           tag<by_id>,
                           member<bridge_blocks,
                                  block_id_type,
                                  &bridge_blocks::id> >
           >
   > bridge_block_index;

    typedef multi_index_container<
            block_header_trace,
            indexed_by<
                    ordered_unique<
                            tag<by_id>,
                            member<block_header_trace,
                                    uint32_t,
                                    &block_header_trace::block_num> >,
                    ordered_non_unique<
                            tag<by_status>,
                            member<block_header_trace,
                                    uint8_t,
                                    &block_header_trace::status> >
            >
    > block_header_trace_index;

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

      unique_ptr<boost::asio::steady_timer> change_schedule_timer;
      unique_ptr<boost::asio::steady_timer> prove_action_timer;

      boost::asio::steady_timer::duration change_schedule_timeout{std::chrono::milliseconds{1000}};
      boost::asio::steady_timer::duration prove_action_timeout{std::chrono::milliseconds{1000}};

      bridge_block_index            block_index;
      bridge_change_schedule_index  change_schedule_index;
      bridge_prove_action_index     prove_action_index;
      block_header_trace_index      trace_index;

      int action_block_num = 0;
      fc::path datadir;

      void change_schedule_timer_tick();
      void prove_action_timer_tick();

      void collect_blocks_timer_tick();

      void irreversible_block(const chain::block_state_ptr &);

      void apply_action_receipt(std::tuple<const transaction_trace_ptr&, const std::vector<action_receipt>&>);

      void open_db();
      void close_db();

      void collect_block_headers_and_ids(const chain::block_state_ptr &, bridge_change_schedule_index::iterator &);
      void collect_block_headers_and_ids(const chain::block_state_ptr &, bridge_prove_action_index::iterator &);

      void init_prove_actions(const block_id_type &, const action &, const action_receipt &, const std::vector<block_id_type> &, const incremental_merkle &, const signed_block_header &);
      std::optional<std::tuple<action, action_receipt, action_trace, std::vector<block_id_type>>> get_index_and_action_proof(const std::vector<action_trace>&, const std::vector<action_receipt> &);

      std::optional<std::tuple<block_id_type, signed_block_header, incremental_merkle>> get_block_id_and_merkle(const action_trace &);
   };

   void bridge_plugin_impl::change_schedule_timer_tick() {
      ilog("bridge_plugin_impl::change_schedule_timer_tick.");
      change_schedule_timer->expires_from_now(change_schedule_timeout);
      change_schedule_timer->async_wait([&](boost::system::error_code ec) {
         auto status_iter = change_schedule_index.get<by_status>().find( 1 );
         auto it = change_schedule_index.project<0>(status_iter);
         for (; it != change_schedule_index.end(); ++it) {
             ilog("sending data to bifrost for proving action.");
//             change_schedule(
//                "127.0.0.1",
//                "bob",
//                &(it->imcre_merkle), it->imcre_merkle._active_nodes.size(),
//                it->block_headers.data(), it->block_headers.size(),
//                it->block_id_lists
//             );
         }

         change_schedule_timer_tick();
      });
   }

   void bridge_plugin_impl::prove_action_timer_tick() {
      prove_action_timer->expires_from_now(prove_action_timeout);
      prove_action_timer->async_wait([&](boost::system::error_code ec) {

          auto ti = trace_index.get<by_status>().lower_bound( 1 );
          auto ti_end = trace_index.get<by_status>().upper_bound( 2 );
          for (auto ti = trace_index.begin(); ti != trace_index.end(); ++ti) {
             if (ti->status != 1) continue;
             auto bl_state = block_state();
             std::vector<signed_block_header> block_headers;
             for (auto bls: ti->bs) {
                if (ti->block_num == bls.block_num) {
                   block_headers.push_back(bls.header);
                   bl_state = bls;
                   break;
                }
             }
             std::vector<std::vector<block_id_type>>   block_id_lists;
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

             signed_block_header_ffi *blocks_ffi = new signed_block_header_ffi[block_headers.size()];
             for (size_t i = 0; i < block_headers.size(); ++i) {
                auto p = new signed_block_header_ffi(block_headers[i]);
                blocks_ffi[i] = *p;
             }
             ilog("converted block headers to string");

             auto receipts = convert_ffi(ti->act_receipt);
             ilog("action receipt got serialized: ${hash}.", ("hash", ti->act_receipt));

             auto act_ffi = convert_ffi(ti->act);

             auto pre_block_state = block_index.find(bl_state.header.previous);
             auto blockroot_merkle = pre_block_state->bls.blockroot_merkle;
             auto merkle_ptr = convert_ffi(blockroot_merkle);

             auto merkle_paths = convert_ffi(ti->act_receipt_merkle_paths);

             // ids list pointers
             block_id_type_list *ids_list = new block_id_type_list[block_id_lists.size()];
             for (size_t i = 0; i < block_id_lists.size(); ++i) {
                if (block_id_lists[i].empty()) {
                   ids_list[i] = block_id_type_list();
                   continue;
                }
                ids_list[i] = convert_ffi(block_id_lists[i]);
             }

             rpc_result *result = prove_action(
                     "127.0.0.1",
                     "bob",
                     &act_ffi,
                     &merkle_ptr,
                     &receipts,
                     &merkle_paths,
                     blocks_ffi,
                     block_headers.size(),
                     ids_list,
                     block_id_lists.size()
             );
             delete []ids_list;
             delete []blocks_ffi;

             if (result) { // not null
                if (result->success) {
                   trace_index.modify(ti, [&](auto &entry) {
                       entry.status = 2; // sent successfully
                   });
                   ilog("sent data to bifrost for proving action.");
                   ilog("Transaction got finalized. Hash: ${hash}.", ("hash", std::string(result->msg)));
                } else {
                   ilog("failed to send data to bifrost for proving action due to: ${err}.", ("err", std::string(result->msg)));
                }
             }
          }

         prove_action_timer_tick();
      });
   }

   void bridge_plugin_impl::collect_block_headers_and_ids(
      const chain::block_state_ptr &block,
      bridge_change_schedule_index::iterator &cs_iter
   ) {
      if (cs_iter->status == 1) return;

      auto current_blocks = cs_iter->block_headers;
      auto header_len = current_blocks.size();
      auto last_block_num = current_blocks.back().block_num();
      // get block header and block ids
      auto block_header = block->header;
      auto current_block_num = block_header.block_num();
      if (current_block_num - last_block_num == 12) {
        current_blocks.push_back(block->header);
        auto empty_ids = std::vector<block_id_type>(); // save a empty ids for current block
        change_schedule_index.modify(cs_iter, [=](auto &entry) {
            entry.block_id_lists.push_back(empty_ids);
        });
      } else {
        auto block_ids = cs_iter->block_id_lists.back();
        if (block_ids.size() == 11) return; // block ids are full
        block_ids.push_back(block->id);
      }

      auto first_block = current_blocks.front();
      if (current_block_num - first_block.block_num() == 12 * 15) {
        change_schedule_index.modify(cs_iter, [=](auto &entry) {
            entry.status = 1; // changing status as 1 means this block is finished proves collecting
        });
      }
   }

   void bridge_plugin_impl::collect_block_headers_and_ids(
      const chain::block_state_ptr &block,
      bridge_prove_action_index::iterator &pa_iter
   ) {
      if (pa_iter->status == 1) {
         ilog("prove action status: ${status}", ("status", pa_iter->status));
         return;
      }

      auto current_blocks = pa_iter->block_headers;
      ilog("headers length: ${header_len}", ("header_len", current_blocks.size()));
      auto last_block_num = current_blocks.back().block_num();
      // get block header and block ids
      auto block_header = block->block;
      auto current_block_num = block_header->block_num();
      ilog("current block num: ${num}", ("num", block->block->block_num()));

      auto first_block = current_blocks.front();
      if (current_block_num - first_block.block_num() == 12 * 15) {
         prove_action_index.modify(pa_iter, [=](auto &entry) {
             ilog("finished collecting.");
             entry.status = 1; // changing status as 1 means this block is finished proves collecting
         });
         return;
      }

      if (current_block_num - last_block_num == 12) {
         ilog("new block need to be collected.");
         auto empty_ids = std::vector<block_id_type>(); // save a empty ids for current block
         ilog("block headers size: ${ids_size}", ("ids_size", current_blocks.size()));

         prove_action_index.modify(pa_iter, [=](auto &entry) {
             entry.block_id_lists.push_back(empty_ids);
             entry.block_headers.push_back(block->header);
             ilog("pushed a block header: ${header}", ("header", block->header));
         });
      } else {
         ilog("new ids need to be collected.");
         auto block_ids = pa_iter->block_id_lists.back();
         ilog("block ids size: ${ids_size}", ("ids_size", block_ids.size()));
         if (block_ids.size() >= 10) return; // block ids are full
         prove_action_index.modify(pa_iter, [=](auto &entry) {
             entry.block_id_lists.back().push_back(block->id);
         });
      }
   }

   void bridge_plugin_impl::irreversible_block(const chain::block_state_ptr &block) {
      // TODO read blocks info to local storage
      ilog("irreversible_block: ${n}, id: ${id}, action_mroot: ${root}", ("n", block->block_num)("id", block->id)("root", block->header.action_mroot));
      auto bb = bridge_blocks{block->id, *block};

      for (auto iter = trace_index.begin(); iter !=trace_index.end(); ++iter) {
         if (iter->status == 0 && iter->bs.size() <= 12 * 16) {
            trace_index.modify(iter, [=](auto &entry) {
                entry.bs.push_back(*block);
            });
         }
         if (iter->status != 2 && iter->block_num != 0 && iter->bs.size() >= 12 * 16) {
            trace_index.modify(iter, [=](auto &entry) {
                entry.status = 1; // full
            });
         }
      }

      // record block
      uint64_t block_index_max_size = 1024;
      if (block_index.size() >= block_index_max_size) {
         block_index.erase(block_index.begin());
      }
      block_index.insert(bb);

      // check if block has new producers
      auto blk = block->block;
      if (blk->new_producers) {
         // get the merkle root from current block of previous one
         auto pre_block_state = block_index.find(blk->previous);
         auto before_previous = block_index.find(pre_block_state->bls.block->previous);
         auto blockroot_merkle = before_previous->bls.blockroot_merkle;

         auto cs = bridge_change_schedule{
            block->id,
            blockroot_merkle,
            std::vector<signed_block_header>(),
            std::vector<std::vector<block_id_type>>(),
            0,
         };
         // get block header and block ids
         auto block_header = block->header;

         cs.block_headers.push_back(block_header);
         auto empty_ids = std::vector<block_id_type>(); // if schedule changes, save a empty ids for current block
         cs.block_id_lists.push_back(empty_ids);

         change_schedule_index.insert(cs);
      }

      auto cs_status_iter = change_schedule_index.get<by_status>().find(0);
      auto cs_it = change_schedule_index.project<0>(cs_status_iter);
      for (; cs_it != change_schedule_index.end(); ++cs_it) {
//         collect_block_headers_and_ids(block, cs_it);
      }

      auto pa_status_iter = prove_action_index.get<by_status>().find(0);
      auto pa_it = prove_action_index.project<0>(pa_status_iter);
      for (; pa_it != prove_action_index.end(); ++pa_it) {
         collect_block_headers_and_ids(block, pa_it);
      }
   }

   void bridge_plugin_impl::init_prove_actions(
      const block_id_type &id,
      const action &act,
      const action_receipt &receipt,
      const std::vector<block_id_type> &action_merkle_paths,
      const incremental_merkle &merkle,
      const signed_block_header &block_header
    ) {
      auto pa = bridge_prove_action{
         id,
         act,
         receipt,
         action_merkle_paths,
         merkle,
         std::vector<signed_block_header>(),
         std::vector<std::vector<block_id_type>>(),
         0,
      };
//      ilog("prove action: ${prove}", ("prove", pa));
      pa.block_headers.push_back(block_header);
      pa.block_id_lists.push_back(std::vector<block_id_type>());
      pa.block_id_lists.push_back(std::vector<block_id_type>());
      prove_action_index.insert(pa);
      ilog("prove action len: ${len}", ("len", prove_action_index.size()));
   }

   std::optional<std::tuple<block_id_type, signed_block_header, incremental_merkle>> bridge_plugin_impl::get_block_id_and_merkle(const action_trace& trace) {
      auto block_num = trace.block_num - 2; // we need to find a block like ,if current block num is 9313, find 9311

      auto blk_index_it = block_index.begin();
      for (; blk_index_it != block_index.end(); ++blk_index_it) {
         if (blk_index_it->bls.block->block_num() == block_num) {
            // get block id
            auto blk = blk_index_it->bls.block;
            auto block_id = blk->id();

            if(blk_index_it == block_index.begin()) return std::optional<std::tuple<block_id_type, signed_block_header, incremental_merkle>>{};
            blk_index_it--;
            if(blk_index_it == block_index.begin()) return std::optional<std::tuple<block_id_type, signed_block_header, incremental_merkle>>{};
            blk_index_it--;

            auto block_header = blk_index_it->bls.header;
            auto blockroot_merkle = blk_index_it->bls.blockroot_merkle;
            auto id_and_merkle = std::make_tuple(block_id, block_header, blockroot_merkle);

            return std::optional<std::tuple<block_id_type, signed_block_header, incremental_merkle>>{id_and_merkle};
         };
      }

      return std::optional<std::tuple<block_id_type, signed_block_header, incremental_merkle>>{};
   }

   std::optional<std::tuple<action, action_receipt, action_trace, std::vector<block_id_type>>> bridge_plugin_impl::get_index_and_action_proof(
      const std::vector<action_trace> &action_traces,
      const std::vector<action_receipt> &receipts
   ) {
      int index = -1;
      std::vector<block_id_type> act_receipts_digs;
      for (size_t i = 0; i < action_traces.size(); ++i) {
         auto act = action_traces[i].act;
         auto receiver = action_traces[i].receiver;
         if (act.account == name("eosio.token") && act.name == name("transfer") && receiver == name("eosio.token")) {
            action_transfer der_at;
            fc::raw::unpack<action_transfer>(act.data, der_at);
            ilog("money from: ${from}", ("from", der_at.from));
            ilog("money from: ${to}", ("to", der_at.to));
            ilog("action traces from: ${to}", ("to", action_traces));

            if (!action_traces[i].receipt) {
               return std::optional<std::tuple<action, action_receipt, action_trace, std::vector<block_id_type>>>{};
            }
            if (der_at.from == name("jim") || der_at.to == name("alex")) index = action_traces[i].action_ordinal;
         }
      }

      if (index < 0) return std::optional<std::tuple<action, action_receipt, action_trace, std::vector<block_id_type>>>{};
      for (size_t i = 0; i < receipts.size(); ++i) {
         ilog("block num from action trace: ${num}", ("num", action_traces[i].block_num));
         act_receipts_digs.push_back(receipts[i].digest());
      }
      auto act = action_traces[index].act;
      auto receipt = action_traces[index].receipt;
      auto trace = action_traces[index];

      for (size_t i = 0; i < act_receipts_digs.size(); ++i) {
         if (act_receipts_digs[i] == receipt->digest()) index = i;
      }

      auto action_merkle_paths = get_proof(index, act_receipts_digs);
      auto index_and_merkle_path = std::make_tuple(act, *receipt, trace, action_merkle_paths);

      auto bt = block_header_trace {
        action_traces[index].block_num,
        act,
        *receipt,
        action_merkle_paths,
        std::vector<block_state>(),
        0
      };
      trace_index.insert(bt);

      return std::optional<std::tuple<action, action_receipt, action_trace, std::vector<block_id_type>>>{index_and_merkle_path};
   }

   void bridge_plugin_impl::apply_action_receipt(std::tuple<const transaction_trace_ptr&, const std::vector<action_receipt>&> t) {
      auto tt = std::get<0>(t);
      auto acts = std::get<1>(t);

      auto action_traces = tt->action_traces;
      auto index_and_merkle_path = get_index_and_action_proof(action_traces, acts);
      if (!index_and_merkle_path.has_value()) {
         ilog("not found index and merkle path.");
         return;
      }

      auto deref = index_and_merkle_path.value();
      auto act = std::get<0>(deref); // get action
      auto receipt = std::get<1>(deref); // get action receipt
      auto trace = std::get<2>(deref); // get action trace
      auto action_merkle_paths = std::get<3>(deref); // get merkle path

      auto id_and_merkle = get_block_id_and_merkle(trace);
      ilog("has value: ${r}", ("r", id_and_merkle.has_value()));
      if (!id_and_merkle.has_value()) return;

      auto id = std::get<0>(*id_and_merkle);
      auto block_header = std::get<1>(*id_and_merkle);
      auto merkle = std::get<2>(*id_and_merkle);

      init_prove_actions(id, act, receipt, action_merkle_paths, merkle, block_header);
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
//         cc.applied_transaction.connect(boost::bind(&bridge_plugin_impl::applied_transaction, my.get(), _1));
         cc.apply_action_receipt.connect(boost::bind(&bridge_plugin_impl::apply_action_receipt, my.get(), _1));
//         cc.pre_apply_transaction.connect(boost::bind(&bridge_plugin_impl::pre_apply_transaction, my.get(), _1));

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
