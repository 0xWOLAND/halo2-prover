use std::error::Error;

use halo2_proofs::{dev::CircuitLayout, halo2curves::pasta::Fp, plonk::Circuit};
use plotters::prelude::*;

pub fn draw_graph(name: String, circuit: impl Circuit<Fp>, k: u32) {
    let root = SVGBackend::new(&name, (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.titled(&name, ("sans-serif", 30)).unwrap();

    CircuitLayout::default()
        .show_equality_constraints(true)
        .view_width(0..2)
        .view_height(0..16)
        .show_labels(true)
        .render(k, &circuit, &root)
        .unwrap()
}
