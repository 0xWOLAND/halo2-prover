use halo2_proofs::halo2curves::bn256::Fr;
use halo2_proofs::halo2curves::ff::{Field, PrimeField};

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};

use halo2_gadgets::poseidon::{
    primitives::{self as poseidon, generate_constants, ConstantLength, Mds, Spec},
    Hash, Pow5Chip, Pow5Config,
};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::marker::PhantomData;
use std::panic;
use wasm_bindgen::prelude::wasm_bindgen;

static N_ROUNDS_F: i32 = 8;
static N_ROUNDS_P: [i32; 16] = [
    56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
];
pub const WASM_POSEIDON_WIDTH: usize = 3;
pub const WASM_POSEIDON_RATE: usize = 2;
pub const WASM_POSEIDON_L: usize = 2;

use crate::{constants::constants, unstringify::unstringifyHex};
#[derive(Copy, Clone)]
pub struct PoseidonCircuit<S, const WIDTH: usize, const RATE: usize, const L: usize>
where
    S: Spec<Fr, WIDTH, RATE> + Clone + Copy,
{
    message: Value<[Fr; L]>,
    _spec: PhantomData<S>,
}

#[derive(Serialize, Deserialize)]
pub struct PoseidonInput<'a> {
    pub x: Vec<u64>,
    pub output: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct PoseidonConfig<const WIDTH: usize, const RATE: usize, const L: usize> {
    input: [Column<Advice>; L],
    expected: Column<Instance>,
    poseidon_config: Pow5Config<Fr, WIDTH, RATE>,
}

impl<S, const WIDTH: usize, const RATE: usize, const L: usize> Circuit<Fr>
    for PoseidonCircuit<S, WIDTH, RATE, L>
