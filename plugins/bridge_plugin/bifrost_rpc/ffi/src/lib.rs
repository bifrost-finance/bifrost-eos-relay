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

use eos_chain::{Action, ActionReceipt, Checksum256, IncrementalMerkle, SignedBlockHeader};
use log::info;
use rpc_client::{
    Api,
    compose_call, compose_extrinsic,
    extrinsic::xt_primitives::UncheckedExtrinsicV4,
    keyring::AccountKeyring,
};
use std::{
    convert::TryInto,
    error::Error as _,
    os::raw::c_char,
    ptr,
    slice,
};

mod ffi_types;
use ffi_types::*;

#[no_mangle]
pub extern "C" fn change_schedule(
    url:             *const c_char,
    signer:          *const c_char,
    imcre_merkle:    *const IncrementalMerkleFFI,
    blocks_ffi:      *const SignedBlockHeaderFFI,
    blocks_ffi_size: size_t,
    ids_list:        *const Checksum256FFI,
    ids_list_size:   size_t
) -> *const RpcResponse {
    // check pointers null or not
    match (url.is_null(), signer.is_null(), imcre_merkle.is_null(), blocks_ffi.is_null(), ids_list.is_null()) {
        (false, false, false, false, false) => (),
        _ => {
            return generate_raw_result(false, "cannot send action to bifrost node to prove it due to there're null points");
        }
    }

    let url = {
        let url = char_to_string(url);
        if url.is_err() {
            return generate_raw_result(false, "This is not an valid bifrost node address.");
        }
        url.unwrap()
    };
    let signer = AccountKeyring::Alice.pair();
    let api = Api::new(format!("ws://{}", url)).set_signer(signer.clone());

    let merkle: IncrementalMerkle = {
        let imcre_merkle = unsafe { ptr::read(imcre_merkle) };
        let r: Result<IncrementalMerkle, _> = imcre_merkle.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().description());
        }
        r.unwrap()
    };

    let block_headers: Vec<SignedBlockHeader> = {
        let blocks_ffi = unsafe { slice::from_raw_parts(blocks_ffi, blocks_ffi_size) };
        let mut block_headers: Vec<_> = Vec::with_capacity(blocks_ffi_size);
        for block in blocks_ffi.iter() {
            let ffi = unsafe { ptr::read(block) };
            let r: Result<SignedBlockHeader, Error> = ffi.try_into();
            if r.is_err() {
                return generate_raw_result(false, r.unwrap_err().description());
            }
            block_headers.push(r.unwrap())
        }
        block_headers
    };

    let ids: Vec<Checksum256> = Vec::with_capacity(10);
    let mut ids_lists: Vec<Vec<Checksum256>>= vec![ids; 15];
    let ids_list_ffi = unsafe { slice::from_raw_parts(ids_list, ids_list_size) };
    for ids in ids_list_ffi.iter() {
        let r: Result<Vec<Checksum256>, _> = ids.clone().try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().description());
        }
        ids_lists.push(r.unwrap())
    }

    let proposal = compose_call!(
        api.metadata.clone(),
        "BridgeEos",
        "change_schedule",
        merkle,
        block_headers,
        ids_lists
    );

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Sudo",
        "sudo",
        proposal
    );

    println!("[+] Composed extrinsic: {:?}\n", xt);
    // send and watch extrinsic until finalized
    match api.send_extrinsic(xt.hex_encode()) {
        Ok(tx_hash) => {
            println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
            generate_raw_result(true, tx_hash.to_string())
        }
        Err(e) => {
            println!("[+] Transaction got failure due to: {:?}\n", e);
            generate_raw_result(true, e.to_string())
        }
    }
}

#[no_mangle]
pub extern "C" fn prove_action(
    url:                 *const c_char,
    signer:              *const c_char,
    act_ffi:             *const ActionFFI,
    imcre_merkle:        *const IncrementalMerkleFFI,
    act_receipt:         *const ActionReceiptFFI,
    action_merkle_paths: *const Checksum256FFI,
    blocks_ffi:          *const SignedBlockHeaderFFI,
    blocks_ffi_size:     size_t,
    ids_list:            *const Checksum256FFI,
    ids_list_size:       size_t
) -> *const RpcResponse {
    match (
        url.is_null(), signer.is_null(), act_ffi.is_null(), imcre_merkle.is_null(),
        act_receipt.is_null(), action_merkle_paths.is_null(), blocks_ffi.is_null(), ids_list.is_null()
    ) {
        (false, false, false, false, false, false, false, false) => (),
        _ => { // if there's any null pointer, just return
            return generate_raw_result(false, "cannot send action to bifrost node to prove it due to there're null points");
        }
    }

    let action: Action = {
        let ffi = unsafe { ptr::read(act_ffi) };
        let r: Result<Action, _> = ffi.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().description());
        }
        r.unwrap()
    };

    let merkle: IncrementalMerkle = {
        let imcre_merkle = unsafe { ptr::read(imcre_merkle) };
        let r: Result<IncrementalMerkle, _> = imcre_merkle.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().description());
        }
        r.unwrap()
    };

    let action_receipt: ActionReceipt = {
        let act_ffi = unsafe { ptr::read(act_receipt) };
        let r: Result<ActionReceipt, _> = act_ffi.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().description());
        }
        r.unwrap()
    };

    let action_merkle_paths: Vec<Checksum256> = {
        let paths = unsafe { ptr::read(action_merkle_paths) };
        let r: Result<Vec<Checksum256>, _> = paths.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().description());
        }
        r.unwrap()
    };

    let block_headers: Vec<SignedBlockHeader> = {
        let blocks_ffi = unsafe { slice::from_raw_parts(blocks_ffi, blocks_ffi_size) };
        let mut block_headers: Vec<_> = Vec::with_capacity(blocks_ffi_size);
        for block in blocks_ffi.iter() {
            let ffi = unsafe { ptr::read(block) };
            let r: Result<SignedBlockHeader, Error> = ffi.try_into();
            if r.is_err() {
                return generate_raw_result(false, r.unwrap_err().description());
            }
            block_headers.push(r.unwrap());
        }
        block_headers
    };

    let mut ids_lists: Vec<Vec<Checksum256>>= Vec::with_capacity(15);
    let ids_list_ffi = unsafe { slice::from_raw_parts(ids_list, ids_list_size) };
    for ids in ids_list_ffi.iter() {
        let r: Result<Vec<Checksum256>, _> = ids.clone().try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().description());
        }
        ids_lists.push(r.unwrap());
    }

    let url = {
        let url = char_to_string(url);
        if url.is_err() {
            return generate_raw_result(false, "This is not an valid bifrost node address.");
        }
        url.unwrap()
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
        block_headers,
        ids_lists
    );

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Sudo",
        "sudo",
        proposal
    );

    println!("[+] Composed extrinsic: {:?}\n", xt);
    // send and watch extrinsic until finalized
    match api.send_extrinsic(xt.hex_encode()) {
        Ok(tx_hash) => {
            println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
            generate_raw_result(true, tx_hash.to_string())
        }
        Err(e) => {
            println!("[+] Transaction got failure due to: {:?}\n", e);
            generate_raw_result(true, e.to_string())
        }
    }
}
