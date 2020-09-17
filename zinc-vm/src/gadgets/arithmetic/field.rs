use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{AllocatedNum, Scalar};
use crate::{Engine, Result};
use algebra::Field;
use r1cs_core::{ConstraintSystem, SynthesisError};

pub fn inverse<E, CS>(cs: CS, scalar: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn inner<E, CS>(mut cs: CS, scalar: &Scalar<E>) -> Result<Scalar<E>>
    where
        E: Engine,
        CS: ConstraintSystem<E::Fr>,
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
    use crate::gadgets::Scalar;
    use algebra::{bn254::{Bn254, Fr}, Field, One};
    use r1cs_core::ConstraintSystem;
    use r1cs_std::test_constraint_system::TestConstraintSystem;
    use zinc_bytecode::scalar::ScalarType;

    #[test]
    fn test_inverse() {
        let mut cs = TestConstraintSystem::<Fr>::new();

        let zero = Scalar::<Bn254>::new_constant_int(0, ScalarType::Field);
        let one = Scalar::<Bn254>::new_constant_int(1, ScalarType::Field);

        assert!(inverse(cs.ns(|| "zero"), &zero).is_err(), "zero");
        assert_eq!(
            inverse(cs.ns(|| "one"), &one).unwrap().get_value().unwrap(),
            Fr::one(),
            "one"
        );
    }
}
