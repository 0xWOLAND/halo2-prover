use halo2_proofs::halo2curves::bn256::Fr;
use halo2_proofs::halo2curves::ff::{Field, PrimeField};
use halo2_proofs::halo2curves::pasta::{pallas, vesta, EqAffine};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column,
        ConstraintSystem, Error, Instance,
    },
    poly::{
        commitment::ParamsProver,
        ipa::{
            commitment::{IPACommitmentScheme, ParamsIPA},
            multiopen::ProverIPA,
            strategy::SingleStrategy,
        },
        VerificationStrategy,
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};

use halo2_gadgets::poseidon::{
    primitives::{self as poseidon, generate_constants, ConstantLength, Mds, Spec},
    Hash, Pow5Chip, Pow5Config,
};
use std::convert::TryInto;
use std::marker::PhantomData;

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

static N_ROUNDS_F: i32 = 8;
static N_ROUNDS_P: [i32; 16] = [
    56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
];

use crate::{constants::constants, unstringify::unstringifyHex};
#[derive(Copy, Clone)]
pub struct PoseidonCircuit<S, const WIDTH: usize, const RATE: usize, const L: usize>
where
    S: Spec<Fr, WIDTH, RATE> + Clone + Copy,
{
    message: Value<[Fr; L]>,
    _spec: PhantomData<S>,
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
struct PoseidonSpec<const WIDTH: usize, const RATE: usize>;

impl<const WIDTH: usize, const RATE: usize> Spec<Fr, WIDTH, RATE> for PoseidonSpec<WIDTH, RATE> {
    fn full_rounds() -> usize {
        8
    }

    fn partial_rounds() -> usize {
        56
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
    let nRoundsF = N_ROUNDS_F;
    let nRoundsP = N_ROUNDS_P[0];
    let t = inputs.len() + 1;

    let (C, M) = constants();
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
    for x in 0..(nRoundsF + nRoundsP) {
        for y in 0..state.len() {
            state[y] += C[(x as usize) * t + y];
            if (x < nRoundsF / 2 || x >= nRoundsF / 2 + nRoundsP) {
                state[y] = sbox(state[y]);
            } else if (y == 0) {
                state[y] = sbox(state[y]);
            }
        }
        state = mix(state, &M);
    }

    Ok(state[0])
}

#[cfg(test)]
mod test {
    use halo2_proofs::{halo2curves::bn256, transcript};

    use crate::utils::{
        generate_params, generate_proof, generate_proof_with_instance, verify_with_instance,
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

        let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
        let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

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
        // let inputs = vec![Fr::ZERO];
        // assert!(poseidon(inputs).unwrap() > Fr::ZERO);
        assert_eq!(
            (),
            bench::<PoseidonSpec<3, 2>, 3, 2, 2>("WIDTH = 3, RATE = 2").unwrap()
        );
    }
}
