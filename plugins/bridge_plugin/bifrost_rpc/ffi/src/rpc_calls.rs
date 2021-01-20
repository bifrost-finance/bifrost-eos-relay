// Copyright 2019-2020 Liebi Technologies.
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

use codec::Encode;
use core::marker::PhantomData;
use eos_chain::{
	Action, ActionReceipt, Checksum256, Digest, IncrementalMerkle,
	ProducerAuthoritySchedule, SignedBlockHeader
};
use once_cell::sync::Lazy; // sync::OnceCell is thread-safe
use once_cell::sync::OnceCell; // sync::OnceCell is thread-safe
use subxt::{
	PairSigner, DefaultNodeRuntime as BifrostRuntime, Call, Client,
	system::{AccountStoreExt, System, SystemEventsDecoder}, Error as SubxtErr,
};
use sp_core::{sr25519::Pair, Pair as TraitPair};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};

static BIFROST_RPC_CLIENT: Lazy<Arc<Mutex<subxt::ClientBuilder<BifrostRuntime>>>> = {
	Lazy::new(move || {
		let builder: subxt::ClientBuilder<BifrostRuntime> = subxt::ClientBuilder::new();
		Arc::new(Mutex::new(builder))
	})
};

async fn global_client(url: &str) -> Result<&'static Mutex<subxt::Client<BifrostRuntime>>, crate::Error> {
	static INSTANCE: OnceCell<Mutex<subxt::Client<BifrostRuntime>>> = OnceCell::new();
	let builder = subxt::ClientBuilder::new().set_url(url).build().await.map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;
	Ok(INSTANCE.get_or_init(|| {
		Mutex::new(builder)
	}))
}

#[subxt::module]
pub trait BridgeEos: System {}

