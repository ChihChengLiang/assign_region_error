fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use halo2::{
        circuit::{Layouter, Region, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Selector},
        poly::Rotation,
    };
    use pasta_curves::arithmetic::FieldExt;
    use pasta_curves::pallas;
    use std::marker::PhantomData;

    #[test]
    fn test_failing_assign_region() {
        #[derive(Clone)]
        struct DummyConfig<F> {
            q_enable: Selector,
            a: Column<Advice>,
            _marker: PhantomData<F>,
        }
        impl<F: FieldExt> DummyConfig<F> {
            fn configure(
                meta: &mut ConstraintSystem<F>,
                q_enable: Selector,
                a: Column<Advice>,
            ) -> Self {
                meta.create_gate("a is one", |meta| {
                    let q = meta.query_selector(q_enable);
                    let a = meta.query_advice(a, Rotation::cur());
                    let one = Expression::Constant(F::one());
                    vec![("check a", q * (a - one))]
                });
                Self {
                    q_enable,
                    a,
                    _marker: PhantomData,
                }
            }

            fn assign(
                &self,
                region: &mut Region<'_, F>,
                offset: usize,
                dummy: Option<F>,
            ) -> Result<(), Error> {
                self.q_enable.enable(region, offset)?;
                region.assign_advice(
                    || "a",
                    self.a,
                    offset,
                    || dummy.ok_or(Error::SynthesisError),
                )?;
                Ok(())
            }
        }

        #[derive(Default)]
        struct MyCircuit<F> {
            a: Option<F>,
        }
        impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
            type Config = DummyConfig<F>;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self::default()
            }

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let a = meta.advice_column();
                let q_enable = meta.selector();
                DummyConfig::configure(meta, q_enable, a)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<F>,
            ) -> Result<(), Error> {
                layouter.assign_region(
                    || "assign a",
                    |mut region| {
                        let offset = 0;
                        config.assign(&mut region, offset, self.a)?;
                        Ok(())
                    },
                )?;
                Ok(())
            }
        }

        let circuit = MyCircuit::<pallas::Base> {
            a: Some(pallas::Base::one()),
        };
        let prover = MockProver::<pallas::Base>::run(3, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()));

        let circuit = MyCircuit::<pallas::Base> { a: None };
        let prover = MockProver::<pallas::Base>::run(3, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}
