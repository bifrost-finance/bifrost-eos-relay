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

use eos_chain::{
    Action, AccountName, ActionName, ActionReceipt, PermissionLevel, Checksum256,
    Signature, BlockHeader, Extension, utils::flat_map::FlatMap, UnsignedInt, PublicKey,
    ProducerKey, BlockTimestamp, ProducerSchedule, IncrementalMerkle, SignedBlockHeader
};
use std::os::raw::{c_char, c_uint, c_ushort, c_ulonglong};
use std::{slice, ffi::{CStr, CString}, ptr};
use std::borrow::Cow;
use std::str::FromStr;
use std::mem;

#[allow(non_camel_case_types)]
pub(crate) type size_t = usize;
pub(crate) type FFIResult<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send + 'static>>;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ActionFFI {
    pub account: AccountName,
    pub name: ActionName,
    pub authorization: *const PermissionLevel,
    pub authorization_size: usize,
    pub data: *const c_char,
    pub data_size: usize
}

impl ActionFFI {
    pub(crate) unsafe fn into_action(&self) -> FFIResult<Action> {
        let account = self.account;
        let name = self.name;
        let authorization = slice::from_raw_parts(self.authorization, self.authorization_size).to_vec();
        let data = slice::from_raw_parts(self.data, self.data_size).iter().map(|c| *c as u8).collect::<Vec<_>>();
        Ok(Action {
            account,
            name,
            authorization,
            data
        })
    }
}

