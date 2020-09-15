use algebra::Field;
use r1cs_core::{ConstraintSystem, Index, LinearCombination, SynthesisError, Variable};

pub struct DebugConstraintSystem<F: Field> {
    inputs: Vec<F>,
    witness: Vec<F>,

    satisfied: bool,
    constraints_num: usize,
}

impl<F: Field> Default for DebugConstraintSystem<F> {
    fn default() -> Self {
        let mut cs = Self {
            inputs: Vec::new(),
            witness: Vec::new(),
            satisfied: true,
            constraints_num: 0,
        };

        cs.inputs.push(F::one());
        cs
    }
}

impl<F: Field> DebugConstraintSystem<F> {
    pub fn is_satisfied(&self) -> bool {
        self.satisfied
    }

    pub fn num_constraints(&self) -> usize {
        self.constraints_num
    }
}

impl<F: Field> ConstraintSystem<F> for DebugConstraintSystem<F> {
    type Root = Self;

    fn alloc<FF, A, AR>(&mut self, _annotation: A, f: FF) -> Result<Variable, SynthesisError>
    where
        FF: FnOnce() -> Result<F, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        let value = f()?;
        self.witness.push(value);
        Ok(Variable::new_unchecked(Index::Aux(self.witness.len() - 1)))
    }

    fn alloc_input<FF, A, AR>(&mut self, _annotation: A, f: FF) -> Result<Variable, SynthesisError>
    where
        FF: FnOnce() -> Result<F, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        let value = f()?;
        self.inputs.push(value);
        Ok(Variable::new_unchecked(Index::Input(self.inputs.len() - 1)))
    }

    fn enforce<A, AR, LA, LB, LC>(&mut self, _annotation: A, a: LA, b: LB, c: LC)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
        LA: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
        LB: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
        LC: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
    {
        let zero = LinearCombination::zero();
        let value_a = eval_lc::<F>(a(zero.clone()).as_ref(), &self.inputs, &self.witness);
        let value_b = eval_lc::<F>(b(zero.clone()).as_ref(), &self.inputs, &self.witness);
        let value_c = eval_lc::<F>(c(zero).as_ref(), &self.inputs, &self.witness);

        let value_ab = {
            let mut tmp: F = value_a;
            tmp.mul_assign(&value_b);
            tmp
        };

        if value_ab != value_c {
            self.satisfied = false;
        }

        self.constraints_num += 1;
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

fn eval_lc<F: Field>(terms: &[(Variable, F)], inputs: &[F], witness: &[F]) -> F {
    let mut acc = F::zero();

    for &(var, ref coeff) in terms {
        let mut tmp = match var.get_unchecked() {
            Index::Input(index) => inputs[index],
            Index::Aux(index) => witness[index],
        };

        tmp.mul_assign(coeff);
        acc.add_assign(&tmp);
    }

    acc
}
