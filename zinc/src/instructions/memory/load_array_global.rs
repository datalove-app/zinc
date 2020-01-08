use crate::primitive::{Primitive, PrimitiveOperations};
use crate::vm::{VMInstruction, InternalVM};
use crate::vm::{RuntimeError, VirtualMachine};
use zinc_bytecode::LoadSequenceGlobal;

impl<E, O> VMInstruction<E, O> for LoadSequenceGlobal
where
    E: Primitive,
    O: PrimitiveOperations<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, O>) -> Result<(), RuntimeError> {
        for i in 0..self.len {
            let value = vm.load_global(self.address + self.len - i - 1)?;
            vm.push(value)?;
        }

        Ok(())
    }
}