impl Into<Action> for ActionFFI {
    fn into(self) -> Action {
        let account = self.account;
        let name = self.name;
        let (authorization, data) = unsafe {
            (
                slice::from_raw_parts(self.authorization, self.authorization_size).to_vec(),
                slice::from_raw_parts(self.data, self.data_size).iter().map(|c| *c as u8).collect::<Vec<_>>()
            )
        };
        Action { account, name, authorization, data }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Checksum256FFI {
    pub id: *const Checksum256,
    pub ids_size: usize,
}

impl Into<Vec<Checksum256>> for Checksum256FFI {
    fn into(self) -> Vec<Checksum256> {
        match self.ids_size.eq(&0) || self.id.is_null() {
            false => unsafe { slice::from_raw_parts(self.id, self.ids_size).to_vec() },
            true => vec![] // if id is null or size equals to 0
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ActionReceiptFFI {
    pub receiver: AccountName,
    pub act_digest: Checksum256,
    pub global_sequence: c_ulonglong,
    pub recv_sequence: c_ulonglong,
    pub auth_sequence: *const (AccountName, c_ulonglong),
    pub auth_sequence_size: usize,
    pub code_sequence: UnsignedInt,
    pub abi_sequence: UnsignedInt,
}

impl Into<ActionReceipt> for ActionReceiptFFI {
    fn into(self) -> ActionReceipt {
        let receiver = self.receiver;
        let act_digest = self.act_digest;
        let global_sequence = self.global_sequence;
        let recv_sequence = self.recv_sequence;
        let code_sequence = self.code_sequence;
        let abi_sequence = self.abi_sequence;
        let auth_sequence = {
            let maps = unsafe { slice::from_raw_parts(self.auth_sequence, self.auth_sequence_size).to_vec() };
            FlatMap::assign(maps)
        };

        ActionReceipt {
            receiver,
            act_digest,
            global_sequence,
            recv_sequence,
            auth_sequence,
            code_sequence,
            abi_sequence
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct FlatMapFFI<K, V> { // Todo
    maps: *const (K, V),
}

impl<K, V> FlatMapFFI<K, V> where K: Clone, V: Clone {
    pub unsafe fn into_flat_map(&self, map_len: usize) -> FlatMap<K, V> {
        let maps = slice::from_raw_parts(self.maps, map_len).to_vec();
        FlatMap::assign(maps)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct IncrementalMerkleFFI {
    pub _node_count: c_ulonglong,
    pub _active_nodes: *const Checksum256,
    pub _active_nodes_size: usize,
}

impl Into<IncrementalMerkle> for IncrementalMerkleFFI {
    fn into(self) -> IncrementalMerkle {
        if self._active_nodes.is_null() {
            IncrementalMerkle::new(0, vec![]);
        }
        let _active_nodes = unsafe {
            slice::from_raw_parts(self._active_nodes, self._active_nodes_size).to_vec()
        };

        IncrementalMerkle::new(self._node_count, _active_nodes)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ExtensionFFI {
    pub _type: c_ushort,
    pub data: *const c_char,
    pub data_size: usize,
}

impl Into<Extension> for ExtensionFFI {
    fn into(self) -> Extension {
        if self.data.is_null() {
            return Extension(0, vec![]);
        }

        let ext = unsafe { slice::from_raw_parts(self.data, self.data_size).iter().map(|c| *c as u8 ).collect::<Vec<u8>>() };

        Extension(self._type, ext)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ExtensionsFFI {
    pub extensions: *const ExtensionFFI,
    pub extensions_size: usize,
}

impl Into<Vec<Extension>> for ExtensionsFFI {
    fn into(self) -> Vec<Extension> {
        if self.extensions.is_null() {
            return vec![];
        }
        let exts = unsafe { slice::from_raw_parts(self.extensions, self.extensions_size) };
        let mut extensions: Vec<_> = Vec::with_capacity(exts.len());
        for (i, v) in exts.iter().enumerate() {
            extensions[i] = v.clone().into();
        }
        extensions
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PublicKeyFFI {
    pub type_: UnsignedInt,
    pub data: *const c_char, // constant array, length is 33
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ProducerKeyFFI {
    pub producer_name: AccountName,
    pub block_signing_key: *const c_char,
}

impl Into<ProducerKey> for ProducerKeyFFI {
    fn into(self) -> ProducerKey {
        let producer_name = self.producer_name;
        let key_str = unsafe { char_to_string(self.block_signing_key).expect("failed to convert c str to rust string") };
        let block_signing_key = PublicKey::from_str(&key_str).expect("failed to get public key from string");

        ProducerKey { producer_name, block_signing_key }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ProducerScheduleFFI {
    pub version: c_uint,
    pub producers: *const ProducerKeyFFI,
    pub producers_size: usize,
}

impl Into<ProducerSchedule> for ProducerScheduleFFI {
    fn into(self) -> ProducerSchedule {
        let producers_ffi = unsafe { slice::from_raw_parts(self.producers, self.producers_size) };
        let mut producers: Vec<ProducerKey> = Vec::with_capacity(producers_ffi.len());
        for (i, p) in producers_ffi.iter().enumerate() {
            producers[i] = p.clone().into();
        }

        ProducerSchedule {
            version: self.version,
            producers
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct BlockHeaderFFI {
    pub timestamp: BlockTimestamp,
    pub producer: AccountName,
    pub confirmed: c_ushort,
    pub previous: *const c_char,
    pub transaction_mroot: *const c_char,
    pub action_mroot: *const c_char,
    pub schedule_version: c_uint,
    pub new_producers: *const ProducerScheduleFFI,
    pub header_extensions: *const ExtensionsFFI,
}

impl Into<BlockHeader> for BlockHeaderFFI {
    fn into(self) -> BlockHeader {
        let new_producers: Option<ProducerSchedule> = {
            if self.new_producers.is_null() {
                None
            } else {
                let ffi = unsafe { ptr::read(self.new_producers) };
                Some(ffi.into())
            }
        };
        let header_extensions: Vec<Extension> = {
            if self.header_extensions.is_null() {
                vec![]
            } else {
                let ffi = unsafe { ptr::read(self.header_extensions) };
                ffi.into()
            }
        };
        let (previous, transaction_mroot, action_mroot) = {
            let s = unsafe { slice::from_raw_parts(self.previous, 32) };
            let s1 = unsafe { slice::from_raw_parts(self.transaction_mroot, 32) };
            let s2 = unsafe { slice::from_raw_parts(self.action_mroot, 32) };
            let (previous, transaction_mroot, action_mroot) = {
                let mut a: [u8;32] = [0u8;32];
                let mut a1: [u8;32] = [0u8;32];
                let mut a2: [u8;32] = [0u8;32];
                for (i, v) in s.iter().enumerate() {
                    a[i] = *v as u8;
                }
                let mut b = [0i8;32];
                let t = b.copy_from_slice(s);

                for (i, v) in s1.iter().enumerate() {
                    a1[i] = *v as u8;
                }

                for (i, v) in s2.iter().enumerate() {
                    a2[i] = *v as u8;
                }
                (Checksum256::from(a), Checksum256::from(a1), Checksum256::from(a2))
            };

            (previous, transaction_mroot, action_mroot)
        };
        BlockHeader {
            timestamp: self.timestamp,
            producer: self.producer,
            confirmed: self.confirmed,
            previous,
            transaction_mroot,
            action_mroot,
            schedule_version: self.schedule_version,
            new_producers,
            header_extensions
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SignedBlockHeaderFFI {
    pub block_header: *const BlockHeaderFFI,
    pub producer_signature: *const c_char,
}

impl Into<SignedBlockHeader> for SignedBlockHeaderFFI {
    fn into(self) -> SignedBlockHeader {
        let ps_str = unsafe { char_to_string(self.producer_signature).expect("failed to convert producer_signature to rust string.") };
        dbg!(&ps_str);
        let producer_signature = Signature::from_str(&ps_str).expect("failed to get Signature");

        let block_header: BlockHeader = {
            let ffi = unsafe { ptr::read(self.block_header) };
            ffi.into()
        };

        SignedBlockHeader {
            block_header,
            producer_signature
        }
    }
}

pub(crate) unsafe fn char_to_string(cstr: *const c_char) -> FFIResult<String> {
    let cstr = CStr::from_ptr(cstr);
    let rust_string = cstr.to_str()?.to_string();
    Ok(rust_string)
}

pub(crate) unsafe fn char_to_cstr(cstr: *const c_char) -> FFIResult<String> {
    let cstr = CStr::from_ptr(cstr);
    let rust_string = cstr.to_str()?.to_string();
    Ok(rust_string)
}

pub(crate) fn generate_result(success: bool, msg: impl AsRef<str>) -> RpcResponse {
    let c_str = CString::new(msg.as_ref())
                .unwrap_or(
                    CString::new("unknow error type.").expect("failed to get raw pointer of error message")
                );
    RpcResponse {
        success,
        msg: c_str.into_raw()
    }
}

// this struct will return to c++ caller
#[derive(Clone, Debug)]
#[repr(C)]
pub struct RpcResponse {
    success: bool,
    msg: *const c_char, // this could be error message or successful message
}
