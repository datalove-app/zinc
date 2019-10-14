//!
//! Transpiler temporary element.
//!

use std::fmt;

use crate::element::Descriptor;

pub struct Element {
    pub identifier: String,
    pub descriptors: Vec<Descriptor>,
}

impl Element {
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            descriptors: Default::default(),
        }
    }

    pub fn push_descriptor(&mut self, descriptor: Descriptor) {
        self.descriptors.push(descriptor);
    }
}

impl Into<String> for Element {
    fn into(self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            "(".repeat(self.descriptors.len()),
            self.identifier,
            self.descriptors
                .iter()
                .map(|descriptor| descriptor.to_string() + ")")
                .collect::<Vec<String>>()
                .join(""),
        )
    }
}
