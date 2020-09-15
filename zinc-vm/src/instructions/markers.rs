use crate::core::{location::CodeLocation, RuntimeError, VMInstruction, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::*;

impl<F, CS> VMInstruction<F, CS> for FileMarker
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        vm.location = CodeLocation {
            file: Some(self.file.clone()),
            function: None,
            line: None,
            column: None,
        };

        Ok(())
    }
}

impl<F, CS> VMInstruction<F, CS> for FunctionMarker
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        vm.location.function = Some(self.function.clone());
        Ok(())
    }
}

impl<F, CS> VMInstruction<F, CS> for LineMarker
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        vm.location.line = Some(self.line);
        Ok(())
    }
}

impl<F, CS> VMInstruction<F, CS> for ColumnMarker
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        vm.location.column = Some(self.column);
        Ok(())
    }
}
