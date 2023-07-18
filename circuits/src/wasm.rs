use crate::{
    arithmetic_circuit::{self, parse_string, ArithmeticInput},
    collatz,
    poseidon::{
        self,
        primitives::{ConstantLength, Spec},
    },
    poseidon_circuit::{
        self, wasm_poseidon_solution, PoseidonInput, PoseidonSpec, WASM_POSEIDON_L,
        WASM_POSEIDON_RATE, WASM_POSEIDON_WIDTH,
    },
    utils::{
        generate_keys, generate_params, generate_proof, generate_proof_with_instance, hex_to_fr,
        verify, verify_with_instance,
    },
};
use halo2_proofs::{
    circuit::Value,
    halo2curves::{
        bn256::{Bn256, Fr, G1Affine},
        ff::PrimeField,
    },
    plonk::{keygen_pk, keygen_vk, Circuit, ProvingKey, VerifyingKey},
    poly::{commitment::Params, kzg::commitment::ParamsKZG},
};
use js_sys::Uint8Array;
use std::{
    cmp::min,
    io::{empty, BufReader},
    panic,
};

// Use a struct that impl's these functions and has a `getCurrentCircuit` function
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn to_uint8_array(a: Vec<u8>) -> Uint8Array {
    let res = Uint8Array::new_with_length(a.len() as u32);
    res.copy_from(&a);
    res
}

#[wasm_bindgen]
pub fn setup(k: u32) -> Uint8Array {
    let params = generate_params(k);
    let mut buf = vec![];
    params.write(&mut buf).expect("Should write params");

    to_uint8_array(buf)
}

pub fn wasm_generate_keys(
    params: &ParamsKZG<Bn256>,
    circuit: impl Circuit<Fr>,
) -> (ProvingKey<G1Affine>, VerifyingKey<G1Affine>) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let vk = keygen_vk(params, &circuit).expect("vk should not fail");
    let pk = keygen_pk(params, vk.clone(), &circuit).expect("keygen_pk should not fail");
    (pk, vk)
}

#[wasm_bindgen]
pub fn wasm_simulate_circuit(s: &str, circuit: i32) -> String {
    match circuit {
        0 => collatz::simulate_circuit(),
        1 => arithmetic_circuit::simulate_circuit(s),
        _ => poseidon_circuit::simulate_circuit(s),
    }
}

#[wasm_bindgen]
pub fn wasm_generate_proof(_params: &[u8], s: &str, circuit: i32) -> Uint8Array {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let params = ParamsKZG::<Bn256>::read(&mut BufReader::new(_params))
        .expect("should be able to read params");

    let proof = match circuit {
        0 => {
            let circuit = crate::collatz::create_circuit_from_string(s);
            let empty_circuit = collatz::empty_circuit();
            let (pk, vk) = wasm_generate_keys(&params, empty_circuit);
            generate_proof(&params, &pk, circuit)
        }
        1 => {
            let public_inputs: ArithmeticInput = parse_string(s);
            let circuit = crate::arithmetic_circuit::create_circuit_from_string(s);
            let empty_circuit = arithmetic_circuit::empty_circuit(public_inputs.constant);
            let public_inputs =
                [public_inputs.constant, public_inputs.z.unwrap()].map(|k| Fr::from(k));
            let (pk, vk) = wasm_generate_keys(&params, empty_circuit);
            generate_proof_with_instance(&params, &pk, circuit, &public_inputs)
        }
        _ => {
            log(&format!("raw public inputs - {:?}", s));
            let public_inputs: PoseidonInput = poseidon_circuit::parse_string(s);
            log(&format!("public inputs - {:?}", public_inputs.output));
            let circuit = poseidon_circuit::create_circuit_from_string::<
                PoseidonSpec<WASM_POSEIDON_WIDTH, WASM_POSEIDON_RATE>,
                WASM_POSEIDON_WIDTH,
                WASM_POSEIDON_RATE,
                WASM_POSEIDON_L,
            >(s);
            let empty_circuit = poseidon_circuit::empty_circuit::<
                PoseidonSpec<WASM_POSEIDON_WIDTH, WASM_POSEIDON_RATE>,
                WASM_POSEIDON_WIDTH,
                WASM_POSEIDON_RATE,
                WASM_POSEIDON_L,
            >();
            let (pk, _vk) = generate_keys(&params, &empty_circuit);

            let public_inputs: [Fr; 1] = [public_inputs.output.unwrap()].map(|k| hex_to_fr(k));
            generate_proof_with_instance(&params, &pk, circuit, &public_inputs)
        }
    };

    to_uint8_array(proof)
}

#[wasm_bindgen]
pub fn wasm_verify_proof(_params: &[u8], proof: &[u8], s: &str, circuit: i32) -> bool {
    let params = ParamsKZG::<Bn256>::read(&mut BufReader::new(_params))
        .expect("should be able to read params");
    let res = match circuit {
        0 => {
            log("Collatz");
            let empty_circuit = collatz::empty_circuit();
            let (pk, _vk) = generate_keys(&params, &empty_circuit);
            verify(&params, &pk, &proof.to_vec())
        }
        1 => {
            log("Arithmetic Circuit");
            let public_inputs: ArithmeticInput = arithmetic_circuit::parse_string(s);
            let empty_circuit = arithmetic_circuit::empty_circuit(public_inputs.constant);
            let (pk, _vk) = generate_keys(&params, &empty_circuit);
            let public_inputs =
                [public_inputs.constant, public_inputs.z.unwrap()].map(|k| Fr::from(k));
            verify_with_instance(&params, &pk, &proof.to_vec(), &public_inputs)
        }
        _ => {
            log("Poseidon Circuit");
            let public_inputs: PoseidonInput = poseidon_circuit::parse_string(s);
            let empty_circuit = poseidon_circuit::empty_circuit::<
                PoseidonSpec<WASM_POSEIDON_WIDTH, WASM_POSEIDON_RATE>,
                WASM_POSEIDON_WIDTH,
                WASM_POSEIDON_RATE,
                WASM_POSEIDON_L,
            >();
            let (pk, _vk) = generate_keys(&params, &empty_circuit);
            let message: [Fr; WASM_POSEIDON_L] = public_inputs
                .x
                .iter()
                .map(|k| Fr::from(*k))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

            let output = wasm_poseidon_solution::<
                PoseidonSpec<WASM_POSEIDON_WIDTH, WASM_POSEIDON_RATE>,
                WASM_POSEIDON_WIDTH,
                WASM_POSEIDON_RATE,
                WASM_POSEIDON_L,
            >(message);
            verify_with_instance(&params, &pk, &proof.to_vec(), &[output])
        }
    };

    match res {
        Err(e) => {
            log(&format!("{}", e));
            false
        }
        _ => true,
    }
}

#[wasm_bindgen]
pub fn get_circuit_count() -> i32 {
    3
}
