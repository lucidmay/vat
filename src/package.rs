use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Package{
    #[serde(rename="package")]
    pub package_info: PackageInfo,
    pub dependencies: Vec<String>,
    #[serde(rename="commands")]
    pub commands: Vec<BuildCommand>,
    pub environments: Vec<Environtment>,
}

impl Package {  
    pub fn new(
        name: String,   
    ) -> Self 
    {

        
        Self { 
            package_info: PackageInfo { name, version: "0.0.1".to_string(), description: "".to_string(), authors: vec![] },
            dependencies: vec![],
            commands: vec![],
            environments: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
}



#[derive(Serialize, Deserialize, Debug)]
pub struct BuildCommand{
    pub command: String,
    pub args: Vec<String>,
}


impl BuildCommand {
    pub fn new() -> Self {
        Self { command: "cargo build".to_string(), args: vec![] }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Environtment {
    pub name: String,
    pub value: String,
    pub action: EnvAction,
}


#[derive(Serialize, Deserialize, Debug)]
pub enum EnvAction {
    Prepend,
    Append,
    Define,
}

