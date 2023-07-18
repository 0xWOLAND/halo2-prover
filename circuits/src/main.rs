#[cfg(not(target_family = "wasm"))]
fn main() {
    use halo2_proofs::{
        arithmetic::Field,
        halo2curves::bn256::{self, Fr},
    };
    use halo2_prover::{
        arithmetic_circuit,
        collatz::{self, collatz_conjecture},
        poseidon_circuit::{self, PoseidonSpec},
        utils::*,
    };

    // Arithmetic Circuit
    let k = 4;
    let circuit = arithmetic_circuit::empty_circuit(0);
    draw_graph(k, "img/arithmetic_circuit.svg", &circuit, Some(5));

    // Collatz
    let k = 10;
    let circuit = collatz::create_circuit(collatz_conjecture(4));
    draw_graph(k, "img/collatz.svg", &circuit, Some(1 << 6));
    // Poseidon hash

    let k = 6;
    use rand_core::{OsRng, RngCore};
    const L: usize = 11;
    const WIDTH: usize = 12;
    const RATE: usize = 11;
    let message: Vec<u64> = (0..L).map(|_| OsRng.next_u64()).collect::<Vec<_>>();
    let circuit =
        poseidon_circuit::create_circuit::<PoseidonSpec<WIDTH, RATE>, WIDTH, RATE, L>(message);

    draw_graph(k, "img/poseidon.svg", &circuit, Some(1 << 6));
}
