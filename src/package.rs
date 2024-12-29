use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::Write;
use color_print::cprintln;
use git2::Repository;

const VAT_TOML: &str = "vat.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageVersions{
    pub publishes: HashMap<String, Package>,
    pub default: String,
}

impl Default for PackageVersions {
    fn default() -> Self {
        PackageVersions { publishes: HashMap::new(), default: "".to_string() }
    }
}

impl PackageVersions {
    pub fn append_version(&mut self, package: Package) {
        self.publishes.insert(package.get_version(), package);
    }

    pub fn from(package: Package) -> Self {
        let mut package_versions = PackageVersions{
            publishes: HashMap::new(),
            default: "".to_string(),
        };
        package_versions.append_version(package);
        package_versions
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package{
    #[serde(rename="package")]
    pub package_info: PackageInfo,
    pub dependencies: Dependencies,
    pub environment: Option<HashMap<String, Environtment>>,
    pub command: Option<HashMap<String, Command>>,
}

impl Package {  
    pub fn new(
        name: String,   
    ) -> Self 
    {
        let current_dir = std::env::current_dir().unwrap();

        let env = Environtment::new();

        let command = Command { script: "houdini".to_string(), env: Some(vec!["PATH".to_string(), "PYTHONPATH".to_string()]) };

        Self { 
            package_info: PackageInfo { name,
                version: "0.0.0".to_string(),
                description: "".to_string(),
                authors: vec![] ,
                repository: RepositoryType::Local(current_dir),
                },
            dependencies: Dependencies::default(),
            command: Some(HashMap::from([("houdini20".to_string(), command)])),
            environment: Some(HashMap::from([("python".to_string(), env)])),
        }
    }

    pub fn get_name(&self) -> String {
        self.package_info.name.clone()
    }

    pub fn get_version(&self) -> String {
        self.package_info.version.clone()
    }


    pub fn init(current_dir: PathBuf) -> Result<Self, anyhow::Error> {
              cprintln!("      <green>Creating</green> vat default package");

        // get the current working directory
        let current_dir_name = current_dir.file_name().unwrap().to_str().unwrap();

        match current_dir.is_empty() {
            false => return Err(anyhow::anyhow!("The current directory is not empty")),
            true => (),
        }

        let vat_yaml_path = current_dir.join(VAT_TOML);
        if vat_yaml_path.exists() {
            return Err(anyhow::anyhow!("{} already exists, looks like you already initialized the package", VAT_TOML));
        }

        // create yaml file
        let toml_string = toml::to_string(&Package::new(current_dir_name.to_string()))?;

        let mut toml_file = std::fs::File::create(vat_yaml_path)?;
        toml_file.write_all(toml_string.as_bytes())?;

        let repo = match Repository::init(&current_dir) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to init: {}", e),
        };

        Ok(Self::new(current_dir_name.to_string()))

    }


    pub fn read(package_path: &PathBuf) -> Result<Package, anyhow::Error> {
        let toml_string = std::fs::read_to_string(package_path.join(VAT_TOML))?;
        let package: Package = toml::from_str(&toml_string)?;
        Ok(package)
    }


    pub fn save(&self, package_path: &PathBuf) -> Result<(), anyhow::Error> {
        let toml_string = toml::to_string(self)?;
        let mut toml_file = std::fs::File::create(package_path.join(VAT_TOML))?;
        toml_file.write_all(toml_string.as_bytes())?;
        Ok(())
    }


    pub fn is_vat_package(package_path: &PathBuf ) -> bool {
        let vat_yaml_path = package_path.join(VAT_TOML);
        vat_yaml_path.exists()
    }
    

    pub fn is_vat_package_dir(package_path: &PathBuf ) -> bool {
        let vat_yaml_path = package_path.join(VAT_TOML);
        vat_yaml_path.exists()
    }


    pub fn get_current_version(&self) -> String{
        self.package_info.version.clone()
    }


    pub fn increment_version(&mut self, major: bool, minor: bool, patch: bool) {
        let version_parts: Vec<&str> = self.package_info.version.split('.').collect();
    
        let major_version = version_parts[0].parse::<i32>().unwrap();
        let minor_version = version_parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let patch_version = version_parts.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);

        if major {
            // Increment major version and reset minor and patch
            self.package_info.version = format!("{}.0.0", major_version + 1);
        } else if minor {
            // Increment minor version and reset patch
            self.package_info.version = format!("{}.{}.0", major_version, minor_version + 1);
        } else {
            // Increment patch version
            self.package_info.version = format!("{}.{}.{}", major_version, minor_version, patch_version + 1);
        } 
    }


