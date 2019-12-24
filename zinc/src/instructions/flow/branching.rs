extern crate franklin_crypto;

use crate::primitive::{Primitive, PrimitiveOperations};
use crate::vm::{VMInstruction, InternalVM};
use crate::vm::{RuntimeError, VirtualMachine};
use zinc_bytecode::{If, Else, EndIf};

impl<E, O> VMInstruction<E, O> for If
    where
        E: Primitive,
        O: PrimitiveOperations<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, O>) -> Result<(), RuntimeError> {
        vm.branch_then()
    }
}

impl<E, O> VMInstruction<E, O> for Else
    where
        E: Primitive,
        O: PrimitiveOperations<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, O>) -> Result<(), RuntimeError> {
        vm.branch_else()
    }
}

impl<E, O> VMInstruction<E, O> for EndIf
    where
        E: Primitive,
        O: PrimitiveOperations<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, O>) -> Result<(), RuntimeError> {
        vm.branch_end()
    }
}

#[cfg(test)]
mod tests {
    use crate::instructions::testing_utils::{VMTestRunner, TestingError};
    use zinc_bytecode::*;
    use std::cmp;

    #[test]
    fn test_stack() -> Result<(), TestingError> {
        // let a = _;
        // let b = _;
        //
        // if a > b {
        //     (a, b)
        // } else {
        //     (b, a)
        // }
        let data = [
            (5, 7),
            (7, 5),
            (6, 6),
        ];

        for (a, b) in data.iter() {
            VMTestRunner::new()
                .add(PushConst { value: (*a).into() })
                .add(Store::new(0))
                .add(PushConst { value: (*b).into() })
                .add(Store::new(1))
                .add(Load::new(0))
                .add(Load::new(1))
                .add(Gt)
                .add(If)
                .add(Load::new(0))
                .add(Load::new(1))
                .add(Else)
                .add(Load::new(1))
                .add(Load::new(0))
                .add(EndIf)
                .test(&[cmp::max(*a, *b), cmp::min(*a, *b)])?;
        }

        Ok(())
    }

    #[test]
    fn test_storage() -> Result<(), TestingError> {
        // let mut a = 0;
        // let c = _;
        //
        // if c {
        //     a += 1;
        // } else {
        //     a -= 1;
        // }
        let data = [
            (1, 1),
            (0, -1),
        ];

        for (c, r) in data.iter() {
            VMTestRunner::new()
                .add(PushConst { value: 0.into() })
                .add(Store::new(0))
                .add(PushConst { value: (*c).into() })
                .add(If)
                .add(PushConst { value: 1.into() })
                .add(Load::new(0))
                .add(Add)
                .add(Store::new(0))
                .add(Else)
                .add(PushConst { value: 1.into() })
                .add(Load::new(0))
                .add(Sub)
                .add(Store::new(0))
                .add(EndIf)
                .add(Load::new(0))
                .test(&[*r])?;
        }
        Ok(())
    }
}