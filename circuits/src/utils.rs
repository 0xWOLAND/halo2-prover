use std::error::Error;

use halo2_proofs::{
    dev::{CircuitLayout, MockProver},
    halo2curves::pasta::Fp,
    plonk::Circuit,
    poly::commitment::Params,
};
use plotters::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn draw_graph(k: u32, name: String, circuit: &impl Circuit<Fp>) {
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
    circuit: &impl Circuit<Fp>,
    public_input: &Vec<Fp>,
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
