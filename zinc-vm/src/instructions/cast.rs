use crate::core::{Cell, InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use crate::gadgets;
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Cast;

impl<F, CS> VMInstruction<F, CS> for Cast
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let old_value = vm.pop()?.value()?;

        let condition = vm.condition_top()?;
        let cs = vm.constraint_system();
        let new_value = gadgets::conditional_type_check(
            cs.ns(|| "type check"),
            &condition,
            &old_value,
            self.scalar_type,
        )?;

        vm.push(Cell::Value(new_value))
    }
}

#[cfg(test)]
mod test {
    use crate::instructions::testing_utils::TestingError;

    #[test]
    fn test_cast() -> Result<(), TestingError> {
        Ok(())
    }
}