where
    S: Spec<Fr, WIDTH, RATE> + Copy + Clone,
{
    type Config = PoseidonConfig<WIDTH, RATE, L>;
    type FloorPlanner = SimpleFloorPlanner;

    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self {
            message: Value::unknown(),
            _spec: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        let state = (0..WIDTH).map(|_| meta.advice_column()).collect::<Vec<_>>();
        let expected = meta.instance_column();
        meta.enable_equality(expected);
        let partial_sbox = meta.advice_column();

        let rc_a = (0..WIDTH).map(|_| meta.fixed_column()).collect::<Vec<_>>();
        let rc_b = (0..WIDTH).map(|_| meta.fixed_column()).collect::<Vec<_>>();

        meta.enable_constant(rc_b[0]);

        Self::Config {
            input: state[..RATE].try_into().unwrap(),
            expected,
            poseidon_config: Pow5Chip::configure::<S>(
                meta,
                state.try_into().unwrap(),
                partial_sbox,
                rc_a.try_into().unwrap(),
                rc_b.try_into().unwrap(),
            ),
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        let chip = Pow5Chip::construct(config.poseidon_config.clone());

        let message = layouter.assign_region(
            || "load message",
            |mut region| {
                let message_word = |i: usize| {
                    let value = self.message.map(|message_vals| message_vals[i]);
                    region.assign_advice(
                        || format!("load message_{}", i),
                        config.input[i],
                        0,
                        || value,
                    )
                };
                let message: Result<Vec<_>, Error> = (0..L).map(message_word).collect();
                Ok(message?.try_into().unwrap())
            },
        )?;

        let hasher = Hash::<_, _, S, ConstantLength<L>, WIDTH, RATE>::init(
            chip,
            layouter.namespace(|| "init"),
        )?;
        let output = hasher.hash(layouter.namespace(|| "hash"), message)?;

        layouter.constrain_instance(output.cell(), config.expected, 0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PoseidonSpec<const WIDTH: usize, const RATE: usize>;

impl<const WIDTH: usize, const RATE: usize> Spec<Fr, WIDTH, RATE> for PoseidonSpec<WIDTH, RATE> {
    fn full_rounds() -> usize {
        8
    }

    fn partial_rounds() -> usize {
        N_ROUNDS_P[WIDTH] as usize
    }

    fn sbox(val: Fr) -> Fr {
        val.pow_vartime(&[5])
    }

    fn secure_mds() -> usize {
        0
    }

    fn constants() -> (Vec<[Fr; WIDTH]>, Mds<Fr, WIDTH>, Mds<Fr, WIDTH>) {
        generate_constants::<_, Self, WIDTH, RATE>()
    }
}

const K: u32 = 7;

fn sbox(x: Fr) -> Fr {
    let y = x * x;
    return y * y * x;
}

fn mix(state: Vec<Fr>, M: &Vec<Vec<Fr>>) -> Vec<Fr> {
    let mut out = vec![];
    for x in 0..state.len() {
        let mut o = Fr::from(0);
        for y in 0..state.len() {
            o += M[x][y] * state[y];
        }
        out.push(o);
    }
    out
}

fn poseidon(inputs: Vec<Fr>) -> Result<Fr, String> {
    let n_rounds_f = N_ROUNDS_F;
    let n_rounds_p = N_ROUNDS_P[0];
    let t = inputs.len() + 1;

    let (C, M) = constants();
    #[allow(non_upper_case_globals)]
    let C: Vec<Fr> = C
        .iter()
        .map(|x| Fr::from_str_vartime(&unstringifyHex(x)).unwrap())
        .collect();
    let M: Vec<Vec<Fr>> = M
        .iter()
        .map(|y| {
            y.iter()
                .map(|x| Fr::from_str_vartime(&unstringifyHex(x)).unwrap())
                .collect()
        })
        .collect();
    println!("{}", format!("{:?}", M[0][0]));

    if M.len() != t {
        return Err(format!(
            "invalid `M` length: Expected {} got {}",
            M.len(),
            t
        ));
    }

    let mut state = vec![Fr::ZERO];
    state.extend_from_slice(&inputs);
    for x in 0..(n_rounds_f + n_rounds_p) {
        for y in 0..state.len() {
            state[y] += C[(x as usize) * t + y];
            if x < n_rounds_f / 2 || x >= n_rounds_f / 2 + n_rounds_p {
                state[y] = sbox(state[y]);
            } else if y == 0 {
                state[y] = sbox(state[y]);
            }
        }
        state = mix(state, &M);
    }

    Ok(state[0])
}

pub fn empty_circuit<S, const WIDTH: usize, const RATE: usize, const L: usize>(
) -> PoseidonCircuit<S, WIDTH, RATE, L>
where
    S: Spec<Fr, WIDTH, RATE> + Copy + Clone,
{
    PoseidonCircuit::<S, WIDTH, RATE, L> {
        message: Value::unknown(),
        _spec: PhantomData,
    }
}

pub fn parse_string(s: &str) -> PoseidonInput {
    serde_json::from_str(s).unwrap()
}

pub fn create_circuit_from_string<S, const WIDTH: usize, const RATE: usize, const L: usize>(
    s: &str,
) -> PoseidonCircuit<S, WIDTH, RATE, L>
where
    S: Spec<Fr, WIDTH, RATE> + Copy + Clone,
{
    let v = parse_string(s);
    let mut sequence = v.x;
    sequence.resize(L, 1);
    create_circuit(sequence)
}

pub fn create_circuit<S, const WIDTH: usize, const RATE: usize, const L: usize>(
    message: Vec<u64>,
) -> PoseidonCircuit<S, WIDTH, RATE, L>
where
    S: Spec<Fr, WIDTH, RATE> + Copy + Clone,
{
    let message: [Fr; L] = message
        .clone()
        .iter()
        .map(|f| Fr::from(*f))
        .collect::<Vec<Fr>>()
        .try_into()
        .unwrap();

    PoseidonCircuit::<S, WIDTH, RATE, L> {
        message: Value::known(message),
        _spec: PhantomData,
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn simulate_circuit(s: &str) -> String {
    log(&format!("S: {}", s));
    let public_inputs = parse_string(s);
    let message: [Fr; WASM_POSEIDON_L] = public_inputs
        .x
        .iter()
        .map(|k| Fr::from(*k))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let res: String = format!(
        "{:?}",
        wasm_poseidon_solution::<
            PoseidonSpec<WASM_POSEIDON_WIDTH, WASM_POSEIDON_RATE>,
            WASM_POSEIDON_WIDTH,
            WASM_POSEIDON_RATE,
            WASM_POSEIDON_L,
        >(message)
    );
    res
}

pub fn wasm_poseidon_solution<S, const WIDTH: usize, const RATE: usize, const L: usize>(
    message: [Fr; L],
) -> Fr
where
    S: Spec<Fr, WIDTH, RATE> + Copy + Clone,
{
    poseidon::Hash::<_, S, ConstantLength<L>, WIDTH, RATE>::init().hash(message)
}

#[cfg(test)]
mod test {
    use halo2_proofs::halo2curves::bn256;
    use rand_core::OsRng;

    use crate::utils::{
        generate_keys, generate_params, generate_proof_with_instance, verify_with_instance,
    };

    use super::*;

    fn bench<S, const WIDTH: usize, const RATE: usize, const L: usize>(
        name: &str,
    ) -> Result<(), Error>
    where
        S: Spec<Fr, WIDTH, RATE> + Copy + Clone,
    {
        let params = generate_params(K);
        let empty_circuit = PoseidonCircuit::<S, WIDTH, RATE, L> {
            message: Value::unknown(),
            _spec: PhantomData,
        };

        let (pk, vk) = generate_keys(&params, &empty_circuit);

        let prover_name = format!("{}-prover", &name);
        let verfier_name = format!("{}-verifier", &name);

        let mut rng = OsRng;
        let message: [Fr; L] = (0..L)
            .map(|_| bn256::Fr::random(rng))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let output = poseidon::Hash::<_, S, ConstantLength<L>, WIDTH, RATE>::init().hash(message);

        let circuit = PoseidonCircuit::<S, WIDTH, RATE, L> {
            message: Value::known(message),
            _spec: PhantomData,
        };

        let proof = generate_proof_with_instance(&params, &pk, circuit, &[output]);
        verify_with_instance(&params, &pk, &proof, &[output])
    }

    #[test]
    fn poseidon_test() {
        assert_eq!(
            (),
            bench::<PoseidonSpec<3, 2>, 3, 2, 2>("WIDTH = 3, RATE = 2").unwrap()
        );
    }

    #[test]
    fn parse_test() {
        let res: Fr = PrimeField::from_str_vartime(
            "9580880353492959643917950575845981964081484910446475371419582752754603137303",
        )
        .unwrap();
        println!("{:?}", res);
    }
}
