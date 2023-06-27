use std::error::Error;

use halo2_proofs::halo2curves::bn256::Fr;
use halo2_proofs::{
    dev::{CircuitLayout, MockProver},
    plonk::Circuit,
    poly::commitment::Params,
};
use plotters::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(not(target_family = "wasm"))]
pub fn draw_graph(k: u32, name: String, circuit: &impl Circuit<Fr>) {
    let root = SVGBackend::new(&name, (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.titled(&name, ("sans-serif", 30)).unwrap();

    CircuitLayout::default()
        .show_equality_constraints(true)
        .view_width(0..2)
        .view_height(0..16)
        .show_labels(true)
        .render(k, circuit, &root)
        .unwrap()
}

pub fn run_mock_prover(
    k: u32,
    circuit: &impl Circuit<Fr>,
    public_input: &Vec<Fr>,
) -> Result<(), Vec<halo2_proofs::dev::VerifyFailure>> {
    let pub_inp = {
        if public_input.len() > 0 {
            vec![public_input.clone()]
        } else {
            vec![]
        }
    };
    let prover = MockProver::run(k, circuit, pub_inp).expect("Mock prover should run");

    prover.verify()
}

#[wasm_bindgen]
pub fn hello_world() -> String {
    "Hello World from Rust!".to_string()
}
