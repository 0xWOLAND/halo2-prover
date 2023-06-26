use std::marker::PhantomData;

// use crate::utils
use halo2_proofs::{
    circuit::{AssignedCell, Cell, Layouter, SimpleFloorPlanner, Value},
    halo2curves::FieldExt,
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

#[derive(Clone, Copy, Debug)]
pub struct CollatzConfig {
    witness: Column<Advice>,
    // Normally, you would use `Selector` instead
    is_odd: Column<Advice>,
    is_one: Column<Advice>,
    selector: Selector,
    final_entry: Selector,
}

impl CollatzConfig {
    pub fn configure<F: FieldExt>(meta: &mut ConstraintSystem<F>) -> Self {
        // create witness column
        let witness = meta.advice_column();
        let is_odd = meta.advice_column();
        let is_one = meta.advice_column();
        let final_entry = meta.selector();
        let selector = meta.selector();

        meta.enable_equality(witness);

        meta.create_gate("is_even", |meta| {
            let x = meta.query_advice(witness, Rotation::cur());
            let y = meta.query_advice(witness, Rotation::next());

            let is_odd = meta.query_advice(is_odd, Rotation::cur());
            let sel = meta.query_selector(selector);

            vec![
                sel * ((Expression::Constant(F::one()) - is_odd)
                    * (x - Expression::Constant(F::from(2)) * y)),
            ]
        });

        meta.create_gate("is_odd", |meta| {
            let x = meta.query_advice(witness, Rotation::cur());
            let y = meta.query_advice(witness, Rotation::next());

            let is_odd = meta.query_advice(is_odd, Rotation::cur());
            let is_one = meta.query_advice(is_one, Rotation::cur());
            let sel = meta.query_selector(selector);

            vec![
                sel * (Expression::Constant(F::one()) - is_one)
                    * (is_odd
                        * (Expression::Constant(F::from(3)) * x
                            + Expression::Constant(F::from(1))
                            - y)),
            ]
        });

        meta.create_gate("is_one", |meta| {
            let x = meta.query_advice(witness, Rotation::cur());
            let y = meta.query_advice(witness, Rotation::next());

            let is_one = meta.query_advice(is_one, Rotation::cur());
            let sel = meta.query_selector(selector);
            vec![sel * is_one * ((x.clone() - y) + (x.clone() - Expression::Constant(F::one())))]
        });

        meta.create_gate("final_element", |meta| {
            let x = meta.query_advice(witness, Rotation::cur());
            let sel = meta.query_selector(final_entry);
            vec![sel * (Expression::Constant(F::from(1)) - x)]
        });

        Self {
            witness,
            is_odd,
            is_one,
            selector,
            final_entry,
        }
    }
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

    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        row: usize,
        entry: Value<Assigned<F>>,
        next: Value<Assigned<F>>,
        is_odd: Value<Assigned<F>>,
        is_one: Value<Assigned<F>>,
    ) -> Result<
        (
            AssignedCell<Assigned<F>, F>,
            AssignedCell<Assigned<F>, F>,
            AssignedCell<Assigned<F>, F>,
        ),
        halo2_proofs::plonk::Error,
    > {
        layouter.assign_region(
            || "initial entry",
            |mut region| {
                self.config.selector.enable(&mut region, row)?;

                let x = region.assign_advice(|| "x", self.config.witness, row, || entry)?;
                let y = region.assign_advice(|| "y", self.config.witness, row + 1, || next)?;
                let a: Value<Assigned<F>> = Value::known(F::from(2)).into();
                println!("{:?} -> {:?}", entry, is_odd);

                let is_odd_cell =
                    region.assign_advice(|| "sel", self.config.is_odd, row, || is_odd)?;
                let is_one_cell =
                    region.assign_advice(|| "sel", self.config.is_one, row, || is_one)?;

                Ok((x, y, is_odd_cell))
            },
        )
    }

    fn assign_last(
        &mut self,
        mut layouter: impl Layouter<F>,
        row: usize,
        entry: Value<Assigned<F>>,
    ) -> Result<Cell, Error> {
        layouter.assign_region(
            || "final output",
            |mut region| {
                let a = region.assign_advice(|| "out", self.config.witness, row, || entry)?;
                let _ = self.config.final_entry.enable(&mut region, row);

                Ok(a.cell())
            },
        )
    }
}

#[derive(Clone, Default)]
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
        CollatzConfig::configure::<F>(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let mut chip: CollatzChip<F> = CollatzChip::new(config);
        let nrows = self.x.len();
        let one: Value<Assigned<F>> = Value::known(F::one()).into();

        for row in 0..(nrows - 1) {
            let s = format!("Collatz({})", row);
            let is_odd: Value<Assigned<F>> = self.x[row]
                .map(|k| F::from(k.is_odd().unwrap_u8() as u64))
                .into();

            let is_one: Value<Assigned<F>> = self.x[row]
                .map(|k| F::from((k - F::one()).is_zero().unwrap_u8() as u64))
                .into();

            let (contents, next, is_odd) = chip.assign(
                layouter.namespace(|| s),
                row,
                self.x[row].into(),
                self.x[row + 1].into(),
                is_odd,
                is_one,
            )?;
            // println!(
            // "cell: {:?} \n next: {:?} \nis odd: {:?}\n",
            // contents.value(),
            // next.value(),
            // is_odd.value()
            // );
            // println!("-------------------------------");
        }

        let out_cell = chip.assign_last(
            layouter.namespace(|| "out"),
            nrows - 1,
            self.x[nrows - 1].into(),
        )?;
        println!("out cell: {:?}", self.x[nrows - 1]);

        Ok(())
    }
}
pub fn collatz_conjecture(mut n: u64) -> Vec<u64> {
    let mut ans = Vec::new();
    ans.push(n);

    while n > 1 {
        if n & 1 > 0 {
            n = 3 * n + 1;
        } else {
            n /= 2;
        }
        ans.push(n);
    }
    for _ in 1..4 {
        ans.push(1);
    }
    ans
}

#[cfg(test)]
mod test {
    use halo2_proofs::{circuit::Value, dev::MockProver, halo2curves::pasta::Fp};

    use super::CollatzCircuit;

    #[test]
    fn test_collatz() {
        let k = 8;
        let x: Vec<Value<_>> = super::collatz_conjecture(7)
            .iter()
            .map(|y: &u64| Value::known(Fp::from(*y)))
            .collect();

        let circuit = CollatzCircuit { x };

        MockProver::run(k, &circuit, vec![])
            .unwrap()
            .assert_satisfied();
    }
}
