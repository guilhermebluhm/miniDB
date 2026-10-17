use regex::Regex;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::ops::ControlFlow;

mod ParserEnum;
mod Storage;
mod Operation;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut initial_storage = Storage::Storage::new();
    let mut file = OpenOptions::new()
        .create(true).write(true)
        .append(true).read(true)
        .open("Data.log")?;

    let reader = BufReader::new(&file);
    let mut id = 1;
    for (_, line) in reader.lines().enumerate() {
        let line = line?.clone();
        let tmp_vl = line.split_whitespace().collect::<Vec<&str>>().to_vec();
        if tmp_vl.len() >= 2 {
            let mut hsmp = HashMap::new();
            hsmp.insert(id, format!("{}/{}", tmp_vl[1].to_string(), tmp_vl[2].to_string()));
            initial_storage.insert(Operation::Operation::new(id, ParserEnum::ParserEnum::SET), &hsmp);
            id+=1;
        }
    }

    while true {
        let mut comando = String::new();
        let _ = std::io::stdin().read_line(&mut comando);
        comando = comando.trim().to_string();

        let re = Regex::new(r"[^\p{L}\p{N}]+").unwrap();
        comando = re.replace_all(comando.as_str(), " ").to_string();

        let instrucoes = comando
            .split_whitespace().into_iter()
            .collect::<Vec<&str>>();

        if instrucoes.len() <= 2 {
            println!("invalid instruction length {}", instrucoes.len());
            continue;
        }

        let d = instrucoes.to_vec().iter().try_fold(0, |mut acc, op|{

            let mut it = 0;

            if op.to_ascii_uppercase().contains("SET")
                || op.to_ascii_uppercase().contains("GET"){
                acc+=1
            }

            if it == 0 && acc == 1 {
                return ControlFlow::Break(acc);
            }
            else{
                it+=1;
                return ControlFlow::Continue(99);
            }
        });

        if d.continue_value().unwrap_or(0) == 99 {
            println!("invalid position and/or operation");
            continue;
        }

        match instrucoes.get(0).unwrap().to_ascii_uppercase().as_ref() {
            "SET" => {

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

                hsmp.insert(id, format!("{}/{}", instrucoes[1].to_string(), instrucoes[2].to_string()));

                //introduzir validação para garantir que não haja conteudo duplicado no hashmap

                initial_storage.insert(Operation::Operation::new(id, ParserEnum::ParserEnum::SET), &hsmp);
                file.write(format!("SET {} {}\n", instrucoes[1], instrucoes[2]).as_bytes())?;
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

                    oprc = dataset_group_operation.iter().find(|f| f.actual_id() == opsel).unwrap().clone();
                    println!("insert new value");
                    std::io::stdin().read_line(&mut new_value_for_hsmp)?;
                }

                if oprc.actual_id() != 0 {
                    initial_storage.retrieve_value_from_operation(&oprc.clone(), format!("{}/{}", instrucoes[1], new_value_for_hsmp).as_str());
                    file.write(format!("SET {} {}\n", instrucoes[1], new_value_for_hsmp).as_bytes())?;
                    std::io::stdout().flush()?;
                    continue;
                }
                //--seção de leitura para producao de novos dados por conferencia
            }
            _ => {}
        }
        std::io::stdout().flush()?;
    }
    Ok(())

}
