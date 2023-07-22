use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::Field,
    circuit::{Cell, Layouter, SimpleFloorPlanner, Value},
    halo2curves::bn256::Fr,
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};
use serde::{Deserialize, Serialize};

trait ArithmeticInstructions<F: Field> {
    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error>;

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error>;
}

#[derive(Serialize, Deserialize)]
pub struct ArithmeticInput {
    pub x: u64,
    pub y: u64,
    pub constant: u64,
    pub z: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ArithmeticConfig {
    pub l: Column<Advice>,
    pub r: Column<Advice>,
    pub o: Column<Advice>,
    pub sl: Column<Fixed>,
    pub sr: Column<Fixed>,
    pub so: Column<Fixed>,
    pub sm: Column<Fixed>,
    pub sc: Column<Fixed>,
    pub PI: Column<Instance>,
}

pub struct ArithmeticChip<F: Field> {
    config: ArithmeticConfig,
    marker: PhantomData<F>,
}

impl<F: Field> ArithmeticChip<F> {
    fn new(config: ArithmeticConfig) -> Self {
        Self {
            config,
            marker: PhantomData,
        }
    }
}

impl<F: Field> ArithmeticInstructions<F> for ArithmeticChip<F> {
    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>,
    {
        layouter.assign_region(
            || "mul",
            |mut region| {
                let values = Some(f());
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || values.unwrap().map(|v| v.0),
                )?;

                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|v| v.1),
                )?;

                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|v| v.2),
                )?;

                region.assign_fixed(|| "m", self.config.sm, 0, || Value::known(F::ONE))?;
                region.assign_fixed(|| "o", self.config.so, 0, || Value::known(F::ONE))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>,
    {
        layouter.assign_region(
            || "add",
            |mut region| {
                let values = Some(f());

                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || values.unwrap().map(|v| v.0),
                )?;

                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|v| v.1),
                )?;
                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|v| v.2),
                )?;

                region.assign_fixed(|| "l", self.config.sl, 0, || Value::known(F::ONE))?;
                region.assign_fixed(|| "r", self.config.sr, 0, || Value::known(F::ONE))?;
                region.assign_fixed(|| "o", self.config.so, 0, || Value::known(F::ONE))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error> {
        layouter.assign_region(|| "copy", |mut region| region.constrain_equal(a, b))
    }

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell, self.config.PI, row)
    }
}

#[derive(Default)]
pub struct ArithmeticCircuit<F: Field> {
    pub x: Value<F>,
    pub y: Value<F>,
    pub constant: F,
}

impl<F: Field> Circuit<F> for ArithmeticCircuit<F> {
    type Config = ArithmeticConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let l = meta.advice_column();
        let r = meta.advice_column();
        let o = meta.advice_column();

        meta.enable_equality(l);
        meta.enable_equality(r);
        meta.enable_equality(o);

        let sm = meta.fixed_column();
        let sl = meta.fixed_column();
        let sr = meta.fixed_column();
        let so = meta.fixed_column();
        let sc = meta.fixed_column();

        let PI = meta.instance_column();
        meta.enable_equality(PI);

