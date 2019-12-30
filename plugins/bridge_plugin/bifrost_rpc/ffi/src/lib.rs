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
    mem,
    ptr,
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

#[no_mangle]
pub extern "C" fn prove_action(
    url: *const c_char,
    signer: *const c_char,
    act_ffi: *const ActionFFI,
    imcre_merkle: *const IncrementalMerkleFFI,
    receipt_json: *const c_char,
    action_merkle_paths: *const Checksum256FFI,
    blocks_json: *const c_char,
    ids_list: *const Checksum256FFI,
    ids_list_size: usize
) -> *const RpcResponse {
    let ids: Vec<Checksum256> = Vec::with_capacity(10);
    let mut ids_lists: Vec<Vec<Checksum256>>= vec![ids; 15];
    let ids_list_ffi = unsafe { slice::from_raw_parts(ids_list, ids_list_size) };
    for (i, val) in ids_list_ffi.iter().enumerate() {
        ids_lists[i] = val.clone().into();
    }

    let merkle: IncrementalMerkle = {
        let imcre_merkle = unsafe { ptr::read(imcre_merkle) };
        imcre_merkle.into()
    };

    let action: Action = {
        let ffi = unsafe { ptr::read(act_ffi) };
        ffi.into()
    };

    let block_headers = {
        let blocks = unsafe {
            cstr_to_string(blocks_json)
        };
        match blocks {
            Ok(ref blocks) => {
                let block_headers: Result<Vec<SignedBlockHeader>, _> = serde_json::from_str(blocks);
                if block_headers.is_err() {
                    let err = generate_result(false, "failed to deserialize SignedBlockHeader.");
                    let box_err = Box::new(err);
                    return Box::into_raw(box_err);
                }
                block_headers.unwrap()
            }
            Err(e) => {
                let err = generate_result(false, e.to_string());
                let box_err = Box::new(err);
                return Box::into_raw(box_err);
            },
        }
    };

    let action_receipt = unsafe {
        cstr_to_string(receipt_json).expect("parse block header with failure.")
    };
    let action_receipt: Result<ActionReceipt, _> = serde_json::from_str(&action_receipt);
    let action_receipt = action_receipt.unwrap();

    let action_merkle_paths: Vec<_> = {
        let paths = unsafe { ptr::read(action_merkle_paths) };
        paths.into()
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
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);

    let result = generate_result(true, tx_hash.to_string());
    let box_result = Box::new(result);
    Box::into_raw(box_result)
}
