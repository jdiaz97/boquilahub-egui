use clap::{Arg, Command};

use crate::api::rest::{get_ip, run_api};

pub async fn run_cli() {
    let ascii_art = r#"
     
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
        let ip_text = format!("http://{}:8791",get_ip());
        println!("{}", ascii_art);
        println!("Model deployed: {}", model_name);
        println!("IP Address: {}", ip_text);

        // Add your deployment logic here
        run_api().await;
    }
}
