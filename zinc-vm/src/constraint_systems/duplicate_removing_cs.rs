use crate::Engine;
use algebra::Zero;
use r1cs_core::{ConstraintSystem, Index, LinearCombination, SynthesisError, Variable};
use std::{collections::BTreeMap, marker::PhantomData, ops::AddAssign};

pub struct DuplicateRemovingCS<E, CS>(CS, PhantomData<E>)
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>;

impl<E, CS> DuplicateRemovingCS<E, CS>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
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

impl<E, CS> ConstraintSystem<E::Fr> for DuplicateRemovingCS<E, CS>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    type Root = Self;

    fn alloc<F, A, AR>(&mut self, annotation: A, f: F) -> Result<Variable, SynthesisError>
    where
        F: FnOnce() -> Result<E::Fr, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.0.alloc(annotation, f)
    }

    fn alloc_input<F, A, AR>(&mut self, annotation: A, f: F) -> Result<Variable, SynthesisError>
    where
        F: FnOnce() -> Result<E::Fr, SynthesisError>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.0.alloc_input(annotation, f)
    }

    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
        LA: FnOnce(LinearCombination<E::Fr>) -> LinearCombination<E::Fr>,
        LB: FnOnce(LinearCombination<E::Fr>) -> LinearCombination<E::Fr>,
        LC: FnOnce(LinearCombination<E::Fr>) -> LinearCombination<E::Fr>,
    {
        self.0.enforce(
            annotation,
            |zero| remove_duplicates::<E>(a(zero)),
            |zero| remove_duplicates::<E>(b(zero)),
            |zero| remove_duplicates::<E>(c(zero)),
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

fn remove_duplicates<E: Engine>(lc: LinearCombination<E::Fr>) -> LinearCombination<E::Fr> {
    let mut inputs_map = BTreeMap::<usize, E::Fr>::new();
    let mut aux_map = BTreeMap::<usize, E::Fr>::new();

    let zero = E::Fr::zero();
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
