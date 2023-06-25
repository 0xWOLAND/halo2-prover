use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Cell, Layouter, SimpleFloorPlanner, Value},
    plonk::{
        Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Expression, Fixed, Instance,
        Selector,
    },
    poly::Rotation,
};

#[derive(Debug, Clone)]
pub struct CollatzConfig {
    pub x: Column<Advice>,
    pub y: Column<Advice>,

    pub se: Selector,
    pub so: Selector,

    pub pi: Column<Instance>,
}

pub struct CollatzChip<F: FieldExt> {
    config: CollatzConfig,
    marker: PhantomData<F>,
}

impl<F: FieldExt> CollatzChip<F> {
    pub fn new(config: CollatzConfig) -> Self {
        Self {
            config,
            marker: PhantomData,
        }
    }

    fn apply_function<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
        isEven: bool,
    ) -> Result<(Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>)>,
    {
        layouter.assign_region(
            || "handle_even",
            |mut region| {
                let values = Some(f());

                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.x,
                    0,
                    || values.unwrap().map(|v| v.0),
                )?;

                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.y,
                    0,
                    || values.unwrap().map(|v| v.1),
                )?;

                match isEven {
                    true => {
                        self.config.se.enable(&mut region, 0)?;
                    }
                    _ => {
                        self.config.so.enable(&mut region, 0)?;
                    }
                }

                Ok((lhs.cell(), rhs.cell()))
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
        layouter.constrain_instance(cell, self.config.pi, row)
    }
}

#[derive(Default)]
pub struct CollatzCircuit<F: FieldExt> {
    pub x: Vec<Value<F>>,
}

impl<F: FieldExt> Circuit<F> for CollatzCircuit<F> {
    type Config = CollatzConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let x = meta.advice_column();
        let y = meta.advice_column();

        meta.enable_equality(x);
        meta.enable_equality(y);

        let se = meta.selector();
        let so = meta.selector();

        let pi = meta.instance_column();
        meta.enable_equality(pi);

        meta.create_gate("check even", |meta| {
            let x = meta.query_advice(x, Rotation::cur());
            let y = meta.query_advice(y, Rotation::cur());

            let se = meta.query_selector(se);

            vec![se * (x - Expression::Constant(F::from(2)) * y)]
        });

        meta.create_gate("check odd", |meta| {
            let x = meta.query_advice(x, Rotation::cur());
            let y = meta.query_advice(y, Rotation::cur());

            let so = meta.query_selector(so);

            vec![so * (Expression::Constant(F::from(3)) * x + Expression::Constant(F::from(1)) - y)]
        });

        CollatzConfig { x, y, se, so, pi }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let cs = CollatzChip::new(config);

        let arr: Vec<Value<Assigned<_>>> = self
            .x
            .iter()
            .map(|k| Into::<Value<Assigned<_>>>::into(*k))
            .collect();

        for x in arr {
            let consta = Assigned::from(F::from(3));
            let constb = Assigned::from(F::from(2));
            let constc = Assigned::from(F::from(1));

            let (x0, y0) =
                cs.apply_function(&mut layouter, || x.map(|x| (x, consta * x + constc)), false)?;

            cs.expose_public(&mut layouter, y0, 0);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use halo2_proofs::circuit::Value;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::pasta::Fp;

    use crate::collatz::CollatzCircuit;
    // use halo2_proofs::pasta::Fp;

    fn collatz(mut n: i32) -> Vec<i32> {
        let mut ans = Vec::new();
        ans.push(n);

        while (n >= 1) {
            if (n & 1 > 0) {
                n *= 3 + 1;
            } else {
                n /= 2;
            }
            ans.push(n);
        }

        ans
    }

    #[test]
    fn test() {
        let k = 4;
        let x = Fp::from(5);
        let y = Fp::from(16);
        println!("{:?}", collatz(16));

        let circuit: CollatzCircuit<Fp> = CollatzCircuit {
            x: vec![Value::known(x)],
        };

        let mut public_inputs = vec![y];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}
