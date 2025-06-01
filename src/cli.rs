use clap::{Arg, Command};

use crate::api::{
    eps::LIST_EPS,
    inference::set_model,
    rest::{get_ip, run_api},
};

pub async fn run_cli() {
    let matches = Command::new("BoquilaHUB")
        .version("1.0")
        .about("BoquilaHUB - GUI and CLI tool")
        .arg(
            Arg::new("deploy")
                .long("deploy")
                .help("Deploy mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("model")
                .long("model")
                .help("Model name to deploy")
                .value_name("MODEL_NAME")
                .requires("deploy"),
        )
        .get_matches();

    // Check if CLI arguments are provided
    if matches.get_flag("deploy") {
        // CLI mode
        let model_name = matches.get_one::<String>("model").unwrap();
        let ip_text = format!("http://{}:8791", get_ip());
        println!("{}", ASCII_ART);
        println!("Model deployed: {}", model_name);
        println!("IP Address: {}", ip_text);

        let model_path = format!(
            "models/{}",
            if model_name.ends_with(".bq") {
                model_name.to_string()
            } else {
                format!("{}.bq", model_name)
            }
        );

        // Add your deployment logic here
        set_model(model_path, LIST_EPS[1].clone());
        run_api().await;
    }
}

const ASCII_ART: &'static str = r#"
     
 /$$$$$$$                                /$$ /$$           /$$   /$$ /$$   /$$ /$$$$$$$ 
| $$__  $$                              |__/| $$          | $$  | $$| $$  | $$| $$__  $$
| $$  \ $$  /$$$$$$   /$$$$$$  /$$   /$$ /$$| $$  /$$$$$$ | $$  | $$| $$  | $$| $$  \ $$
| $$$$$$$  /$$__  $$ /$$__  $$| $$  | $$| $$| $$ |____  $$| $$$$$$$$| $$  | $$| $$$$$$$ 
| $$__  $$| $$  \ $$| $$  \ $$| $$  | $$| $$| $$  /$$$$$$$| $$__  $$| $$  | $$| $$__  $$
| $$  \ $$| $$  | $$| $$  | $$| $$  | $$| $$| $$ /$$__  $$| $$  | $$| $$  | $$| $$  \ $$
| $$$$$$$/|  $$$$$$/|  $$$$$$$|  $$$$$$/| $$| $$|  $$$$$$$| $$  | $$|  $$$$$$/| $$$$$$$/
|_______/  \______/  \____  $$ \______/ |__/|__/ \_______/|__/  |__/ \______/ |_______/ 
                          | $$                                                          
                          | $$                                                          
                          |__/                                      AI for Biodiversity                             

    "#;
