use crate::core::{Block, Branch, Cell, FunctionFrame, Loop, VirtualMachine};
use crate::errors::MalformedBytecode;
use crate::gadgets;
use crate::gadgets::Gadgets;
use crate::stdlib::NativeFunction;
use crate::Result;
use crate::RuntimeError;
use algebra::Field;
use r1cs_core::ConstraintSystem;

/// This is an internal interface to virtual machine used by instructions.
pub trait InternalVM<F: Field> {
    fn push(&mut self, cell: Cell<F>) -> Result;
    fn pop(&mut self) -> Result<Cell<F>>;

    fn load(&mut self, address: usize) -> Result<Cell<F>>;
    fn load_global(&mut self, address: usize) -> Result<Cell<F>>;
    fn store(&mut self, address: usize, cell: Cell<F>) -> Result;
    fn store_global(&mut self, address: usize, cell: Cell<F>) -> Result;

    fn loop_begin(&mut self, iter_count: usize) -> Result;
    fn loop_end(&mut self) -> Result;

    fn call(&mut self, address: usize, inputs_count: usize) -> Result;
    fn ret(&mut self, outputs_count: usize) -> Result;

    fn branch_then(&mut self) -> Result;
    fn branch_else(&mut self) -> Result;
    fn branch_end(&mut self) -> Result;

    fn exit(&mut self, values_count: usize) -> Result;
    fn call_native<FF: NativeFunction<F>>(&mut self, function: FF) -> Result;
}

impl<F, CS> InternalVM<F> for VirtualMachine<F, CS>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn push(&mut self, cell: Cell<F>) -> Result {
        self.state.evaluation_stack.push(cell)
    }

    fn pop(&mut self) -> Result<Cell<F>> {
        self.state.evaluation_stack.pop()
    }

    fn load(&mut self, address: usize) -> Result<Cell<F>> {
        let offset = self.top_frame()?.stack_frame_begin;
        self.state.data_stack.get(offset + address)
    }

    fn load_global(&mut self, address: usize) -> Result<Cell<F>> {
        self.state.data_stack.get(address)
    }

    fn store(&mut self, address: usize, cell: Cell<F>) -> Result {
        {
            let frame = self.top_frame()?;
            frame.stack_frame_end =
                std::cmp::max(frame.stack_frame_end, frame.stack_frame_begin + address + 1);
        }
        let offset = self.top_frame()?.stack_frame_begin;
        self.state.data_stack.set(offset + address, cell)
    }

    fn store_global(&mut self, address: usize, cell: Cell<F>) -> Result {
        self.state.data_stack.set(address, cell)
    }

    fn loop_begin(&mut self, iterations: usize) -> Result {
        let frame = self
            .state
            .frames_stack
            .last_mut()
            .ok_or_else(|| RuntimeError::InternalError("Root frame is missing".into()))?;

        frame.blocks.push(Block::Loop(Loop {
            first_instruction_index: self.state.instruction_counter,
            iterations_left: iterations - 1,
        }));

        Ok(())
    }

    fn loop_end(&mut self) -> Result {
        let frame = self.state.frames_stack.last_mut().unwrap();

        match frame.blocks.pop() {
            Some(Block::Loop(mut loop_block)) => {
                if loop_block.iterations_left != 0 {
                    loop_block.iterations_left -= 1;
                    self.state.instruction_counter = loop_block.first_instruction_index;
                    frame.blocks.push(Block::Loop(loop_block));
                }
                Ok(())
            }
            _ => Err(MalformedBytecode::UnexpectedLoopEnd.into()),
        }
    }

    fn call(&mut self, address: usize, inputs_count: usize) -> Result {
        let offset = self.top_frame()?.stack_frame_end;
        self.state
            .frames_stack
            .push(FunctionFrame::new(offset, self.state.instruction_counter));

        for i in 0..inputs_count {
            let arg = self.pop()?;
            self.store(inputs_count - i - 1, arg)?;
        }

        self.state.instruction_counter = address;
        Ok(())
    }

    fn ret(&mut self, outputs_count: usize) -> Result {
        let mut outputs = Vec::new();
        for _ in 0..outputs_count {
            let output = self.pop()?;
            outputs.push(output);
        }

        let frame = self
            .state
            .frames_stack
            .pop()
            .ok_or(MalformedBytecode::StackUnderflow)?;

        self.state.instruction_counter = frame.return_address;

        for p in outputs.into_iter().rev() {
            self.push(p)?;
        }

        Ok(())
    }

    fn branch_then(&mut self) -> Result {
        let condition = self.pop()?.value()?;

        let prev = self.condition_top()?;

        let cs = self.constraint_system();
        let next = gadgets::boolean::and(cs.ns(|| "branch"), &condition, &prev)?;
        self.state.conditions_stack.push(next);

        let branch = Branch {
            condition,
            is_full: false,
        };

        self.top_frame()?.blocks.push(Block::Branch(branch));

        self.state.evaluation_stack.fork();
        self.state.data_stack.fork();

        Ok(())
    }

    fn branch_else(&mut self) -> Result {
        let frame = self
            .state
            .frames_stack
            .last_mut()
            .ok_or_else(|| RuntimeError::InternalError("Root frame is missing".into()))?;

        let mut branch = match frame.blocks.pop() {
            Some(Block::Branch(branch)) => Ok(branch),
            Some(_) | None => Err(RuntimeError::MalformedBytecode(
                MalformedBytecode::UnexpectedElse,
            )),
        }?;

        if branch.is_full {
            return Err(MalformedBytecode::UnexpectedElse.into());
        } else {
            branch.is_full = true;
        }

        let condition = branch.condition.clone();

        frame.blocks.push(Block::Branch(branch));

        self.condition_pop()?;
        let prev = self.condition_top()?;
        let cs = self.constraint_system();
        let not_cond = gadgets::not(cs.ns(|| "not"), &condition)?;
        let next = self.operations().and(prev, not_cond)?;
        self.condition_push(next)?;

        self.state.data_stack.switch_branch()?;
        self.state.evaluation_stack.fork();

        Ok(())
    }

    fn branch_end(&mut self) -> Result {
        self.condition_pop()?;

        let frame = self
            .state
            .frames_stack
            .last_mut()
            .ok_or_else(|| RuntimeError::InternalError("Root frame is missing".into()))?;

        let branch = match frame.blocks.pop() {
            Some(Block::Branch(branch)) => Ok(branch),
            Some(_) | None => Err(MalformedBytecode::UnexpectedEndIf),
        }?;

        if branch.is_full {
            self.state
                .evaluation_stack
                .merge(self.cs.namespace(), &branch.condition)?;
        } else {
            self.state.evaluation_stack.revert()?;
        }

        self.state
            .data_stack
            .merge(branch.condition, &mut Gadgets::new(self.cs.namespace()))?;

        Ok(())
    }

    fn exit(&mut self, outputs_count: usize) -> Result {
        for _ in 0..outputs_count {
            let value = self.pop()?.value()?;
            self.outputs.push(value);
        }
        self.outputs.reverse();

        self.state.instruction_counter = std::usize::MAX;
        Ok(())
    }

    fn call_native<FF: NativeFunction<F>>(&mut self, function: FF) -> Result {
        let stack = &mut self.state.evaluation_stack;
        let cs = &mut self.cs.cs;

        function.execute(cs.ns(|| "native function"), stack)
    }
}
