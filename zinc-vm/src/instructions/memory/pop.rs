use crate::core::{InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Pop;

impl<F, CS> VMInstruction<F, CS> for Pop
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        for _ in 0..self.count {
            vm.pop()?.value()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::*;

    #[test]
    fn test_pop() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new_field(1.into()))
            .add(PushConst::new_field(2.into()))
            .add(Pop::new(1))
            .add(PushConst::new_field(3.into()))
            .add(PushConst::new_field(4.into()))
            .add(PushConst::new_field(5.into()))
            .add(Pop::new(2))
            .test(&[3, 1])
    }
}
