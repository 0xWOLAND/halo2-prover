#[cfg(not(target_family = "wasm"))]
fn main() {
    use halo2_prover::{collatz::*, utils::*};

    let k = 16;

    let mut x = collatz_conjecture(9);
    // Uncomment to test invalid sequence
    // x[31] = 2;

    let circuit = create_circuit(x);

    draw_graph(k, "img/collatz.svg".to_string(), &circuit);
    let res = run_mock_prover(k, &circuit, &vec![]);

    let params = generate_params(k);

    let empty_circuit = empty_circuit();
    let (pk, vk) = generate_keys(&params, empty_circuit);

    let proof = generate_proof(&params, &pk, circuit);
    let res = verify(&params, &vk, &proof);
    println!("RES: {:?}", res);
}
