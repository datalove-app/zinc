use crate::core::{Cell, InternalVM, RuntimeError, VirtualMachine, VMInstruction};
use crate::gadgets::Scalar;
use crate::{gadgets, Engine};
use r1cs_core::ConstraintSystem;
use zinc_bytecode::StoreSequenceByIndex;

impl<E, CS> VMInstruction<E, CS> for StoreSequenceByIndex
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        let mut array = Vec::with_capacity(self.array_len);
        for i in 0..self.array_len {
            let value = vm.load(self.address + i)?.value()?;
            array.push(value);
        }

        let mut values = Vec::with_capacity(self.value_len);
        for _ in 0..self.value_len {
            let value = vm.pop()?.value()?;
            values.push(value);
        }
        values.reverse();

        let index = vm.pop()?.value()?;

        for (i, value) in values.into_iter().enumerate() {
            let cs = vm.constraint_system();
            let offset = Scalar::new_constant_bigint(&i.into(), index.get_type())?;
            let address = gadgets::add(cs.ns(|| format!("address {}", i)), &index, &offset)?;
            array = vm
                .operations()
                .array_set(array.as_slice(), address, value)?;
        }

        for (i, value) in array.into_iter().enumerate() {
            vm.store(self.address + i, Cell::Value(value))?;
        }

        Ok(())
    }
}
