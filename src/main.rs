use crate::enums::ParserEnum;
use crate::model::Operation;
use model::Storage;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use log::{error, info, warn};
use crate::parser::Command;

mod model;
mod enums;
mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut initial_storage = Storage::Storage::new();
    let mut file = OpenOptions::new()
        .create(true).write(true)
        .append(true).read(true)
        .open("Data.log")?;

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Initializing initial storage");

    let reader = BufReader::new(&file);
    let mut id = 1;
    for (_, line) in reader.lines().enumerate() {
        let line = line?.clone();
        let tmp_vl = line.split_whitespace().collect::<Vec<&str>>().to_vec();
        if tmp_vl.len() >= 2 && !tmp_vl[0].eq("GET") {
            if tmp_vl[0].eq("DELETE"){
                let _ = initial_storage.delete_value(tmp_vl[1]);
            }
            else {

                if !tmp_vl[0].eq("SET") {
                    warn!("invalid command is not loaded: {:?}", tmp_vl[0]);
                    continue;
                }

                if tmp_vl.get(2).is_none(){
                    warn!("mal-formed command: {:?}", tmp_vl[0]);
                    continue;
                }

                let mut hsmp = HashMap::new();
                hsmp.insert(id, format!("{}/{}", tmp_vl[1].to_string(), tmp_vl[2].to_string()));
                initial_storage.insert(Operation::Operation::new(id, ParserEnum::ParserEnum::SET), &hsmp);
                id += 1;
            }
        }
    }

    while true {
        println!("Storage engine. Supported operations: [GET;SET;DELETE;COMPACT]");
        let mut comando = String::new();
        let _ = std::io::stdin().read_line(&mut comando);
        comando = comando.trim().to_string();

        let parser_data = Command::ParseCommand(&mut comando);
        if parser_data.is_err() {
            println!("Failed for this operation because: {:?}", parser_data.unwrap_err());
            continue;
        }

        match parser_data.unwrap() {
            Command::Command::SET {key, value} => {

                let mut new_value_for_hsmp = String::new();
                let mut oprc = &Operation::Operation::new_with_empty_values();
                let op = ParserEnum::ParserEnum::SET;
                let mut user_choise = String::new();

                //--seção de persistência
                let mut hsmp = HashMap::new();
                let mut id = 0;
                if initial_storage.KvMap.is_empty(){
                    id = 1;
                }
                else{
                    id = initial_storage.KvMap.iter().map(|f| f.0.actual_id()).max().unwrap();
                    id+=1;
                }

                hsmp.insert(id, format!("{}/{}", key.as_str(), value.as_str()));
                initial_storage.insert(Operation::Operation::new(id, ParserEnum::ParserEnum::SET), &hsmp);
                file.write(format!("SET {} {}\n", key.as_str(), value.as_str()).as_bytes())?;
                file.sync_data()?;
                //--seção de persistência

                //--seção de leitura para producao de novos dados por conferencia
                let dataset_group_operation = initial_storage.check_and_mount_data(&op);
                if !dataset_group_operation.is_empty(){
                    for i in dataset_group_operation.iter() {
                        println!("{:?} -> {:?}", i, initial_storage.get_value_from_operation(*i));
                    }

                    println!("informe o codigo do registro a ser selecionado: ");
                    std::io::stdin().read_line(&mut user_choise)?;
                    let opsel = user_choise.trim().parse::<u32>()?;

                    if opsel == 0{
                        println!("cancel update registry");
                        continue;
                    }

                    oprc = dataset_group_operation.iter().find(|f| f.actual_id() == opsel).unwrap();
                    println!("insert new value");
                    std::io::stdin().read_line(&mut new_value_for_hsmp)?;
                }

                if oprc.actual_id() != 0 {
                    new_value_for_hsmp = new_value_for_hsmp
                        .trim().to_string();
                    initial_storage
                        .retrieve_value_from_operation(&oprc.clone(), format!("{}/{}", key.as_str(), new_value_for_hsmp).as_str());
                    file.write(format!("SET {} {}\n", key.as_str(), new_value_for_hsmp).as_bytes())?;
                    file.sync_data()?;
                    std::io::stdout().flush()?;
                    continue;
                }
                //--seção de leitura para producao de novos dados por conferencia
            }
            Command::Command::GET {key} => {
                let content = initial_storage.mount_value_for_get_operation(key.as_str());
                if content.is_some(){
                    println!("value is fetched: {:?}", content.clone().unwrap());
                    file.write(format!("GET {} {}\n", key.as_str(), content.unwrap().split("/")
                        .nth(1).unwrap()).as_bytes())?;
                    file.sync_data()?;
                }
            }
            Command::Command::DELETE {key} => {
                let success = initial_storage.delete_value(key.as_str());
                if success.is_some(){
                    println!("Registry deleted: {:?}", success.clone().unwrap());
                    file.write(format!("DELETE {} {}\n", key.as_str(), success.unwrap().split("/")
                        .nth(1).unwrap()).as_bytes())?;
                    file.sync_data()?;
                }
            }
            Command::Command::COMPACT => {
                file = initial_storage.update_data_log_lastest_values(file)?;
                println!("Compact action");
            }
            _ => {}
        }
        std::io::stdout().flush()?;
    }
    Ok(())

}
