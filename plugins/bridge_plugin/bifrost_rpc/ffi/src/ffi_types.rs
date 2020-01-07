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
use std::{
    convert::TryInto,
    ffi::{CStr, CString},
    fmt::{self, Display},
    marker::PhantomData,
    mem,
    ops::Deref,
    os::raw::{c_char, c_uint, c_ushort, c_ulonglong},
    ptr,
    str::FromStr,
    slice,
};

#[allow(non_camel_case_types)]
pub(crate) type size_t = usize;
pub(crate) type FFIResult<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug)]
pub enum Error {
    NullPtr,
    CStrConvertError,
    PublicKeyError,
    SignatureError,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NullPtr => write!(f, "Null pointer."),
            Self::CStrConvertError => write!(f, "Failed to convert c string to rust string."),
            Self::PublicKeyError => write!(f, "Failed to convert string to PublicKey."),
            Self::SignatureError => write!(f, "Failed to convert string to Signature."),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Self::NullPtr => "Null pointer.",
            Self::CStrConvertError => "Failed to convert c string to rust string",
            Self::PublicKeyError => "Failed to convert string to PublicKeyError.",
            Self::SignatureError => "Failed to convert string to Signature.",
        }
    }
}

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

impl TryInto<Action> for ActionFFI {
    type Error = Error;
    fn try_into(self) -> Result<Action, Self::Error> {
        if self.authorization.is_null() || self.data.is_null() {
            Err(Error::NullPtr)
        } else {
            let account = self.account;
            let name = self.name;
            let (authorization, data) = unsafe {
                (
                    slice::from_raw_parts(self.authorization, self.authorization_size).to_vec(),
                    slice::from_raw_parts(self.data, self.data_size).iter().map(|c| *c as u8).collect::<Vec<_>>()
                )
            };

            Ok(Action { account, name, authorization, data })
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Checksum256FFI {
    pub data: *const c_char,
    pub data_size: usize, // basically, it should be 32
}

impl TryInto<Checksum256> for Checksum256FFI {
    type Error = Error;
    fn try_into(self) -> Result<Checksum256, Self::Error> {
        if self.data.is_null() {
            Err(Error::NullPtr)
        } else {
            let data: [u8; 32] = {
                let slice = unsafe { slice::from_raw_parts(self.data, self.data_size) };
                unsafe { mem::transmute_copy(&slice[0]) }
            };

            Ok(Checksum256::from(data))
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Checksum256ListFFI {
    pub ids: *const Checksum256FFI,
    pub ids_size: usize,
}

impl TryInto<Vec<Checksum256>> for Checksum256ListFFI {
    type Error = Error;
    fn try_into(self) -> Result<Vec<Checksum256>, Self::Error> {
        if self.ids.is_null() {
            Err(Error::NullPtr)
        } else {
            let ids_slice = unsafe { slice::from_raw_parts(self.ids, self.ids_size) };
            let mut id_list: Vec<Checksum256> = Vec::with_capacity(self.ids_size);
            for id in ids_slice.iter() {
                let i = id.clone().try_into()?;
                id_list.push(i);
            }

            Ok(id_list)
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

impl TryInto<ActionReceipt> for ActionReceiptFFI {
    type Error = Error;
    fn try_into(self) -> Result<ActionReceipt, Self::Error> {
        if self.auth_sequence.is_null() {
            Err(Error::NullPtr)
        } else {
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

            Ok(ActionReceipt {
                receiver,
                act_digest,
                global_sequence,
                recv_sequence,
                auth_sequence,
                code_sequence,
                abi_sequence
            })
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct IncrementalMerkleFFI {
    pub _node_count: c_ulonglong,
    pub _active_nodes: *const Checksum256,
    pub _active_nodes_size: usize,
}

impl TryInto<IncrementalMerkle> for IncrementalMerkleFFI {
    type Error = Error;
    fn try_into(self) -> Result<IncrementalMerkle, Self::Error> {
        dbg!(self._active_nodes);
        if self._active_nodes.is_null() {
//        if false {
            dbg!("IncrementalMerkleFFI is null");
            Err(Error::NullPtr)
        } else {
            let _active_nodes = unsafe {
                slice::from_raw_parts(self._active_nodes, self._active_nodes_size).to_vec()
            };

            Ok(IncrementalMerkle::new(self._node_count, _active_nodes))
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ExtensionFFI {
    pub _type: c_ushort,
    pub data: *const c_char,
    pub data_size: usize,
}

impl TryInto<Extension> for ExtensionFFI {
    type Error = Error;
    fn try_into(self) -> Result<Extension, Self::Error> {
        if self.data.is_null() {
            Err(Error::NullPtr)
        } else {
            let ext = unsafe { slice::from_raw_parts(self.data, self.data_size).iter().map(|c| *c as u8 ).collect::<Vec<u8>>() };

            Ok(Extension(self._type, ext))
        }

    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ExtensionsFFI {
    pub extensions: *const ExtensionFFI,
    pub extensions_size: usize,
}

impl TryInto<Vec<Extension>> for ExtensionsFFI {
    type Error = Error;
    fn try_into(self) -> Result<Vec<Extension>, Self::Error> {
        if self.extensions.is_null() {
            Err(Error::NullPtr)
        } else {
            let exts = unsafe { slice::from_raw_parts(self.extensions, self.extensions_size) };
            let mut extensions: Vec<_> = Vec::with_capacity(exts.len());
            for (i, v) in exts.iter().enumerate() {
                extensions[i] = v.clone().try_into()?;
            }

            Ok(extensions)
        }
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

impl TryInto<ProducerKey> for ProducerKeyFFI {
    type Error = Error;
    fn try_into(self) -> Result<ProducerKey, Self::Error> {
        let producer_name = self.producer_name;

        let chars = Char::new(self.block_signing_key);
        let key_str = char_to_str(&chars)?;
        let block_signing_key = PublicKey::from_str(&key_str).map_err(|_| Error::PublicKeyError)?;

        Ok(ProducerKey { producer_name, block_signing_key })
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ProducerScheduleFFI {
    pub version: c_uint,
    pub producers: *const ProducerKeyFFI,
    pub producers_size: usize,
}

impl TryInto<ProducerSchedule> for ProducerScheduleFFI {
    type Error = Error;
    fn try_into(self) -> Result<ProducerSchedule, Self::Error>  {
        if self.producers.is_null() {
            return Err(Error::NullPtr);
        } else {
            let producers_ffi = unsafe { slice::from_raw_parts(self.producers, self.producers_size) };
            let mut producers: Vec<ProducerKey> = Vec::with_capacity(producers_ffi.len());
            for (i, p) in producers_ffi.iter().enumerate() {
                producers[i] = p.clone().try_into()?;
            }

            Ok(ProducerSchedule {
                version: self.version,
                producers
            })
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

impl TryInto<BlockHeader> for BlockHeaderFFI {
    type Error = Error;
    fn try_into(self) -> Result<BlockHeader, Self::Error> {
        if self.previous.is_null() || self.transaction_mroot.is_null() || self.action_mroot.is_null() {
            return Err(Error::NullPtr);
        }

        let new_producers: Option<ProducerSchedule> = {
            if self.new_producers.is_null() {
                None
            } else {
                let ffi = unsafe { ptr::read(self.new_producers) };
                Some(ffi.try_into()?)
            }
        };
        let header_extensions: Vec<Extension> = {
            if self.header_extensions.is_null() {
                vec![]
            } else {
                let ffi = unsafe { ptr::read(self.header_extensions) };
                ffi.try_into()?
            }
        };
        let (previous, transaction_mroot, action_mroot) = {
            let previous: [u8; 32] = {
                let slice = unsafe { slice::from_raw_parts(self.previous, 32) };
                unsafe { mem::transmute_copy(&slice[0]) } // 0 means copy all data as u8 from i8
            };
            let transaction_mroot: [u8; 32] = {
                let slice = unsafe { slice::from_raw_parts(self.transaction_mroot, 32) };
                unsafe { mem::transmute_copy(&slice[0]) }
            };
            let action_mroot: [u8; 32] = {
                let slice = unsafe { slice::from_raw_parts(self.action_mroot, 32) };
                unsafe { mem::transmute_copy(&slice[0]) }
            };

            (Checksum256::from(previous), Checksum256::from(transaction_mroot), Checksum256::from(action_mroot))
        };
        Ok(BlockHeader {
            timestamp: self.timestamp,
            producer: self.producer,
            confirmed: self.confirmed,
            previous,
            transaction_mroot,
            action_mroot,
            schedule_version: self.schedule_version,
            new_producers,
            header_extensions
        })
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SignedBlockHeaderFFI {
    pub block_header: *const BlockHeaderFFI,
    pub producer_signature: *const c_char,
}

impl TryInto<SignedBlockHeader> for SignedBlockHeaderFFI {
    type Error = Error;
    fn try_into(self) -> Result<SignedBlockHeader, Self::Error> {
        if self.block_header.is_null() || self.producer_signature.is_null() {
            return Err(Error::NullPtr);
        }

        let chars = Char::new(self.producer_signature);
        let ps_str = char_to_str(&chars)?;
        let producer_signature = Signature::from_str(&ps_str).map_err(|_| Error::SignatureError)?;

        let block_header: BlockHeader = {
            let ffi = unsafe { ptr::read(self.block_header) };
            ffi.try_into()?
        };

        Ok(SignedBlockHeader {
            block_header,
            producer_signature
        })
    }
}

pub(crate) fn char_to_string(cstr: *const c_char) -> FFIResult<String> {
    if cstr.is_null() {
        return Err(Error::NullPtr);
    }
    let cstr = unsafe { CStr::from_ptr(cstr) };
    let rust_string = cstr.to_str().map_err(|_| Error::CStrConvertError)?.to_string();

    Ok(rust_string)
}

pub(crate) fn char_to_str<'a>(ch: &'a Char) -> FFIResult<&'a str> {
    if ch.ptr.is_null() {
        return Err(Error::NullPtr);
    }
    let cstr = unsafe { CStr::from_ptr(**ch) };
    let slice = cstr.to_str().map_err(|_| Error::CStrConvertError)?;

    Ok(slice)
}

pub struct Char<'a> {
    pub ptr: *const c_char,
    ghost: PhantomData<&'a c_char>,
}

impl Char<'_> {
    fn new(ptr: *const c_char) -> Self {
        Self {
            ptr,
            ghost: PhantomData
        }
    }
}

impl Deref for Char<'_> {
    type Target = *const c_char;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

pub(crate) fn generate_raw_result(success: bool, msg: impl AsRef<str>) -> *const RpcResponse {
    let c_str = CString::new(msg.as_ref())
                .unwrap_or(
                    CString::new("unknow error type.").expect("failed to get raw pointer of error message")
                );
    let result = RpcResponse { success, msg: c_str.into_raw() };
    let box_result = Box::new(result);

    Box::into_raw(box_result)
}

// this struct will return to c++ caller
#[derive(Clone, Debug)]
#[repr(C)]
pub struct RpcResponse {
    success: bool,
    msg: *const c_char, // this could be error message or successful message
}
