// Copyright 2019 Liebi Technologies.
// This file is part of Bifrost.

// Bifrost is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Bifrost is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Bifrost.  If not, see <http://www.gnu.org/licenses/>.

use eos_chain::{Action, ActionReceipt, Checksum256, Digest, IncrementalMerkle, SignedBlockHeader};
use log::info;
use keyring::AccountKeyring;
use primitives::crypto::Pair;
use rpc_client::{
    Api,
    compose_call, compose_extrinsic,
    extrinsic::xt_primitives::UncheckedExtrinsicV4,
};
use std::{
    os::raw::c_char,
    slice,
};

use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

mod ffi_types;
use ffi_types::*;

#[no_mangle]
pub extern "C" fn change_schedule(
    url: *const c_char,
    signer: *const c_char,
    merkle: *const IncrementalMerkleFFI,
    merkle_checksum_len: size_t,
    block_headers: *const SignedBlockHeaderFFI, // vec<>
    block_headers_len: size_t,
    block_ids_list: *const *const Checksum256 // vec<vec<>>
) {
    // check pointers null or not
    // Todo, find a more elegant way to check these pointers null or not
    match (url.is_null(), signer.is_null(), merkle.is_null(), block_headers.is_null(), block_ids_list.is_null()) {
        (true, true, true, true, true) => {
            info!("all are valid pointers.");
        }
        _ => {
            return;
        }
    }
    let url = unsafe {
        cstr_to_string(url).expect("failed to convert cstring to rust string.") // Todo, remove expect
    };
    let signer = AccountKeyring::Alice.pair();
    let api = Api::new(format!("ws://{}", url)).set_signer(signer.clone());

    let merkle = unsafe {
        (*merkle).clone().into_incrementl_merkle(merkle_checksum_len)
    };

    let block_headers = unsafe {
        let ffi = slice::from_raw_parts(block_headers, block_headers_len);
        ffi.into_iter().map(|f| f.into_signed_block_header()).collect::<Vec<_>>()
    };

    let block_ids_list = unsafe {
        let ffi = slice::from_raw_parts(block_ids_list, 15).to_vec();
        ffi.into_iter().map(|f|{
//            slice::from_raw_parts(f, 11).iter().map(|c| ptr::read(c)).collect::<Vec<Checksum256>>()
            slice::from_raw_parts(f, 10).to_vec()
        }).collect::<Vec<_>>()
    };
//    let block_ids_list: Vec<Vec<Checksum256>> = Vec::new();

    let proposal = compose_call!(
        api.metadata.clone(),
        "BridgeEos",
        "change_schedule",
        merkle,
        block_headers,
        block_ids_list
    );

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Sudo",
        "sudo",
        proposal
    );

    // Unable to decode Vec on index 2 createType(ExtrinsicV4):: Source is too large
    println!("[+] Composed extrinsic: {:?}\n", xt);
    // send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
}

fn create_file(s: &str) -> std::io::Result<()> {
    let file_name = format!("{}.txt", s);
    let mut file = std::fs::File::create(file_name)?;
    file.write_all(b"Hello, world!")?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn prove_action(
    url: *const c_char,
    signer: *const c_char,
    action_json: *const c_char,
    receipt_json: *const c_char,
    action_merkle_paths: *const Checksum256,
    action_merkle_paths_len: size_t,
    active_nodes: *const Checksum256,
    active_node_size: size_t,
    node_count: u64,
    blocks_json: *const c_char,
    ids_json: *const c_char
) -> bool {
    let blocks = unsafe {
        cstr_to_string(blocks_json).expect("parse block header with failure.")
    };
    let block_headers1: Result<Vec<SignedBlockHeader>, _> = serde_json::from_str(&blocks);
    let block_headers1 = block_headers1.unwrap();

    let ids_list = unsafe {
        cstr_to_string(ids_json).expect("parse block header with failure.")
    };
    dbg!(&ids_list);
    let ids_list: Result<Vec<Vec<Checksum256>>, _> = serde_json::from_str(&ids_list);
    dbg!(&ids_list);
    let ids_list = ids_list.unwrap();

    match (
        url.is_null(), signer.is_null(), action_json.is_null(), receipt_json.is_null(), action_merkle_paths.is_null(),
        active_nodes.is_null(), blocks_json.is_null()//, block_ids_list.is_null()
    ) {
        (false, false, false, false, false, false, false) => {
            // normal code
            info!("all are valid pointers.");
        }
        _ => {
            info!("all of them are not null pointers");
            return false;
        }
    }
    let action = unsafe {
        cstr_to_string(action_json).expect("parse block header with failure.")
    };
    dbg!(&action);
    let action: Result<Action, _> = serde_json::from_str(&action);
    dbg!(&action);
    let action = action.unwrap();

    let action_receipt = unsafe {
        cstr_to_string(receipt_json).expect("parse block header with failure.")
    };
    let action_receipt: Result<ActionReceipt, _> = serde_json::from_str(&action_receipt);
    let action_receipt = action_receipt.unwrap();

    let action_merkle_paths = unsafe {
        slice::from_raw_parts(action_merkle_paths, action_merkle_paths_len).to_vec()
    };

    let merkle = unsafe {
        let active_nodes = slice::from_raw_parts(active_nodes, active_node_size).to_vec();
        for i in &active_nodes {
            dbg!(&i.to_string());
        }
        IncrementalMerkle::new(node_count, active_nodes)
    };

    let url = unsafe {
        cstr_to_string(url).expect("failed to convert cstring to rust string.")
    };
    let signer = AccountKeyring::Alice.pair();
    let api = Api::new(format!("ws://{}:9944", url)).set_signer(signer.clone());

    let proposal = compose_call!(
        api.metadata.clone(),
        "BridgeEos",
        "prove_action",
        action,
        action_receipt,
        action_merkle_paths,
        merkle,
        block_headers1,
        ids_list
    );

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Sudo",
        "sudo",
        proposal
    );

    println!("[+] Composed extrinsic: {:?}\n", xt);
    // send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);

    true
}
