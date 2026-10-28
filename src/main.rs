use crate::enums::ParserEnum;
use crate::model::Operation;
use model::Storage;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use colored::Colorize;
use log::{info, warn};
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

    loop {
        println!("Storage engine. Supported operations: [GET;SET;DELETE;PREFIX;EXPORT;COMPACT]");
        let mut comando = String::new();
        let mut dir = std::env::current_dir()?;
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
            Command::Command::PREFIX{key} => {
                println!("PREFIXO {:?}", key);
                initial_storage.search_key_by_prefix(key.as_str());
            }
            Command::Command::COMPACT => {
                file = initial_storage.update_data_log_lastest_values(file)?;
            }
            Command::Command::EXPORT => {

                let mut opc = 0;
                let mut name_file = String::new();
                let json_type = "1. JSON".green();
                let csv_type = "2. CSV".blue();
                let mut type_format_export = String::new();

                println!("{}\n{}", json_type, csv_type);
                std::io::stdin().read_line(&mut type_format_export)?;
                opc = type_format_export.trim().parse::<u8>()?;
                if opc <= 0 || opc > 2 {
                    println!("invalid op number");
                }
                else{
                    match opc {
                        1 => {
                            name_file.push_str("export.json");
                        }
                        2 => {
                            name_file.push_str("export.csv");
                        }
                        _ => ()
                    }
                }

                loop {

                    let mut inside_dir:Vec<String> = Vec::new();
                    let mut mount_dir:Vec<String> = Vec::new();
                    let actual_dir = std::fs::read_dir(dir.as_path())?;

                    println!("Directory navigation");
                    println!("Actual diretory: {:?}", actual_dir);


                    let mut it = 0;
                    for i in actual_dir{
                        let entry = i?;
                        let current_dir = entry.file_type()?;
                        if current_dir.is_dir(){
                            println!("pos {:?} -> path: {:?}", it,entry.file_name().to_string_lossy());
                            inside_dir.push(entry.file_name().to_string_lossy().to_string());
                            it+=1;
                        }
                    }

                    println!("append directory [2-No]");
                    type_format_export = String::new();
                    std::io::stdin().read_line(&mut type_format_export)?;
                    opc = type_format_export.trim().parse::<u8>()?;

                    if opc != 2 {
                        let directory_to_append = inside_dir.get(opc as usize).unwrap();
                        println!("value extract and appended: {:?}", directory_to_append);
                        dir.push(directory_to_append);
                    }

                    println!("show ancestrors [1-y | 2-stop]");
                    type_format_export = String::new();
                    std::io::stdin().read_line(&mut type_format_export)?;
                    opc = type_format_export.trim().parse::<u8>()?;

                    if opc == 2 {
                        break;
                    }

                    if opc == 1 {
                        println!("Ancestors");
                        for i in dir.as_path().ancestors().enumerate(){
                            mount_dir.push(i.1.to_str().unwrap().to_string());
                            println!("pos {} -> {}", i.0, i.1.to_str().unwrap());
                        }

                        println!("Change currenct dir ? [99-n]");
                        type_format_export = String::new();
                        std::io::stdin().read_line(&mut type_format_export)?;
                        opc = type_format_export.trim().parse::<u8>()?;

                        if opc != 99 {
                            dir = PathBuf::from(mount_dir.get(opc as usize).unwrap().as_str());
                        }
                    }

                    std::io::stdout().flush()?;

                }
                dir = dir.join(name_file);
                println!("dir choose: {:?}", dir.as_path().to_string_lossy());
                let mut file:File = File::create(&dir)?;
                let data_for_file = initial_storage.mount_all_data_in_memory_for_export();
                let json = serde_json::to_string_pretty(&data_for_file)?;
                file.write(json.as_bytes())?;

            }
            _ => {}
        }
        std::io::stdout().flush()?;
    }
    Ok(())

}
