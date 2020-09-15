use algebra::Field;
use r1cs_core::{ConstraintSystem, Index, LinearCombination, SynthesisError, Variable};
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub struct DuplicateRemovingCS<F, CS>(CS, PhantomData<F>)
where
    F: Field,
    CS: ConstraintSystem<F>;

impl<F, CS> DuplicateRemovingCS<F, CS>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    pub fn new(cs: CS) -> Self {
        Self(cs, PhantomData)
    }

    pub fn inner(&self) -> &CS {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut CS {
        &mut self.0
    }

    pub fn into_inner(self) -> CS {
        self.0
    }
}

impl<F, CS> ConstraintSystem<F> for DuplicateRemovingCS<F, CS>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    type Root = Self;

    fn alloc<FF, A, AR>(&mut self, annotation: A, f: FF) -> Result<Variable, SynthesisError>
    where
        FF: FnOnce() -> Result<F, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.0.alloc(annotation, f)
    }

    fn alloc_input<FF, A, AR>(&mut self, annotation: A, f: FF) -> Result<Variable, SynthesisError>
    where
        FF: FnOnce() -> Result<F, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.0.alloc_input(annotation, f)
    }

    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
        LA: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
        LB: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
        LC: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
    {
        self.0.enforce(
            annotation,
            |zero| remove_duplicates(a(zero)),
            |zero| remove_duplicates(b(zero)),
            |zero| remove_duplicates(c(zero)),
        )
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.0.get_root().push_namespace(name_fn);
    }

    fn pop_namespace(&mut self) {
        self.0.get_root().pop_namespace();
    }

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn num_constraints(&self) -> usize {
        todo!()
    }
}

fn remove_duplicates<F: Field>(lc: LinearCombination<F>) -> LinearCombination<F> {
    let mut inputs_map = BTreeMap::<usize, F>::new();
    let mut aux_map = BTreeMap::<usize, F>::new();

    let zero = F::zero();
    for (var, c) in lc.as_ref() {
        match var.get_unchecked() {
            Index::Input(i) => {
                let mut tmp = *inputs_map.get(&i).unwrap_or(&zero);
                tmp.add_assign(c);
                inputs_map.insert(i, tmp);
            }
            Index::Aux(i) => {
                let mut tmp = *aux_map.get(&i).unwrap_or(&zero);
                tmp.add_assign(c);
                aux_map.insert(i, tmp);
            }
        }
    }

    let mut lc = LinearCombination::zero();

    for (i, c) in inputs_map.into_iter() {
        if c.is_zero() {
            continue;
        }
        let var = Variable::new_unchecked(Index::Input(i));
        lc = lc + (c, var);
    }

    for (i, c) in aux_map.into_iter() {
        if c.is_zero() {
            continue;
        }
        let var = Variable::new_unchecked(Index::Aux(i));
        lc = lc + (c, var);
    }

    lc
}