    pub fn load_environments(&self, env_name: &str) -> Result<(), anyhow::Error> {
        if self.environment.is_some() {
            let env = self.environment.as_ref().unwrap();
            if env.contains_key(env_name) {
                let env = env.get(env_name).unwrap();
                if env.action.is_some() {
                    if env.variable == "PATH"{
                        let path = PathBuf::from(env.value.clone());
                        if !path.exists() {
                            return Err(anyhow::anyhow!("The path is not valid: {}", env.value));
                        }
                    }
                    match env.action.as_ref().unwrap() {
                        EnvAction::Prepend => {
                            let current_value = std::env::var(env.variable.clone()).unwrap_or_default();
                            std::env::set_var(env.variable.clone(), format!("{};{}", env.value, current_value));
                        }
                        EnvAction::Append => {
                            let current_value = std::env::var(env.variable.clone()).unwrap_or_default();
                            std::env::set_var(env.variable.clone(), format!("{};{}", current_value, env.value));
                        }
                        EnvAction::Define => {
                            std::env::set_var(env.variable.clone(), env.value.clone());
                        }
                    }
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!("Environment not found: {}", env_name))
            }
        } else {
            Err(anyhow::anyhow!("No environments found in the package"))
        }
    }

    pub fn run_command(&self, command_name: &str) -> Result<(), anyhow::Error> {
        if self.command.is_some() {
            let commands = self.command.as_ref().unwrap();
            if commands.contains_key(command_name) {
                let command = commands.get(command_name);
                if command.is_some() {
                    println!("Command found: {}", &command_name);
                    let command = command.unwrap();
                    // check if env is defined
                    if command.env.is_some() {
                        for env in command.env.as_ref().unwrap() {
                            println!("Loading environment variable: {}", &env);
                            self.load_environments(env)?;
                        }
                    }else{
                        // load all environemts
                        if self.environment.is_some() {
                            let env = self.environment.as_ref().unwrap();
                            for (key, value) in env {
                                println!("Loading environment: {}", &key);
                                self.load_environments(key)?;
                            }
                        }
                    }
                    let script = command.script.clone();
                    println!("Running script: {}", &script);
                    let status = std::process::Command::new(&script).status()?;
                    if !status.success() {
                        return Err(anyhow::anyhow!("Command failed: {}", script));
                    }
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Command not found: {}", command_name))
                }
            } else {
                Err(anyhow::anyhow!("Command not found: {}", command_name))
            }
        } else {
            Err(anyhow::anyhow!("No commands found in the package"))
        }
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub repository: RepositoryType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RepositoryType {
    Local(PathBuf),
    Remote(String),
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub script: String,
    pub env: Option<Vec<String>>
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildCommand{
    pub command: String,
    pub args: Vec<String>,
}


impl BuildCommand {
    pub fn new() -> Self {
        Self { command: "cargo build".to_string(), args: vec![] }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Environtment {
    pub variable: String,
    pub value: String,
    pub action: Option<EnvAction>,
}

impl Environtment {
    pub fn new() -> Self {
        Self { variable: "PATH".to_string(), value: "{root}/bin".to_string(), action: Some(EnvAction::Define) }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnvAction {
    Prepend,
    Append,
    Define,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependencies {
    pub dependencies: Vec<String>,
}

impl Default for Dependencies {
    fn default() -> Self {
        Self { dependencies: vec![] }
    }
}

trait Path {
    fn is_empty(&self) -> bool;
}

impl Path for PathBuf {
    fn is_empty(&self) -> bool {
        let files = std::fs::read_dir(self).unwrap();
        if files.count() == 0 {
            return true;
        }
        false
    }
}

