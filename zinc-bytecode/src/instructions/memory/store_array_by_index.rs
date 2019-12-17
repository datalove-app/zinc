use crate::{DecodingError, Instruction, InstructionCode, InstructionInfo, utils};

/// Takes `index` and several values from evaluation stack, stores values in data stack at `address + index`.
#[derive(Debug, PartialEq, Clone)]
pub struct StoreArrayByIndex {
    pub address: usize,
    pub array_len: usize,
    pub value_len: usize,
}

impl StoreArrayByIndex {
    pub fn new(address: usize, array_len: usize, value_len: usize) -> Self {
        Self { address, array_len, value_len }
    }
}

impl InstructionInfo for StoreArrayByIndex {
    fn to_assembly(&self) -> String {
        format!("store_array_by_index {} {} {}", self.address, self.array_len, self.value_len)
    }

    fn code() -> InstructionCode {
        InstructionCode::StoreArrayByIndex
    }

    fn encode(&self) -> Vec<u8> {
        utils::encode_with_usize(Self::code(), &[self.address, self.array_len, self.value_len])
    }

    fn decode(bytes: &[u8]) -> Result<(StoreArrayByIndex, usize), DecodingError> {
        let (args, len) = utils::decode_with_usize(Self::code(), bytes, 3)?;

        Ok((
            Self::new(args[0], args[1], args[2]),
            len,
        ))
    }

    fn inputs_count(&self) -> usize {
        1 + self.value_len
    }

    fn outputs_count(&self) -> usize {
        0
    }

    fn wrap(&self) -> Instruction {
        Instruction::StoreArrayByIndex((*self).clone())
    }
}
