#[cfg(not(target_family = "wasm"))]
fn main() {
    use halo2_proofs::{
        circuit::Value,
        halo2curves::{bn256::Fr, pasta::Fp},
    };
    use halo2_prover::{
        arithmetic_circuit::*,
        collatz::*,
        utils::{draw_graph, run_mock_prover},
    };

    let k = 16;

    let x = generate_sequence(9);
    let circuit = create_circuit(&x);

    draw_graph(k, "img/collatz.svg".to_string(), &circuit);
    let res = run_mock_prover(k, &circuit, &vec![]);
    match res {
        Ok(()) => println!("Passed!"),
        _ => println!("didn't pass lol"),
    }

    let params = generate_params(k);
}
