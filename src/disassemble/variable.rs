use std::fmt;

#[derive(Debug, Clone)]
pub enum VariableValue {
    U8(u8),
    U16(u16),
}

impl fmt::Display for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::U8(v) => write!(f, "${:02X?}", v),
            Self::U16(v) => write!(f, "${:04X?}", v),
        };
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: VariableValue,
}
