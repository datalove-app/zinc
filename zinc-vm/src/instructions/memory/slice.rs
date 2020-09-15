use crate::core::{Cell, InternalVM, RuntimeError, VMInstruction, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Slice;

impl<F, CS> VMInstruction<F, CS> for Slice
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let offset = vm.pop()?.value()?;

        let mut array = Vec::with_capacity(self.array_len);
        for _ in 0..self.array_len {
            let value = vm.pop()?.value()?;
            array.push(value);
        }
        array.reverse();

        for i in 0..self.slice_len {
            let condition = vm.condition_top()?;
            let value = vm.operations().conditional_array_get(
                &condition,
                &array[i..=array.len() - self.slice_len + i],
                &offset,
            )?;
            vm.push(Cell::Value(value))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::PushConst;

    #[test]
    fn test_slice() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new_field(1.into()))
            .add(PushConst::new_field(2.into()))
            .add(PushConst::new_field(3.into()))
            .add(PushConst::new_field(4.into()))
            .add(PushConst::new_field(5.into()))
            .add(PushConst::new_field(6.into()))
            .add(PushConst::new_field(2.into()))
            .add(Slice::new(5, 2))
            .test(&[5, 4, 1])
    }
}
