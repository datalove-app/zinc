//!
//! Transpiler output debug statement.
//!

use crate::element::Element;

pub struct Output {}

impl Output {
    pub fn output(expression: Element) -> String {
        format!(r#"dbg!(&{0}.get_value());"#, expression)
    }
}
