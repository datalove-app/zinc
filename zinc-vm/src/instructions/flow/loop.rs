use crate::core::{InternalVM, RuntimeError, VMInstruction, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::{LoopBegin, LoopEnd};

impl<F, CS> VMInstruction<F, CS> for LoopBegin
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        vm.loop_begin(self.iterations)
    }
}

impl<F, CS> VMInstruction<F, CS> for LoopEnd
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        vm.loop_end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::{Add, Load, PushConst, Store};

    #[test]
    fn test_loop() -> Result<(), TestingError> {
        let _ = env_logger::builder().is_test(true).try_init();

        VMTestRunner::new()
            .add(PushConst::new_field(0.into()))
            .add(Store::new(0))
            .add(PushConst::new_field(0.into()))
            .add(Store::new(1))
            .add(LoopBegin::new(10))
            .add(Load::new(0))
            .add(PushConst::new_field(1.into()))
            .add(Add)
            .add(Store::new(0))
            .add(Load::new(0))
            .add(Load::new(1))
            .add(Add)
            .add(Store::new(1))
            .add(LoopEnd)
            .add(Load::new(0))
            .add(Load::new(1))
            .test(&[55, 10])
    }
}
