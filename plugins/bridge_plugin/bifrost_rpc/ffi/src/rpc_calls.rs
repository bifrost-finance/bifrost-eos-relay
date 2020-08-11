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
use eos_chain::{Action, ActionReceipt, Checksum256, IncrementalMerkle, ProducerAuthoritySchedule, SignedBlockHeader};
use subxt::{PairSigner, DefaultNodeRuntime as BifrostRuntime, Call, Client, system::{AccountStoreExt, System, SystemEventsDecoder}};
use sp_core::{sr25519::Pair, Pair as TraitPair};
use std::sync::atomic::{AtomicU32, Ordering};

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
	url:                  impl AsRef<str>,
	signer:               impl AsRef<str>,
	legacy_schedule_hash: Checksum256,
	schedule:             ProducerAuthoritySchedule,
	merkle:               IncrementalMerkle,
	block_headers:        Vec<SignedBlockHeader>,
	block_ids_list:       Vec<Vec<Checksum256>>
) -> Result<String, crate::Error> {
	let client: Client<BifrostRuntime> = subxt::ClientBuilder::new()
		.set_url(url.as_ref())
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
	let extrinsic = client.watch(args, &signer).await.map_err(|_| crate::Error::SubxtError("failed to commit this transaction"))?;
	let block_hash = extrinsic.block;

	Ok(block_hash.to_string())
}

pub async fn prove_action_call(
	url:                 impl AsRef<str>,
	signer:              impl AsRef<str>,
	action:              Action,
	action_receipt:      ActionReceipt,
	action_merkle_paths: Vec<Checksum256>,
	merkle:              IncrementalMerkle,
	block_headers:       Vec<SignedBlockHeader>,
	block_ids_list:      Vec<Vec<Checksum256>>,
	trx_id:              Checksum256
) -> Result<String, crate::Error> {
	let client: Client<BifrostRuntime> = subxt::ClientBuilder::new()
		.set_url(url.as_ref())
		.build()
		.await
		.map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;

	let signer = Pair::from_string(signer.as_ref(), None).map_err(|_| crate::Error::WrongSudoSeed)?;
	let mut signer = PairSigner::<BifrostRuntime, Pair>::new(signer);

	// set nonce to avoid multiple trades using the same nonce, that will cause some trades will be abandoned.
	// https://substrate.dev/docs/en/knowledgebase/learn-substrate/tx-pool
	static atomic_nonce: AtomicU32 = AtomicU32::new(0);
	let current_nonce = client.account(&signer.signer().public().into(), None).await.map_err(|_| crate::Error::WrongSudoSeed)?.nonce;
	println!("signer current nonce is: {:?}", current_nonce);
	let next_nonce = get_latest_nonce(&atomic_nonce, current_nonce);
	println!("signer next nonce is: {:?}", next_nonce);
	signer.set_nonce(next_nonce);

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
	let block_hash = client.submit(call, &signer).await.map_err(|_| crate::Error::SubxtError("failed to commit this transaction"))?;
	// if trade success, change nonce
	atomic_update_nonce(&atomic_nonce, current_nonce);

	Ok(block_hash.to_string())
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
