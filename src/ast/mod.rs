pub mod expression;
pub mod program;
pub mod statement;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Identifier {
    pub name: Vec<u8>,
    pub line: usize,
}

impl Identifier {
    pub fn new(name: Vec<u8>, line: usize) -> Self {
        Self { name, line }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.name
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.name
    }
}

impl AsRef<[u8]> for Identifier {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
