use std::fmt;

#[derive(PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Nil => "nil",
            Self::Bool(b) => {
                if *b {
                    "true"
                } else {
                    "false"
                }
            }
            Self::Number(n) => &n.to_string(),
            Self::String(s) => s,
        };

        f.write_str(text)
    }
}
