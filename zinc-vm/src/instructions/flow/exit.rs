use crate::core::{InternalVM, RuntimeError, VMInstruction, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Exit;

impl<F, CS> VMInstruction<F, CS> for Exit
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        vm.exit(self.outputs_count)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //    use super::*;
    //    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    //    use zinc_bytecode::*;
    //
    //    #[test]
    //    fn test_exit() -> Result<(), TestingError> {
    //        VMTestRunner::new()
    //            .add(PushConst::new_untyped(1.into()))
    //            .add(Exit::new(0))
    //            .add(PushConst::new_untyped(2.into()))
    //            .test(&[1])
    //    }
}
