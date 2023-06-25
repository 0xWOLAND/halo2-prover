mod arithmetic_circuit;
mod collatz;
mod utils;
use arithmetic_circuit::*;
use collatz::{collatz_conjecture, CollatzCircuit};
use halo2_proofs::{circuit::Value, halo2curves::pasta::Fp, plonk::Circuit};
use yew::prelude::*;

use crate::utils::draw_graph;

fn arithmetic_circuit_test() -> (u32, ArithmeticCircuit<Fp>, Vec<Fp>) {
    println!("Arithmetic Circuit Test");
    let k: u32 = 4;
    let constant = Fp::from(7);
    let x = Fp::from(6);
    let y = Fp::from(9);
    let z = Fp::from(36 * 81 + 8);
    let mut public_inputs = vec![constant, z];

    (
        k,
        ArithmeticCircuit {
            x: Value::known(x),
            y: Value::known(y),
            constant,
        },
        public_inputs,
    )
}

fn collatz_circuit_test() -> (u32, CollatzCircuit<Fp>, Vec<Fp>) {
    let k = 8;
    println!("Collatz Circuit Test");
    let x: Vec<Value<_>> = collatz_conjecture(7)
        .iter()
        .map(|y: &u64| Value::known(Fp::from(*y)))
        .collect();

    (k, CollatzCircuit { x }, vec![])
}
mod test {

    use crate::collatz::{collatz_conjecture, CollatzCircuit};

    use super::*;
    use halo2_proofs::circuit::Value;
    use halo2_proofs::dev::{MockProver, VerifyFailure};
    use halo2_proofs::halo2curves::pasta::Fp;
    // use halo2_proofs::pasta::Fp;

    #[test]
    pub fn arithmetic_circuit() -> Result<(), Vec<VerifyFailure>> {
        let (k, circuit, public_inputs) = arithmetic_circuit_test();
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        prover.verify()
    }

    pub fn collatz_circuit(mut n: u64) -> Result<(), Vec<VerifyFailure>> {
        let (k, circuit, _) = collatz_circuit_test();

        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.verify()
    }
}

#[function_component]
fn App() -> Html {
    let proof = match test::collatz_circuit(4) {
        Ok(()) => "Proof Passed",
        Err(_) => "Proof Failed",
    };
    let (k, circuit, public_inputs) = arithmetic_circuit_test();
    draw_graph("img/collatz.svg".to_string(), circuit, k);

    let proof_val = use_state(|| proof);

    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 20;
            counter.set(value);
        }
    };

    html! {
        <div>
            <button {onclick}>{ "+20" }</button>
            <p>{ *counter }</p>
            <h1>{ *proof_val } </h1>
            <img src = "img/collatz.svg" alt="My Happy SVG"/>

        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
