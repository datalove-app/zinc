use crate::core::{location::CodeLocation, VirtualMachine, VMInstruction};
use crate::{Engine, RuntimeError};
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::*;

impl<E, CS> VMInstruction<E, CS> for FileMarker
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        vm.location = CodeLocation {
            file: Some(self.file.clone()),
            function: None,
            line: None,
            column: None,
        };

        Ok(())
    }
}

impl<E, CS> VMInstruction<E, CS> for FunctionMarker
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        vm.location.function = Some(self.function.clone());
        Ok(())
    }
}

impl<E, CS> VMInstruction<E, CS> for LineMarker
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        vm.location.line = Some(self.line);
        Ok(())
    }
}

impl<E, CS> VMInstruction<E, CS> for ColumnMarker
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        vm.location.column = Some(self.column);
        Ok(())
    }
}
