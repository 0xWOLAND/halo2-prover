#[cfg(not(target_family = "wasm"))]
fn main() {
    use halo2_proofs::{circuit::Value, halo2curves::pasta::Fp};
    use halo2_prover::{arithmetic_circuit::*, utils::draw_graph};

    let k = 6;

    let constant = Fp::from(7);
    let x = Fp::from(6);
    let y = Fp::from(9);
    let z = Fp::from(36 * 81 + 8);
    let mut public_inputs = vec![constant, z];

    let arithmetic_circuit = ArithmeticCircuit {
        x: Value::known(x),
        y: Value::known(y),
        constant,
    };

    draw_graph(k, "Arithmetic Circuit".to_string(), arithmetic_circuit);
}
