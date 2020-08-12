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
use once_cell::sync::Lazy; // sync::OnceCell is thread-safe
use once_cell::sync::OnceCell; // sync::OnceCell is thread-safe
use subxt::{PairSigner, DefaultNodeRuntime as BifrostRuntime, Call, Client, system::{AccountStoreExt, System, SystemEventsDecoder}};
use sp_core::{sr25519::Pair, Pair as TraitPair};
use std::sync::{Arc, Mutex};

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
//	let client: Client<BifrostRuntime> = subxt::ClientBuilder::new()
//		.set_url(url.as_ref())
//		.build()
//		.await
//		.map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;
	let mut client = get_available_bifrost_client(urls).await?.lock().map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;

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
//	let client: Client<BifrostRuntime> = subxt::ClientBuilder::new()
//		.set_url(url.as_ref())
//		.build()
//		.await
//		.map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;
//	let mut client = global_client(url.as_ref()).await?.lock()
//						.map_err(|_| crate::Error::SubxtError("failed to get client builder"))?;
	let mut client = get_available_bifrost_client(urls).await?.lock().map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;

	let signer = Pair::from_string(signer.as_ref(), None).map_err(|_| crate::Error::WrongSudoSeed)?;
	let mut signer = PairSigner::<BifrostRuntime, Pair>::new(signer);

	// set nonce to avoid multiple trades using the same nonce, that will cause some trades will be abandoned.
	// https://substrate.dev/docs/en/knowledgebase/learn-substrate/tx-pool
	let current_nonce = client.account(&signer.signer().public().into(), None).await.map_err(|_| crate::Error::WrongSudoSeed)?.nonce;
	println!("signer current nonce is: {:?}", current_nonce);
	signer.increment_nonce();

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

	Ok(block_hash.to_string())
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
