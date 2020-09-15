use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{AllocatedNum, Scalar};
use crate::Result;
use algebra::Field;
use r1cs_core::{ConstraintSystem, SynthesisError};
use r1cs_std::{alloc::AllocGadget, Assignment};

pub fn inverse<F, CS>(cs: CS, scalar: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn inner<F, CS>(mut cs: CS, scalar: &Scalar<F>) -> Result<Scalar<F>>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        let expr = scalar.to_expression::<CS>();

        let inverse = AllocatedNum::alloc(cs.ns(|| "inverse"), || {
            expr.get_value()
                .ok_or(SynthesisError::AssignmentMissing)?
                .inverse()
                .ok_or(SynthesisError::Unsatisfiable)
        })?;

        cs.enforce(
            || "inverse constraint",
            |zero| zero + &scalar.lc::<CS>(),
            |zero| zero + inverse.get_variable(),
            |zero| zero + CS::one(),
        );

        Ok(inverse.into())
    }

    auto_const!(inner, cs, scalar)
}

#[cfg(test)]
mod tests {
    use super::*;

    use algebra::Field;
    use franklin_crypto::circuit::test::TestConstraintSystem;
    use pairing::bn256::{Bn256, Fr};
    use r1cs_core::ConstraintSystem;

    use crate::gadgets::Scalar;
    use zinc_bytecode::scalar::ScalarType;

    #[test]
    fn test_inverse() {
        let mut cs = TestConstraintSystem::<Bn256>::new();

        let zero = Scalar::new_constant_int(0, ScalarType::Field);
        let one = Scalar::new_constant_int(1, ScalarType::Field);

        assert!(inverse(cs.ns(|| "zero"), &zero).is_err(), "zero");
        assert_eq!(
            inverse(cs.ns(|| "one"), &one).unwrap().get_value().unwrap(),
            Fr::one(),
            "one"
        );
    }
}
