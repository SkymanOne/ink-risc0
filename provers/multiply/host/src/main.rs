// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use methods::{MULTIPLY_ELF, MULTIPLY_ID};

use risc0_zkvm::SessionReceipt;
use risc0_zkvm::{
    default_executor_from_elf,
    serde::{from_slice, to_vec},
    ExecutorEnv,
};

use clap::Parser;
use codec::{Decode, Encode};
use sp_core::blake2_256;
use std::{str::FromStr, time::Instant};
use subxt::{
	blocks::ExtrinsicEvents,
	config::WithExtrinsicParams,
	ext::{
		sp_core::{
			sr25519::{Pair as SubxtPair, Public, Signature},
			Pair as SubxtPairT,
		},
		sp_runtime::{traits::Verify, AccountId32},
	},
	tx::{BaseExtrinsicParams, PairSigner, PlainTip, TxProgress},
	Error, OnlineClient, PolkadotConfig, SubstrateConfig,
};
// // Runtime types, etc
#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod substrate_node {}

use substrate_node::runtime_types::{
	frame_system::AccountInfo, sp_runtime::multiaddress::MultiAddress,
	sp_weights::weight_v2::Weight,
};
type ApiType = OnlineClient<
	WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
>;

// TODO: Check this
const PROOF_SIZE: u64 = u64::MAX / 2;

async fn send_receipt(
	// api: ApiType,
	// contract: AccountId,
	contract: AccountId32,
	// input_data: Vec<u8>,
	receipt: &SessionReceipt,
	// ) -> Result<TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>, Error> {
) -> Result<
	ExtrinsicEvents<
		WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
	>,
	Error,
> {
	let receipt_scale_encoded = to_vec(receipt).unwrap().encode();
	let receipt_recreated: SessionReceipt =
		from_slice(&Vec::<u32>::decode(&mut &receipt_scale_encoded[..]).unwrap()).unwrap();

	// if let Ok(scale_decoded_receipt) = &Vec::<u32>::decode(&mut &proof_bytes[..]) {
	// 	let receipt: Result<SessionReceipt, _> = from_slice(&scale_decoded_receipt);

	// 	if let Ok(receipt) = receipt {
	// 		// Check verification of proof
	// 		receipt.verify(image_id);
	// 	}
	// }
	println!("Vec: {:?}", to_vec(receipt));

	// If fails, the contract won't be able to verify the proof. If passed, all should be good for the contract to verify it
	assert_eq!(&receipt_recreated, receipt.clone());

	let mut call_data = Vec::<u8>::new();
	//append the selector
	call_data.append(&mut (blake2_256("accept".as_bytes())[0..4]).to_vec());
	//append the arguments
	call_data.append(&mut to_vec(receipt).unwrap().encode());

	let call_tx = substrate_node::tx().contracts().call(
		// MultiAddress::Id(contract)
		contract.into(),
		0, // value
		// Both need checking, or values from estimates. These ones come from contracts ui
		Weight { ref_time: 109_106_502_144, proof_size: 104_898_144 }, // gas_limit
		None,                                                  // storage_deposit_limit
		// To zkvm's serialization, then to SCALE encoding
		call_data,
	);

	let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();

	// This is just Alice, which is okay for an example
	let restored_key = SubxtPair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();
	let signer = PairSigner::new(restored_key);

	let result = api
		.tx()
		.sign_and_submit_then_watch_default(&call_tx, &signer)
		.await?
		.wait_for_in_block()
		.await?
		.fetch_events()
		.await?;

	// println!("Call result: {:?}", result);
	Ok(result)
}

// Multiply them inside the ZKP
pub fn multiply_factors(a: u64, b: u64) -> (SessionReceipt, u64) {
    let env = ExecutorEnv::builder()
        // Send a & b to the guest
        .add_input(&to_vec(&a).unwrap())
        .add_input(&to_vec(&b).unwrap())
        .build()
        .unwrap();

    // First, we make an executor, loading the 'multiply' ELF binary.
    let mut exec = default_executor_from_elf(env, MULTIPLY_ELF).unwrap();

    // Run the executor to produce a session.
    let session = exec.run().unwrap();

    // Prove the session to produce a receipt.
    let receipt = session.prove().unwrap();

    // Extract journal of receipt (i.e. output c, where c = a * b)
    let c: u64 = from_slice(&receipt.journal).expect(
        "Journal output should deserialize into the same types (& order) that it was written",
    );


    (receipt, c)
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Contract address
	#[arg(short, long)]
	contract_address: String,
}

#[tokio::main]
async fn main() {
	let args = Args::parse();
	// Pick two numbers
	let (receipt, _) = multiply_factors(17, 23);
	// println!("Receipt size: {}", to_vec(&receipt).unwrap().len());

	let contract_account = AccountId32::from_str(&args.contract_address).unwrap();

	// println!("With IMAGE_ID {:?}. Ensure that this is up-to-date in the contract", MULTIPLY_ID);

	// Verify receipt, panic if it's wrong
	receipt.verify(MULTIPLY_ID).expect(
		"Code you have proven should successfully verify; did you specify the correct image ID?",
	);
	println!("Vec: {:?}", to_vec(&receipt));

	// let res = send_receipt(contract_account, &receipt).await;
	// println!("Result: {}", res.is_ok());
	// println!("{:#?}", res);
}
