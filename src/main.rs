mod arithmetic_circuit;
use arithmetic_circuit::*;
use yew::prelude::*;

mod test {

    use super::*;
    use halo2_proofs::circuit::Value;
    use halo2_proofs::dev::{MockProver, VerifyFailure};
    use halo2_proofs::halo2curves::pasta::Fp;
    // use halo2_proofs::pasta::Fp;

    pub fn test(test_string: &str) -> Result<(), Vec<VerifyFailure>> {
        println!("test string...{}", test_string);
        let k = 4;
        let constant = Fp::from(7);
        let x = Fp::from(6);
        let y = Fp::from(9);
        let z = Fp::from(36 * 81 + 7);

        let circuit: ArithmeticCircuit<Fp> = ArithmeticCircuit {
            x: Value::known(x),
            y: Value::known(y),
            constant: constant,
        };

        let mut public_inputs = vec![constant, z];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
        prover.verify()
    }
}

#[function_component]
fn App() -> Html {
    arithmetic_circuit::draw::draw_graph();
    let proof = match test::test("this is workingn") {
        Ok(res) => "Proof Passed",
        Err(_) => "Proof Failed",
    };

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
            <img src = "img/layout.svg" alt="My Happy SVG"/>

        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
