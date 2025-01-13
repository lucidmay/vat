use crate::config::VatConfig;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Stacks{
    pub stacks: HashMap<String, Stack>,
    pub order: Vec<String>
}

impl Stacks{

    pub fn stacks_config_path() -> PathBuf{
        let config = VatConfig::get_app_dir();
        if config.is_some(){
            let config_path = config.unwrap().join("stacks.toml");
            config_path
        }else{
            PathBuf::new()
        }
    }

    pub fn init() -> Result<Self, anyhow::Error>{
        let config_path = Stacks::stacks_config_path();
        if config_path.exists(){
            let config = toml::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
            Ok(config)
        }else{
            let config = Stacks{stacks: HashMap::new(), order: vec![]};
            let config_str = toml::to_string(&config).unwrap();
            fs::write(config_path, config_str).unwrap();
            Ok(config)
        }
    }

    pub fn save(stacks: &Stacks) -> Result<(), anyhow::Error>{
        let config_path = Stacks::stacks_config_path();
        let config_str = toml::to_string(&stacks).unwrap();
        fs::write(config_path, config_str).unwrap();
        Ok(())
    }
}


#[derive(Serialize, Deserialize)]
pub struct Stack{
    pub cmd: String,
    pub icon: Option<String>
}
