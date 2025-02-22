use crate::config::VatConfig;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Stacks{
    pub stacks: HashMap<String, Stack>,
    pub order: Vec<String>
}

impl Stacks{

    pub fn stacks_config_path() -> PathBuf{
        let config = VatConfig::get_app_dir();
        let config_path = config.unwrap().join("stacks.toml");
        config_path
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

    pub fn save(&self) -> Result<(), anyhow::Error>{
        let config_path = Stacks::stacks_config_path();
        let config_str = toml::to_string(&self).unwrap();
        fs::write(config_path, config_str).unwrap();
        Ok(())
    }

    pub fn save_as(stacks: &Stacks) -> Result<(), anyhow::Error>{
        let config_path = Stacks::stacks_config_path();
        let config_str = toml::to_string(&stacks).unwrap();
        fs::write(config_path, config_str).unwrap();
        Ok(())
    }


    pub fn append_stack(&mut self, stack: Stack) -> Result<(), anyhow::Error>{
        let name = stack.name.clone();
        self.stacks.insert(name.clone(), stack);
        self.order.push(name);
        self.save()?;
        Ok(())
    }

    pub fn remove_stack(&mut self, name: &str) -> Result<(), anyhow::Error>{
        self.stacks.remove(name);
        self.order.remove(self.order.iter().position(|r| r == name).unwrap());
        self.save()?;
        Ok(())
    }

    pub fn update_stack(&mut self, name: &str, stack: Stack) -> Result<(), anyhow::Error>{
        self.stacks.insert(name.to_string(), stack);
        self.save()?;
        Ok(())
    }

    pub fn get_stack(&self, name: &str) -> Option<&Stack>{
        self.stacks.get(name)
    }

    pub fn get_order(&self) -> &Vec<String>{
        &self.order
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stack{
    pub name: String,
    pub icon: Option<String>,
    pub icon_id: Option<String>,
    pub package_name: Option<String>,
    pub package_version: Option<String>,
    pub command: Option<String>,
    pub append: Vec<AppendStackPackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppendStackPackage{
    pub package_name: String,
    pub package_version: String,
    pub env: Vec<String>,
}



