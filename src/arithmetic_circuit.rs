use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::Field,
    circuit::{Cell, Layouter, SimpleFloorPlanner, Value},
    halo2curves::pasta::Fp,
    plonk::{
        keygen_pk, keygen_vk, Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Fixed,
        FloorPlanner, Instance, ProvingKey, VerifyingKey,
    },
    poly::{commitment::Params, Rotation},
};

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

pub mod draw {
    use super::*;
    use halo2_proofs::{circuit::Value, dev::CircuitGates, halo2curves::pasta::pallas};

    pub fn draw_graph() {
        // Prepare the circuit you want to render.
        // You don't need to include any witness variables.
        let k = 4;
        let constant = Fp::from(7);
        let x = Fp::from(6);
        let y = Fp::from(9);
        let circuit: ArithmeticCircuit<Fp> = ArithmeticCircuit {
            x: Value::known(x),
            y: Value::known(y),
            constant: constant,
        };

        // Create the area you want to draw on.
        // Use SVGBackend if you want to render to .svg instead.
        use plotters::prelude::*;
        let root = SVGBackend::new("img/layout.svg", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Arithmetic Circuit Layout", ("sans-serif", 30))
            .unwrap();

        // halo2_proofs::dev::CircuitLayout::default()
        //     // You can optionally render only a section of the circuit.
        //     // You can hide labels, which can be useful with smaller areas.
        //     .show_labels(true)
        //     // Render the circuit onto your area!
        //     // The first argument is the size parameter for the circuit.
        //     .render(5, &circuit, &root)
        //     .unwrap();

        halo2_proofs::dev::CircuitLayout::default()
            // .show_equality_constraints(true)
            .view_width(0..2)
            .view_height(0..16)
            .show_labels(true)
            .render(5, &circuit, &root)
            .unwrap();

        // Generate the DOT graph string.
        let dot_string = halo2_proofs::dev::circuit_dot_graph(&circuit);
        println!("GRAPH: {}", dot_string);
        let gates = CircuitGates::collect::<pallas::Base, ArithmeticCircuit<Fp>>();
        println!("{}", gates);
    }
}

#[cfg(test)]
mod test {
    use super::ArithmeticCircuit;
    use halo2_proofs::arithmetic::Field;
    use halo2_proofs::circuit::Value;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::pasta::Fp;
    // use halo2_proofs::pasta::Fp;

    #[test]
    fn test() {
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
    }
}
