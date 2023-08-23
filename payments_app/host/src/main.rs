use payments_methods::{TRANSFER_ELF, TRANSFER_ID};
use nft_core::{
    payments::{
        state_machine::PaymentsStateMachine, 
        types::{Account, Address, CallType, Transaction}
    },
    traits::StateMachine,
};
use primitive_types::U256;
use risc0_zkvm::{
    default_executor_from_elf,
    serde::{from_slice, to_vec},
    ExecutorEnv,
};
use serde::ser::Serialize;
use sparse_merkle_tree::{
    default_store::DefaultStore, error::Error, traits::Hasher, traits::Value, MerkleProof,
    SparseMerkleTree, H256,
};
use std::time::SystemTime;

fn main() {
    let now = SystemTime::now();
    let mut state_machine = PaymentsStateMachine::new();
    
    let mut address_in_bytes = [0u8; 32];
    let mut address2_in_bytes = [0u8; 32];

    U256::from_dec_str("1").unwrap().to_big_endian(&mut address_in_bytes);
    U256::from_dec_str("2").unwrap().to_big_endian(&mut address2_in_bytes);

    let call_params = Transaction {
        from: Address(address_in_bytes),
        to: Address(address2_in_bytes), 
        amount: 100,
        call_type: CallType::Transfer,
    };

    let state_update = state_machine
        .call(call_params.clone())
        .unwrap();

    let env = ExecutorEnv::builder()
        .add_input(&to_vec(&call_params).unwrap())
        .add_input(&to_vec(&state_update).unwrap())
        .build()
        .unwrap();

    // Next, we make an executor, loading the (renamed) ELF binary.
    let mut exec = default_executor_from_elf(env, TRANSFER_ELF).unwrap();
    // Run the executor to produce a session.
    let session = exec.run().unwrap();
    let segments = session.resolve().unwrap();

    let cycles = segments
        .iter()
        .fold(0, |acc, segment| acc + (1 << segment.po2));

    println!("Executed, cycles: {}k", cycles / 1024);
    // Prove the session to produce a receipt.
    let receipt = session.prove().unwrap();

    match now.elapsed() {
        Ok(elapsed) => {
            // it prints '2'
            println!("execution done, time elapsed: {}s", elapsed.as_secs());
        }
        Err(e) => {
            // an error occurred!
            println!("Error: {e:?}");
        }
    }

    receipt.verify(TRANSFER_ID).unwrap();
}