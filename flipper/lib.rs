#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flipper {
	use ink::prelude::{ vec::Vec};
	use risc0_zkvm::{serde::from_slice, SessionReceipt};
	// /// Defines the storage of your contract.
	// /// Add new fields to the below struct in order
	// /// to add new static storage fields to your contract.
	#[ink(storage)]
	pub struct Flipper {
		/// Stores a single `bool` value on the storage.
		value: bool,
		v: Vec<u32>,
	}

	impl Flipper {
		/// Constructor that initializes the `bool` value to the given `init_value`.
		#[ink(constructor)]
		pub fn new() -> Self {
			Self { value: false, v: Vec::new()}
		}

		/// Constructor that initializes the `bool` value to `false`.
		///
		/// Constructors can delegate to other constructors.
		// #[ink(constructor)]
		// pub fn default() -> Self {
		// 	Self::new(Default::default())
		// }

		/// A message that can be called on instantiated contracts.
		/// This one flips the value of the stored `bool` from `true`
		/// to `false` and vice versa.
		#[ink(message)]
		// pub fn flip(&mut self, proof_bytes: Vec<u8>) {
		pub fn flip(&mut self, scale_decoded_receipt: Vec<u32>) -> Result<u32, ()> {
			// Known image id for the current prover code
			let image_id: [u32; 8] = [1412254835, 707141561, 3873615143, 845298726, 2015286835, 1880548615, 1675505293, 3069112200];
			self.v = scale_decoded_receipt[&scale_decoded_receipt.len()-100..].to_vec();
			self.v.push(scale_decoded_receipt.len() as u32);
			let receipt: Result<SessionReceipt, _> = from_slice(&scale_decoded_receipt);
			return Ok(1);
			// if let Ok(receipt) = receipt {
			// 	// Check verification of proof
			// 	let _ = receipt.verify(image_id);
			// 	self.v = Vec::new();
			// 	self.v.push(10);
			// 	return Ok(0);
			// }
			// self.v.push(5);
			// Ok(0)
		}

		#[ink(message)]
		pub fn accept(&mut self, proof_bytes: Vec<u32>) {
			self.v = proof_bytes
		}

		#[ink(message)]
		pub fn read_v(&self) -> Vec<u32> {
			self.v.clone()
		}
	}
}

/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
/// module and test functions are marked with a `#[test]` attribute.
/// The below code is technically just normal Rust code.
#[cfg(test)]
mod tests {
	/// Imports all the definitions from the outer scope so we can use them here.
	use super::*;

	/// We test if the default constructor does its job.
	#[ink::test]
	fn default_works() {
		let flipper = Flipper::default();
		assert_eq!(flipper.get(), false);
	}

	/// We test a simple use case of our contract.
	#[ink::test]
	fn it_works() {
		let mut flipper = Flipper::new(false);
		assert_eq!(flipper.get(), false);

		let receipt = vec![];

		flipper.flip(receipt);
		assert_eq!(flipper.get(), true);
	}
}

/// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
///
/// When running these you need to make sure that you:
/// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
/// - Are running a Substrate node which contains `pallet-contracts` in the background
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
	/// Imports all the definitions from the outer scope so we can use them here.
	use super::*;

	/// A helper function used for calling contract messages.
	use ink_e2e::build_message;

	/// The End-to-End test `Result` type.
	type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	/// We test that we can upload and instantiate the contract using its default constructor.
	#[ink_e2e::test]
	async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
		// Given
		let constructor = FlipperRef::default();

		// When
		let contract_account_id = client
			.instantiate("flipper", &ink_e2e::alice(), constructor, 0, None)
			.await
			.expect("instantiate failed")
			.account_id;

		// Then
		let get =
			build_message::<FlipperRef>(contract_account_id.clone()).call(|flipper| flipper.get());
		let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
		assert!(matches!(get_result.return_value(), false));

		Ok(())
	}

	/// We test that we can read and write a value from the on-chain contract contract.
	#[ink_e2e::test]
	async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
		// Given
		let constructor = FlipperRef::new(false);
		let contract_account_id = client
			.instantiate("flipper", &ink_e2e::bob(), constructor, 0, None)
			.await
			.expect("instantiate failed")
			.account_id;

		let get =
			build_message::<FlipperRef>(contract_account_id.clone()).call(|flipper| flipper.get());
		let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
		assert!(matches!(get_result.return_value(), false));

		// When
		let flip =
			build_message::<FlipperRef>(contract_account_id.clone()).call(|flipper| flipper.flip());
		let _flip_result = client.call(&ink_e2e::bob(), flip, 0, None).await.expect("flip failed");

		// Then
		let get =
			build_message::<FlipperRef>(contract_account_id.clone()).call(|flipper| flipper.get());
		let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
		assert!(matches!(get_result.return_value(), true));

		Ok(())
	}
}
