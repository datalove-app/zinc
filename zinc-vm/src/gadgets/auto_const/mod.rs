pub mod prelude {
    pub use crate::constraint_systems::ConstantCS;
    pub use crate::gadgets::{Scalar, ScalarVariant};

    use crate::{Engine, Result};

    pub trait ToConstant: Sized {
        fn to_constant(&self) -> Result<Self>;
    }

    impl<E: Engine> ToConstant for Scalar<E> {
        fn to_constant(&self) -> Result<Self> {
            self.as_constant_unchecked()
        }
    }

    impl<E: Engine> ToConstant for (Scalar<E>, Scalar<E>) {
        fn to_constant(&self) -> Result<Self> {
            Ok((
                self.0.as_constant_unchecked()?,
                self.1.as_constant_unchecked()?,
            ))
        }
    }
}

///
/// # Example
/// ```
/// # use ff::PrimeField;
/// # use pairing::bn256::{Bn256, Fr};
/// # use franklin_crypto::circuit::test::TestConstraintSystem;
/// # use zinc_bytecode::scalar::ScalarType;
/// # use bellman::ConstraintSystem;
/// use zinc_vm::auto_const;
/// use zinc_vm::gadgets::Scalar;
/// use zinc_vm::gadgets::arithmetic;
/// use zinc_vm::gadgets::auto_const::prelude::*;
///
/// let a = Scalar::<Bn256>::new_constant_fr(Fr::from_str("42").unwrap(), ScalarType::Field);
/// let b = Scalar::<Bn256>::new_constant_fr(Fr::from_str("69").unwrap(), ScalarType::Field);
///
/// let mut cs = TestConstraintSystem::<Bn256>::new();
///
/// let c: Scalar<Bn256> = auto_const!(arithmetic::mul, cs.ns(|| "mul"), &a, &b).unwrap();
///
/// assert!(c.is_constant());
///
/// let expected = Fr::from_str(&(42 * 69).to_string());
/// assert_eq!(c.get_value(), expected);
///
/// assert!(cs.constraints.is_empty())
/// ```
#[macro_export]
macro_rules! auto_const {
    // Unary operators
    ($op:path, $cs:expr, $a:expr) => {{
        let a = $a;
        match a.get_variant() {
            ScalarVariant::Constant { .. } => {
                let const_cs = ConstantCS;
                let result = $op(const_cs, a);
                result.and_then(|result| result.to_constant())
            }
            _ => $op($cs, a),
        }
    }};
    // Binary operators
    ($op:path, $cs:expr, $a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;
        match (a.get_variant(), b.get_variant()) {
            (ScalarVariant::Constant { .. }, ScalarVariant::Constant { .. }) => {
                let const_cs = ConstantCS;
                let result = $op(const_cs, a, b);
                result.and_then(|result| result.to_constant())
            }
            _ => $op($cs, a, b),
        }
    }};
}
