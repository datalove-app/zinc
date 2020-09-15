use algebra::Field;
use r1cs_core::{ConstraintSystem, Index, LinearCombination, SynthesisError, Variable};

pub struct ConstantCS;

impl<F: Field> ConstraintSystem<F> for ConstantCS {
    type Root = Self;

    fn alloc<FF, A, AR>(&mut self, _annotation: A, f: FF) -> Result<Variable, SynthesisError>
    where
        FF: FnOnce() -> Result<F, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        f()?;
        Ok(<Self as ConstraintSystem<F>>::one())
    }

    fn alloc_input<FF, A, AR>(&mut self, _annotation: A, f: FF) -> Result<Variable, SynthesisError>
    where
        FF: FnOnce() -> Result<F, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        f()?;
        Ok(<Self as ConstraintSystem<F>>::one())
    }

    fn enforce<A, AR, LA, LB, LC>(&mut self, _annotation: A, _a: LA, _b: LB, _c: LC)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
        LA: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
        LB: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
        LC: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
    {
    }

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
    }

    fn pop_namespace(&mut self) {}

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn num_constraints(&self) -> usize {
        todo!()
    }
}
