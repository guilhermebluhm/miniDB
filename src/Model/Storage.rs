use std::collections::{HashMap, HashSet};
use std::fs::{rename, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
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

    pub fn mount_value_for_get_operation(&self, value: &str) -> Option<String> {

        //melhor caso de O(1)
        let raw_values = self.KvMap.values()
            .map(|f| f.values())
            .flatten().collect::<Vec<&String>>();

        for i in raw_values {
            if i.split("/").nth(0)?.contains(value){
                return Some(i.clone());
            }
        }
        None
    }

    pub fn update_data_log_lastest_values(&mut self, original_file: File) -> Result<File, String> {

        let mut new_insert = false;
        let mut hsmp_tmp:HashMap<String, HashMap<String, u32>> = HashMap::new();
        let data = OpenOptions::new().read(true).open("Data.log").unwrap();
        let reader = BufReader::new(&data);

        for i in reader.lines().enumerate().into_iter(){

            let actual_line = (i.0)+1;
            let mount = i.1.unwrap().clone();
            let key = mount.split(" ").nth(0).unwrap().to_string();
            let content = mount.split_at(key.len()).1.split(" ").collect::<Vec<&str>>();
            let mut aux_hsmp = HashMap::<String, u32>::new();
            let mut new_insert_hsmp: &mut HashMap<String, u32> = &mut HashMap::new();

            if !hsmp_tmp.contains_key(content[1]){
                aux_hsmp.insert(key.clone(), 1);
                hsmp_tmp.insert(format!("{}", content[1]), aux_hsmp);
            }
            else{
                new_insert_hsmp = hsmp_tmp.get_mut(content[1]).unwrap();
                if !new_insert_hsmp.contains_key(&key){
                    new_insert = true;
                }
            }

            if new_insert {
                new_insert_hsmp.insert(format!("{}", key.clone()), 1);
            }
            else{
                if (actual_line % 2 == 0) && let Some(x) = hsmp_tmp.get_mut(content[1]) {
                    let mut conj_value = x.values_mut().map(|f| f).collect::<Vec<&mut u32>>();
                    if conj_value.len() == 1{
                        *conj_value[0] += 1;
                        println!("TROCA valor conjunto recorrente {:?}", conj_value);
                    }
                }
            }

        }

        let mut file_tmp:File = File::create("Data_tmp.log").unwrap();
        for (_, v) in hsmp_tmp.iter().enumerate(){

            let mut get_ocurrence_description = String::new();
            let mut delete_ocurrence_description = String::new();
            let mut straight_update_file = true;
            let mut qtde_set = 0;
            let mut qtde_del = 0;

            if v.1.len() > 1 {
                let data = v.1.iter().map(|f| f).collect::<Vec<(&String, &u32)>>();

                for i in data {
                    if i.0.contains("SET") {
                        qtde_set = *i.1;
                    }
                    if i.0.contains("DEL") {
                        if delete_ocurrence_description.is_empty(){
                            delete_ocurrence_description = format!("DELETE {}\n", v.0.to_string())
                        }
                        qtde_del = *i.1;
                    }
                    if i.0.contains("GET") && get_ocurrence_description.is_empty(){
                        get_ocurrence_description = format!("GET {}\n", v.0.to_string())
                    }
                }
            }

            if v.1.len() > 1 && qtde_set == qtde_del{
                straight_update_file = false;
            }

            if straight_update_file {

                let mut linha_gravacao = String::new();
                let last_value_for_operation = self.KvMap.values()
                    .map(|f| f.iter())
                    .flatten().map(|f| f)
                    .filter(|f| f.1.contains(v.0)).collect::<Vec<(&u32, &String)>>()
                    .iter().map(|f| f).max_by_key(|f| f.0).unwrap().1;

                linha_gravacao = format!("{} {} {}\n", v.1.keys().map(|f| f).find(|f|
                                         f.contains("SET")).unwrap_or(&"".to_string()),
                                         last_value_for_operation.split("/").nth(0).unwrap(),
                                         last_value_for_operation.split("/").nth(1).unwrap());

                if !linha_gravacao.is_empty(){
                    file_tmp.write(linha_gravacao.as_bytes()).unwrap();
                    file_tmp.sync_all().unwrap();
                }
                if !get_ocurrence_description.is_empty() {
                    file_tmp.write(get_ocurrence_description.as_bytes()).unwrap();
                    file_tmp.sync_all().unwrap();
                }
                if !delete_ocurrence_description.is_empty() {
                    file_tmp.write(delete_ocurrence_description.as_bytes()).unwrap();
                    file_tmp.sync_all().unwrap();
                }

            }

        }

        drop(original_file);
        let _ = rename("Data_tmp.log","Data.log");
        Ok(data)

    }

    pub fn delete_value(&mut self, value: &str) -> Option<String> {

        let mut content = String::new();
        let mut id_for_delete = 0;
        let raw_values = self.KvMap.values()
            .map(|f| f.iter())
            .flatten().collect::<Vec<(&u32, &String)>>();

        for i in raw_values {
            if i.1.contains(value){
                id_for_delete = *i.0;
                content.push_str(format!("{}/{}", i.1.split("/").nth(0).unwrap(), i.1.split("/").nth(1).unwrap()).as_str());
                break;
            }
        }

        if id_for_delete != 0 {
            self.KvMap.retain(|f, _| f.actual_id() != id_for_delete);
            return Some(content)
        }

        None


    }

}