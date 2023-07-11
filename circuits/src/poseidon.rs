use std::{error::Error, marker::PhantomData};

use halo2_proofs::{
    arithmetic::Field,
    circuit::{SimpleFloorPlanner, Value},
    halo2curves::{bn256::Fr, ff::PrimeField},
    plonk::{Advice, Circuit, Column, Selector},
    *,
};

static N_ROUNDS_F: i32 = 8;
static N_ROUNDS_P: [i32; 16] = [
    56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
];

use crate::{constants::constants, unstringify::unstringifyHex};

#[derive(Debug, Clone)]
pub struct PoseidonConfig {
    witness: Column<Advice>,
    selector: Selector,
}

pub struct PoseidonChip<F: Field> {
    config: PoseidonConfig,
    marker: PhantomData<F>,
}
impl<F: Field> PoseidonChip<F> {
    fn new(config: PoseidonConfig) -> Self {
        Self {
            config,
            marker: PhantomData,
        }
    }
}

#[derive(Default)]
pub struct PoseidonCircuit<F: Field> {
    pub x: Value<F>,
}

impl<F: Field> Circuit<F> for PoseidonCircuit<F> {
    type Config = PoseidonConfig;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut plonk::ConstraintSystem<F>) -> Self::Config {
        todo!()
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl circuit::Layouter<F>,
    ) -> Result<(), plonk::Error> {
        todo!()
    }
}

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
    use halo2_proofs::{arithmetic::Field, halo2curves::bn256::Fr};

    use super::poseidon;

    #[test]
    fn poseidon_test() {
        let inputs = vec![Fr::ZERO];
        assert!(poseidon(inputs).unwrap() > Fr::ZERO);
    }
}
