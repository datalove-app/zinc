use crate::vm::{VMInstruction, VirtualMachine};
use crate::RuntimeError;
use crate::primitive::{Primitive, PrimitiveOperations};

pub mod push;
pub mod pop;
pub mod load;
pub mod store;
pub mod load_array;
pub mod store_array;
pub mod load_by_index;
pub mod store_by_index;
pub mod load_array_by_index;
pub mod store_array_by_index;
pub mod load_global;
pub mod load_array_global;
pub mod load_by_index_global;
pub mod load_array_by_index_global;
