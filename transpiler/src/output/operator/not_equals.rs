//!
//! Transpiler output not-equals operator.
//!

use crate::element::Element;

pub struct Output {}

impl Output {
    pub fn output(
        identifier: String,
        namespace: String,
        operand_1: Element,
        operand_2: Element,
    ) -> String {
        format!(
            r#"let {0} = r1cs::not_equals_number(system.namespace(|| {1}), &{2}, &{3}, 254)?;"#,
            identifier, namespace, operand_1, operand_2,
        )
    }
}
