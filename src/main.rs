use std::env;

use acbr_lib::acbr_lib_esocial::ACBrLibEsocial;
use futures::future::join_all;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let mut handles = Vec::new();

    for i in 1..5000 {
        println!("Abrindo a thread1: {}", i);

        let handle = tokio::spawn(async move {
            match execute().await {
                Ok(result) => println!("Resultado thread1 {}: {}", i, result),
                Err(err) => {
                    eprintln!("Erro Thread1 {}: {}", i, err)
                }
            }
        });

        handles.push(handle);
    }

    // for i in 1..5000 {
    //     println!("Abrindo a thread2: {}", i);

    //     let handle = tokio::spawn(async move {
    //         match execute().await {
    //             Ok(result) => println!("Resultado thread2 {}: {}", i, result),
    //             Err(err) => {
    //                 eprintln!("Erro Thread2 {}: {}", i, err)
    //             }
    //         }
    //     });

    //     handles.push(handle);
    // }

    let results = join_all(handles).await;
    let errors_count = results.iter().filter(|r| r.is_err()).count();

    println!("Todas as threads foram concluídas!");
    println!("Total de threads com erro: {}", errors_count);
}

async fn execute() -> Result<String, String> {
    let mut lib = ACBrLibEsocial::new()?;

    let id = Uuid::new_v4().to_string();

    
    let base_path_config = env::current_dir().map_err(|err| err.to_string())?;

    let schems_path_directory = base_path_config.join("resources/temp/schemas");
    let schemas_caminho = schems_path_directory.to_str().unwrap();

    
    let random_path = format!("resources/temp_ini/config_{id}.ini");
    let current_path_config = env::current_dir().map_err(|err| err.to_string())?;
    let config_directory = current_path_config.join(random_path);
    let caminho_config = config_directory.to_str().unwrap();

    let _ = lib.esocial_inicializar(caminho_config, "")?;

    let current_path_logs = env::current_dir().map_err(|err| err.to_string())?;
    let logs_directory = current_path_logs.join("logs");
    let caminho_logs = logs_directory.to_str().unwrap();

    lib.esocial_config_gravar_valor("Principal", "LogPath", caminho_logs)?;
    lib.esocial_config_gravar_valor("eSocial", "PathSchemas", schemas_caminho)?;

    lib.esocial_config_gravar(caminho_config)?;

    let xml_content = std::fs::read_to_string("evento.xml")
        .map_err(|e| format!("Erro ao ler arquivo evento.xml: {}", e))?;

    let versao = lib.esocial_versao()?;

    lib.carregar_xml_evento(xml_content)?;

    lib.enviar_esocial(1)?;

    let ultimo_retorno = lib.obter_ultimo_retorno()?;
    println!("ultimo retorno: {}", ultimo_retorno);

    Ok("Debug finalizado!!!!".into())
}
