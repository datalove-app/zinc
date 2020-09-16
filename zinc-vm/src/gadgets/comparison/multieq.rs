use algebra::{FpParameters, PrimeField};
use r1cs_core::{ConstraintSystem, LinearCombination, SynthesisError, Variable};

pub struct MultiEq<F: PrimeField, CS: ConstraintSystem<F>> {
    cs: CS,
    ops: usize,
    bits_used: usize,
    lhs: LinearCombination<F>,
    rhs: LinearCombination<F>,
}

impl<F: PrimeField, CS: ConstraintSystem<F>> MultiEq<F, CS> {
    pub fn enforce_equal(
        &mut self,
        num_bits: usize,
        lhs: &LinearCombination<F>,
        rhs: &LinearCombination<F>,
    ) {
        // Check if we will exceed the capacity
        let capacity = F::Params::CAPACITY as usize;
        if capacity <= (self.bits_used + num_bits) {
            self.accumulate();
        }

        assert!(capacity > (self.bits_used + num_bits));

        // FIXME
        let coeff = F::from_str("2").ok().unwrap().pow(&[self.bits_used as u64]);
        self.lhs = self.lhs.clone() + (coeff, lhs);
        self.rhs = self.rhs.clone() + (coeff, rhs);
        self.bits_used += num_bits;
    }

    fn accumulate(&mut self) {
        let ops = self.ops;
        let lhs = self.lhs.clone();
        let rhs = self.rhs.clone();
        self.cs.enforce(
            || format!("multieq {}", ops),
            |_| lhs,
            |lc| lc + CS::one(),
            |_| rhs,
        );
        self.lhs = LinearCombination::zero();
        self.rhs = LinearCombination::zero();
        self.bits_used = 0;
        self.ops += 1;
    }
}

impl<F: PrimeField, CS: ConstraintSystem<F>> Drop for MultiEq<F, CS> {
    fn drop(&mut self) {
        if self.bits_used > 0 {
            self.accumulate();
        }
    }
}

impl<F: PrimeField, CS: ConstraintSystem<F>> From<CS> for MultiEq<F, CS> {
    fn from(cs: CS) -> Self {
        MultiEq {
            cs,
            ops: 0,
            bits_used: 0,
            lhs: LinearCombination::zero(),
            rhs: LinearCombination::zero(),
        }
    }
}

impl<F: PrimeField, CS: ConstraintSystem<F>> ConstraintSystem<F> for MultiEq<F, CS> {
    type Root = Self;

    fn one() -> Variable {
        CS::one()
    }

    fn alloc<FF, A, AR>(&mut self, annotation: A, f: FF) -> Result<Variable, SynthesisError>
    where
        FF: FnOnce() -> Result<F, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.cs.alloc(annotation, f)
    }

    fn alloc_input<FF, A, AR>(&mut self, annotation: A, f: FF) -> Result<Variable, SynthesisError>
    where
        FF: FnOnce() -> Result<F, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.cs.alloc_input(annotation, f)
    }

    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
        LA: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
        LB: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
        LC: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
    {
        self.cs.enforce(annotation, a, b, c)
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.cs.get_root().push_namespace(name_fn)
    }

    fn pop_namespace(&mut self) {
        self.cs.get_root().pop_namespace()
    }

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn num_constraints(&self) -> usize {
        todo!()
    }
}