        meta.create_gate("plonk", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            let r = meta.query_advice(r, Rotation::cur());
            let o = meta.query_advice(o, Rotation::cur());

            let sl = meta.query_fixed(sl, Rotation::cur());
            let sr = meta.query_fixed(sr, Rotation::cur());
            let so = meta.query_fixed(so, Rotation::cur());
            let sm = meta.query_fixed(sm, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());

            vec![l.clone() * sl + r.clone() * sr + l * r * sm + (o * so * (-F::ONE)) + sc]
        });

        ArithmeticConfig {
            l,
            r,
            o,
            sl,
            sr,
            so,
            sm,
            sc,
            PI,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let cs = ArithmeticChip::new(config);

        let x: Value<Assigned<_>> = self.x.into();
        let y: Value<Assigned<_>> = self.y.into();
        let consty = Assigned::from(self.constant);

        let (a0, b0, c0) = cs.raw_multiply(&mut layouter, || x.map(|x| (x, x, x * x)))?;
        cs.copy(&mut layouter, a0, b0)?;

        let (a1, b1, c1) = cs.raw_multiply(&mut layouter, || y.map(|y| (y, y, y * y)))?;
        cs.copy(&mut layouter, a1, b1)?;

        let (a2, b2, c2) = cs.raw_multiply(&mut layouter, || {
            x.zip(y).map(|(x, y)| (x * x, y * y, (x * x) * (y * y)))
        })?;

        cs.copy(&mut layouter, c0, a2)?;
        cs.copy(&mut layouter, c1, b2)?;

        let (a3, b3, c3) = cs.raw_add(&mut layouter, || {
            x.zip(y)
                .map(|(x, y)| ((x * x) * (y * y), consty, (x * x) * (y * y) + consty))
        })?;

        cs.copy(&mut layouter, c2, a3)?;

        cs.expose_public(&mut layouter, b3, 0)?;
        cs.expose_public(&mut layouter, c3, 1)?;

        Ok(())
    }
}

pub fn create_circuit(x: u64, y: u64, constant: u64) -> ArithmeticCircuit<Fr> {
    let x = Value::known(Fr::from(x));
    let y = Value::known(Fr::from(y));
    let constant = Fr::from(constant);

    ArithmeticCircuit { x, y, constant }
}

pub fn empty_circuit(constant: u64) -> ArithmeticCircuit<Fr> {
    ArithmeticCircuit {
        x: Value::unknown(),
        y: Value::unknown(),
        constant: Fr::from(constant),
    }
}

pub fn parse_string(s: &str) -> ArithmeticInput {
    serde_json::from_str(s).unwrap()
}

pub fn create_circuit_from_string(s: &str) -> ArithmeticCircuit<Fr> {
    let v = parse_string(s);
    let x = v.x;
    let y = v.y;
    let constant = v.constant;
    create_circuit(x, y, constant)
}

pub fn simulate_circuit(s: &str) -> String {
    let inp = parse_string(s);
    ((inp.x * inp.x) * (inp.y * inp.y) + inp.constant).to_string()
}

#[cfg(test)]
mod test {
    use super::ArithmeticCircuit;
    use crate::arithmetic_circuit::{create_circuit, empty_circuit};
    use crate::utils::{generate_keys, generate_proof_with_instance, verify_with_instance};
    use halo2_proofs::circuit::Value;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::{Bn256, Fr};
    use halo2_proofs::poly::commitment::ParamsProver;
    use halo2_proofs::poly::kzg::commitment::ParamsKZG;

    #[test]
    fn test() {
        let k = 4;
        let constant = Fr::from(7);
        let x = Fr::from(6);
        let y = Fr::from(9);
        let z = Fr::from(36 * 81 + 7);

        let circuit: ArithmeticCircuit<Fr> = ArithmeticCircuit {
            x: Value::known(x),
            y: Value::known(y),
            constant: constant,
        };

        let mut public_inputs = vec![constant, z];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }

    #[test]
    fn test_ecc() {
        let k = 8;
        let constant = 7;
        let x = 6;
        let y = 9;
        let z = Fr::from(36 * 81 + 7);
        let circuit = create_circuit(x, y, constant);
        let public_input = [Fr::from(constant), z];

        let params = ParamsKZG::<Bn256>::new(k);

        let empty_circuit: ArithmeticCircuit<Fr> = empty_circuit(constant);
        let (pk, vk) = generate_keys(&params, &empty_circuit);
        let proof = generate_proof_with_instance(&params, &pk, circuit, &public_input);

        let is_valid = verify_with_instance(&params, &pk, &proof, &public_input).unwrap();
        assert_eq!(is_valid, ());
    }
}
