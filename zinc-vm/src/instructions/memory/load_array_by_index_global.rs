use crate::core::{Cell, InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use crate::gadgets;
use crate::gadgets::Scalar;
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::LoadSequenceByIndexGlobal;

impl<F, CS> VMInstruction<F, CS> for LoadSequenceByIndexGlobal
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let index = vm.pop()?.value()?;

        let mut array = Vec::with_capacity(self.array_len);
        for i in 0..self.array_len {
            let value = vm.load_global(self.address + i)?.value()?;
            array.push(value);
        }

        let mut values = Vec::with_capacity(self.value_len);
        for i in 0..self.value_len {
            let cs = vm.constraint_system();
            let offset = Scalar::new_constant_bigint(&i.into(), index.get_type())?;
            let address = gadgets::add(cs.ns(|| format!("address {}", i)), &index, &offset)?;

            let condition = vm.condition_top()?;
            let value =
                vm.operations()
                    .conditional_array_get(&condition, array.as_slice(), &address)?;
            values.push(value);
        }

        for value in values.into_iter() {
            vm.push(Cell::Value(value))?;
        }

        Ok(())
    }
}
