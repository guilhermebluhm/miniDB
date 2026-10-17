use std::collections::{HashMap, HashSet};
use crate::Operation::Operation;
use crate::ParserEnum::ParserEnum;

#[derive(Debug)]
pub struct Storage{
    pub KvMap: HashMap<Operation, HashMap<u32, String>>
}

impl Storage {
    pub fn new() -> Storage {
        Self{
            KvMap: HashMap::new()
        }
    }
}

impl Storage{
    pub fn insert(&mut self, k: Operation, v: &HashMap<u32, String>){
       self.KvMap.insert(k, v.clone());
    }

    pub fn check_and_mount_data(&self, k: &ParserEnum) -> Box<Vec<&Operation>> {

        if self.KvMap.is_empty() {
            return Box::new(Vec::new());
        }
        Box::new(self.KvMap.iter()
            .map(|f| f.0)
            .filter(|f| f.type_operation() == k.clone())
            .collect::<Vec<&Operation>>())

    }

    pub fn retrieve_value_from_operation(&mut self, op: &Operation, new_content: &str) -> () {

        if let Some(x) = self.KvMap.get_mut(op).unwrap().get_mut(&op.actual_id()){
            *x = new_content.to_string();
        }

    }
    
    pub fn get_value_from_operation(&self, operation: &Operation) -> String {
        
        let value = self.KvMap.get(operation).unwrap().get(&operation.actual_id());
        if value.is_some(){
            return value.unwrap().clone();
        }
        
        "".to_string()
        
    }

}