impl BridgeEos for BifrostRuntime {}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct ChangeScheduleCall<T: BridgeEos> {
	legacy_schedule_hash: Checksum256,
	schedule:             ProducerAuthoritySchedule,
	merkle:               IncrementalMerkle,
	block_headers:        Vec<SignedBlockHeader>,
	block_ids_list:       Vec<Vec<Checksum256>>,
	pub _runtime:         PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct ProveActionCall<T: BridgeEos> {
	action:               Action,
	action_receipt:       ActionReceipt,
	action_merkle_paths:  Vec<Checksum256>,
	merkle:               IncrementalMerkle,
	block_headers:        Vec<SignedBlockHeader>,
	block_ids_list:       Vec<Vec<Checksum256>>,
	trx_id:               Checksum256,
	pub _runtime:         PhantomData<T>,
}

pub async fn change_schedule_call(
	urls:                 impl IntoIterator<Item=String>,
	signer:               impl AsRef<str>,
	legacy_schedule_hash: Checksum256,
	schedule:             ProducerAuthoritySchedule,
	merkle:               IncrementalMerkle,
	block_headers:        Vec<SignedBlockHeader>,
	block_ids_list:       Vec<Vec<Checksum256>>
) -> Result<String, crate::Error> {
	let url: String = urls.into_iter().take(1).next().ok_or(crate::Error::SubxtError("failed to create subxt client"))?;
	let client: Client<BifrostRuntime> = subxt::ClientBuilder::new()
		.set_url(url)
		.build()
		.await
		.map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;

	let signer = Pair::from_string(signer.as_ref(), None).map_err(|_| crate::Error::WrongSudoSeed)?;
	let signer = PairSigner::<BifrostRuntime, Pair>::new(signer);

	let args = ChangeScheduleCall::<BifrostRuntime> {
		legacy_schedule_hash,
		schedule,
		merkle,
		block_headers,
		block_ids_list,
		_runtime: PhantomData
	};
	let block_hash = client.submit(args, &signer).await.map_err(|_| crate::Error::SubxtError("failed to commit this transaction"))?;

	Ok(block_hash.to_string())
}

pub async fn prove_action_call(
	urls:                impl IntoIterator<Item=String>,
	signer:              impl AsRef<str>,
	action:              Action,
	action_receipt:      ActionReceipt,
	action_merkle_paths: Vec<Checksum256>,
	merkle:              IncrementalMerkle,
	block_headers:       Vec<SignedBlockHeader>,
	block_ids_list:      Vec<Vec<Checksum256>>,
	trx_id:              Checksum256
) -> Result<String, crate::Error> {
	let url: String = urls.into_iter().take(1).next().ok_or(crate::Error::SubxtError("failed to create subxt client"))?;
	let client: Client<BifrostRuntime> = subxt::ClientBuilder::new()
		.set_url(url)
		.build()
		.await
		.map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;

	let signer = Pair::from_string(signer.as_ref(), None).map_err(|_| crate::Error::WrongSudoSeed)?;
	let mut signer = PairSigner::<BifrostRuntime, Pair>::new(signer);

	// set nonce to avoid multiple trades using the same nonce, that will cause some trades will be abandoned.
	// https://substrate.dev/docs/en/knowledgebase/learn-substrate/tx-pool
	static atomic_nonce: AtomicU32 = AtomicU32::new(0);
//	static atomic_nonce: AtomicU32 = AtomicU32::new(0);
//	static signer_current_nonce: AtomicU32 = AtomicU32::new(0);
//	static mut latest_nonce: u32 = 0;
	let current_nonce = client.account(&signer.signer().public().into(), None).await.map_err(|_| crate::Error::WrongSudoSeed)?.nonce;
	// initialize signer current nonce
//	if signer_current_nonce.load(Ordering::Relaxed) == 0 {
//		signer_current_nonce.swap(current_nonce, Ordering::Relaxed);
//	}

	// ensure atomic nonce is bigger than current user nonce
	if atomic_nonce.load(Ordering::Relaxed) <= current_nonce {
		atomic_nonce.swap(current_nonce, Ordering::Relaxed);
	}
//
//	// this means signer nonce has changed.
//	if signer_current_nonce.load(Ordering::Relaxed) < current_nonce {
//		signer_current_nonce.swap(current_nonce, Ordering::Relaxed);
//	}

//	println!("current nonce is: {:?}", current_nonce);
//	let gap = signer_current_nonce.load(Ordering::Relaxed) as i32 - current_nonce as i32;
//	match gap {
//		gap if gap == 0 => {
//			// no change on nonce
////			atomic_nonce.fetch_add(1, Ordering::SeqCst); // increment 1
//			signer.set_nonce(atomic_nonce.load(Ordering::Relaxed) + 1 + current_nonce);
//			println!("equal signer current nonce is: {:?}", atomic_nonce.load(Ordering::Relaxed) + 1 + current_nonce);
//			println!("equal atomic_nonce is: {:?}", atomic_nonce);
////			atomic_nonce.fetch_add(1, Ordering::SeqCst); // increment 1
//		}
//		gap if gap > 0 => {
//			// maybe it's unlikely to happen
//			();
//		}
//		gap if gap < 0 => {
//			// nonce has changed
//			signer.set_nonce(current_nonce);
//			println!("new signer current nonce is: {:?}", current_nonce);
//			println!("new atomic_nonce is: {:?}", atomic_nonce);
////			atomic_nonce.swap(0, Ordering::Relaxed);
////			signer_current_nonce.swap(current_nonce, Ordering::Relaxed);
////			atomic_nonce.swap(0, Ordering::Relaxed);
//		}
//		_ => (),
//	}

//	println!("atomic_nonce is: {:?}", atomic_nonce);
//	println!("signer_current_nonce is: {:?}", atomic_nonce.load(Ordering::Relaxed));
	println!("signer_current_nonce is: {:?}", current_nonce);
	signer.set_nonce(atomic_nonce.load(Ordering::Relaxed));
	atomic_nonce.fetch_add(1, Ordering::SeqCst);

	let call = ProveActionCall::<BifrostRuntime> {
		action,
		action_receipt,
		action_merkle_paths,
		merkle,
		block_headers,
		block_ids_list,
		trx_id,
		_runtime: PhantomData
	};
//	let block_hash = client.submit(call, &signer).await.map_err(|e| {
//		if let SubxtErr::Rpc(err) = e {
//			// the full error: Rpc(Request(Error { code: ServerError(1014), message: "Priority is too low: (0 vs 0)",
//			// data: Some(String("The transaction has too low priority to replace another transaction already in the pool.")) }))
//			println ! ("error is: {:?}", err.to_string());
//			//			if err.to_string().as_str().contains("Priority is too low") {
//			//				atomic_nonce.fetch_add(1, Ordering::SeqCst);
//			//			}
//		}
//		crate::Error::SubxtError("failed to commit this transaction")
//	})?;
//	if let Err(SubxtErr::Rpc(e)) = client.submit(call, &signer).await {
//		signer.increment_nonce();
//		let trx_id = client.submit(call, &signer).await.map_err(|e| {
//			println ! ("error is: {:?}", e.to_string());
//			crate::Error::SubxtError("failed to commit this transaction")
//		})?;
//		return Ok(trx_id.to_string());
//	}
	match client.submit(call.clone(), &signer).await {
		Ok(trx_id) => Ok(trx_id.to_string()),
		Err(SubxtErr::Rpc(e)) => {
//			signer.increment_nonce();
			let trx_id = client.submit(call, &signer).await.map_err(|e| {
				println ! ("error is: {:?}", e.to_string());
				crate::Error::SubxtError("failed to commit this transaction")
			})?;
			Ok(trx_id.to_string())
		}
		_ => Err(crate::Error::SubxtError("failed to commit this transaction"))
	}

//	let mut index = 0u32;
//	loop {
//		println!("signer_current_nonce is: {:?}, index: {:?}", current_nonce, index);
//		match client.submit(call.clone(), &signer).await {
//			Ok(trx_id) => return Ok(trx_id.to_string()),
//			Err(SubxtErr::Rpc(e)) => {
//				if e.to_string().as_str().contains("Priority is too low") {
//					index += 1;
//					signer.increment_nonce();
//				}
//			}
//			_ => {}
//		}
//
//		if index >= 30 {
//			break;
//		}
//	}
//
//	Err(crate::Error::SubxtError("failed to commit this transaction"))
//	while let Err(SubxtErr::Rpc(e)) = client.submit(call.clone(), &signer).await {
//		if err.to_string().as_str().contains("Priority is too low") {
//			signer.increment_nonce();
//		}
//	}
//	atomic_nonce.fetch_add(1, Ordering::SeqCst);

//	let gap = atomic_nonce.load(Ordering::Relaxed) as i32 - current_nonce as i32;
//	if gap >= 30 {
//		atomic_nonce.swap(current_nonce, Ordering::Relaxed);
//	}

//	match gap {
//		gap if gap == 0 => {
//			// no change on nonce
//			atomic_nonce.fetch_add(1, Ordering::SeqCst); // increment 1
//		}
//		gap if gap > 0 => {
//			// maybe it's unlikely to happen
//			();
//		}
//		gap if gap < 0 => {
//			// nonce has changed
//			signer_current_nonce.swap(current_nonce, Ordering::Relaxed);
//			atomic_nonce.swap(0, Ordering::Relaxed);
//		}
//		_ => (),
//	}

	// if trade success, change nonce
//	atomic_update_nonce(&atomic_nonce, current_nonce);
//	atomic_nonce.fetch_add(1, Ordering::SeqCst);

//	Ok(block_hash.to_string())
}

async fn get_available_bifrost_client(urls: impl IntoIterator<Item=String>)
	-> Result<&'static Mutex<subxt::Client<BifrostRuntime>>, crate::Error>
{
	for url in urls.into_iter() {
		let client = global_client(url.as_ref()).await;
		if client.is_ok() {
			return client;
		}
	}

	Err(crate::Error::SubxtError("failed to get client builder"))
}

// update nonce to avoid using the same nonce
pub fn get_latest_nonce(atomic_nonce: &AtomicU32, current_nonce: u32) -> u32 {
	if atomic_nonce.load(Ordering::Relaxed) < current_nonce {
		current_nonce
	} else {
		atomic_nonce.load(Ordering::Relaxed) + 1
	}
}

pub fn atomic_update_nonce(atomic_nonce: &AtomicU32, current_nonce: u32) {
	if atomic_nonce.load(Ordering::Relaxed) < current_nonce {
		atomic_nonce.swap(current_nonce, Ordering::Relaxed);
	} else {
		atomic_nonce.fetch_add(1, Ordering::SeqCst);
	}
}
