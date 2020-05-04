use crate::core::location::CodeLocation;
use crate::core::{VMInstruction, VirtualMachine};
use crate::{Engine, RuntimeError};
use bellman::ConstraintSystem;
use zinc_bytecode::instructions::*;

impl<E, CS> VMInstruction<E, CS> for FileMarker
where
    E: Engine,
    CS: ConstraintSystem<E>,
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
    CS: ConstraintSystem<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        vm.location.function = Some(self.function.clone());
        Ok(())
    }
}

impl<E, CS> VMInstruction<E, CS> for LineMarker
where
    E: Engine,
    CS: ConstraintSystem<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        vm.location.line = Some(self.line);
        Ok(())
    }
}

impl<E, CS> VMInstruction<E, CS> for ColumnMarker
where
    E: Engine,
    CS: ConstraintSystem<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        vm.location.column = Some(self.column);
        Ok(())
    }
}
