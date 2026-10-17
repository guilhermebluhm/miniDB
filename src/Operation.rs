use crate::ParserEnum::ParserEnum;

#[derive(Copy, Clone, Debug, Eq, PartialOrd, PartialEq, Hash)]
pub struct Operation {
    id: u32,
    type_operation: ParserEnum,
}

impl Operation {
    pub fn new(id: u32, type_operation: ParserEnum) -> Operation {
        Self{
            id, type_operation
        }
    }
    
    pub fn new_with_empty_values() -> Operation {
        Self{
            id: 0, type_operation: ParserEnum::SET
        }
    }
    
    pub fn actual_id(&self) -> u32 {
        self.id
    }
    
    pub fn type_operation(&self) -> ParserEnum {
        self.type_operation.clone()
    }
    
}