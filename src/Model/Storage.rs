use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
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

    pub fn update_data_log_lastest_values(&mut self) -> Result<(), String> {

        let mut new_insert = false;
        /*
        hashmap para acumular os registros recorrentes
        para representar a estrutura dado pela CHAVE (p.ex. mmg) seguido da operação (p.ex) GET/SET + numero ocorrencias
        apos montar pelo Data.log remontar a estrutura pelo que há em memoria(kvMap) apenas com os ultimos registros
        de cada chave pela sua respectiva operação
        */
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
                //logica ainda requer melhorias para outros casos.
                //rodando em caso de base para validar o fluxo
                if (actual_line % 2 == 0) && let Some(x) = hsmp_tmp.get_mut(content[1]) {
                    let mut conj_value = x.values_mut().map(|f| f).collect::<Vec<&mut u32>>();
                    if conj_value.len() == 1{
                        *conj_value[0] += 1;
                        println!("TROCA valor conjunto recorrente {:?}", conj_value);
                    }
                }
            }

        }

        //verificar a logica e remontar os registros em memoria mais novos do hashmap
        //de registros (hsmp_tmp) para o Data.log atualizado a ser recriado a partir de um
        //arquivo intermediario (Data_tmp.log)

        println!("{:#?}", hsmp_tmp);
        println!("KVMAP: {:#?}", self.KvMap);

        println!();
        println!("TESTES INICIAIS DA LOGICA DE REMONTAGEM");
        //primeira experimentacao em montar o ultimo registro da entrada "dse" mais recente
        for (k, v) in hsmp_tmp.iter().enumerate(){
            let data = v.1.values().map(|f| f).collect::<Vec<&u32>>();
            //isso e verdade pois, caso o contador interno do hsmp tmp seja > 1.
            //signica que o KvMap precisa recuperar o ultimo registro
            if data.len() == 1 && *data[0] > 1{
                println!("lendo os dados");
                //nao esta pegando o ultimo registro (acertar)
                //TODO: quando pegar o ultimo registro rodar toda a logica que já está montada, após realizar os polimentos
                let r = self.KvMap.values().map(|f| f.values()).flatten()
                    .max_by_key(|f| f.split(" ").nth(0).unwrap().contains("dse"));
                println!("{:?}", r);
            }
        }
        Ok(())
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