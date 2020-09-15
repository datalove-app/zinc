use crate::core::{Cell, InternalVM, RuntimeError, VMInstruction, VirtualMachine};
use crate::gadgets;
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Neg;
use zinc_bytecode::scalar::ScalarType;

impl<F, CS> VMInstruction<F, CS> for Neg
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let value = vm.pop()?.value()?;

        let cs = vm.constraint_system();
        let unchecked_neg = gadgets::neg(cs.ns(|| "unchecked_neg"), &value)?;

        match value.get_type() {
            ScalarType::Integer(mut int_type) => {
                let condition = vm.condition_top()?;
                let cs = vm.constraint_system();
                int_type.is_signed = true;
                let neg = gadgets::types::conditional_type_check(
                    cs.ns(|| "neg"),
                    &condition,
                    &unchecked_neg,
                    int_type.into(),
                )?;
                vm.push(Cell::Value(neg))
            }
            scalar_type => Err(RuntimeError::TypeError {
                expected: "integer type".to_string(),
                actual: scalar_type.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::instructions::*;
    use zinc_bytecode::scalar::IntegerType;

    #[test]
    fn test_neg() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new(128.into(), IntegerType::U8.into()))
            .add(Neg)
            .test(&[-128])
    }
}
