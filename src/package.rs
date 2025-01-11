use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::Write;
use color_print::cprintln;
use git2::Repository;
use colored::*;
use crate::git::Git;

const VAT_TOML: &str = "vat.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageVersions{
    pub publishes: HashMap<semver::Version, Package>,
    pub default: semver::Version,

}


impl Default for PackageVersions {
    fn default() -> Self {
        PackageVersions { publishes: HashMap::new(), default: semver::Version::new(0, 0, 0) }
    }
}

impl PackageVersions {
    pub fn append_version(&mut self, package: Package) {
        self.publishes.insert(package.get_version().clone(), package);
    }

    pub fn from(package: Package) -> Self {
        let mut package_versions = PackageVersions{
            publishes: HashMap::new(),
            default: semver::Version::new(0, 0, 0),
        };
        package_versions.append_version(package);
        package_versions
    }

    pub fn get_latest_version(&self) -> Option<&Package> {
        let latest_version = self.publishes.values().max_by(|a, b| a.get_version().cmp(&b.get_version()));
        latest_version
    }


}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package{
    #[serde(rename="package")]
    pub package_info: PackageInfo,
    pub dependencies: Option<Dependencies>,
    pub environment: Option<HashMap<String, Environtment>>,
    pub command: Option<HashMap<String, Command>>,
    pub examples: Option<Vec<Example>>
}

impl Default for Package {
    fn default() -> Self {
        Self { package_info: PackageInfo::from("".to_string()), dependencies: None, command: None, environment: None, examples: None }
    }
}

impl Package {  
    pub fn default(
        name: String,   
    ) -> Self 
    {

        let env = Environtment::new();

        // let command = Command { command: "app.exe".to_string(), env: Some(vec!["PATH".to_string(), "PYTHONPATH".to_string()]) };

        Self { 
            package_info: PackageInfo::from(name),
            dependencies: None,
            command: None,
            environment: None,
            examples: None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.package_info.name
    }

    pub fn get_version(&self) -> &semver::Version {
        &self.package_info.version
    }

    pub fn get_version_message(&self) -> Option<&str> {
        self.package_info.version_message.as_ref().map(|s| s.as_str())
    }

    pub fn list_commands(&self){
        // print the commands
        if self.command.is_some() {
            let commands = self.command.as_ref().unwrap();
            for (key, value) in commands {
                println!("{}: {}", key, value.command);
            }
        }else{
            eprintln!("{}", format!("No commands found").red());
        }
    }


    pub fn init(current_dir: PathBuf, package_name: Option<String>) -> Result<Self, anyhow::Error> {
            
        let mut directory = current_dir.clone();
        let mut folder_name = current_dir.file_name().unwrap().to_str().unwrap().to_string();

        if package_name.is_some() {
            directory = current_dir.join(package_name.clone().unwrap());
            folder_name = package_name.clone().unwrap();
            std::fs::create_dir_all(&directory)?;
        }

        cprintln!("      <green>Creating</green> vat package, `{}`", &folder_name);


        let vat_yaml_path = directory.join(VAT_TOML);
        if vat_yaml_path.exists() {
            return Err(anyhow::anyhow!("{} already exists, looks like you already initialized the package", VAT_TOML));
        }

        // create yaml file
        let toml_string = toml::to_string(&Package::default(folder_name.to_string()))?;

        let mut toml_file = std::fs::File::create(vat_yaml_path)?;
        toml_file.write_all(toml_string.as_bytes())?;

        let git_repo = directory.join(".git");
        if !git_repo.exists() {
            let repo = match Repository::init(&directory) {
                Ok(repo) => {
                    repo.git_ignore(&directory)?;
                    // repo.create_main_branch()?;
                }
                Err(e) => panic!("failed to init: {}", e),
            };
        }

        Ok(Self::default(folder_name.to_string()))

    }


    pub fn new(current_dir: PathBuf, package_name: String) -> Result<Self, anyhow::Error> {
        let package = Self::init(current_dir, Some(package_name))?;
        Ok(package)
    }


    pub fn read(package_path: &PathBuf) -> Result<Package, anyhow::Error> {
        let vat_toml_path = package_path.join(VAT_TOML);
        if !vat_toml_path.exists() {
            return Err(anyhow::anyhow!("{} not found", VAT_TOML));
        }
        let toml_string = std::fs::read_to_string(vat_toml_path)?;
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


    pub fn get_current_version(&self) -> semver::Version{
        self.package_info.version.clone()
    }


    pub fn increment_version(&mut self, major: bool, minor: bool, patch: bool) {
        let version_parts = self.package_info.version.clone();
        let major_version = version_parts.major;
        let minor_version = version_parts.minor;
        let patch_version = version_parts.patch;

        if major {
            // Increment major version and reset minor and patch
            self.package_info.version = semver::Version::new(major_version + 1, 0, 0);
        } else if minor {
            // Increment minor version and reset patch
            self.package_info.version = semver::Version::new(major_version, minor_version + 1, 0);
        } else {
            // Increment patch version
            self.package_info.version = semver::Version::new(major_version, minor_version, patch_version + 1);
        } 
    }

    pub fn set_version_message(&mut self, message: String) {
        self.package_info.version_message = Some(message);
    }

    pub fn load_all_environments(&self, root_path: &PathBuf) -> Result<(), anyhow::Error> {
        if self.environment.is_some() {
            let env = self.environment.as_ref().unwrap();
            for (key, value) in env {
                self.load_environments(key, root_path)?;
            }
        }
        Ok(())
    }


    pub fn load_environments(&self, env_name: &str, root_path: &PathBuf) -> Result<(), anyhow::Error> {
        if self.environment.is_some() {
            let env = self.environment.as_ref().unwrap();
            if env.contains_key(env_name) {
                let env = env.get(env_name).unwrap();

                let new_env_value = env.value.clone().replace("{root}", &root_path.to_str().unwrap());

                if env.action.is_some() {

                    if env.variable == "PATH"{
                        let path = PathBuf::from(new_env_value.clone());
                        if !path.exists() {
                            return Err(anyhow::anyhow!("The path is not valid: {}", new_env_value));
                        }
                    }

                    match env.action.as_ref().unwrap() {
                        EnvAction::Prepend => {
                            let current_value = std::env::var(env.variable.clone());
                            match current_value {   
                                Ok(value) => {
                                    std::env::set_var(env.variable.clone(), format!("{};{}", new_env_value, value));
                                }
                                Err(_) => {
                                    std::env::set_var(env.variable.clone(), new_env_value);
                                }
                            }
                        }

                        EnvAction::Append => {
                            let current_value = std::env::var(env.variable.clone());
                            match current_value {
                                Ok(value) => {
                                    std::env::set_var(env.variable.clone(), format!("{};{}", value, new_env_value));
                                }
                                Err(_) => {
                                    std::env::set_var(env.variable.clone(), new_env_value);
                                }
                            }
                        }

                        EnvAction::Define => {
                            std::env::set_var(env.variable.clone(), new_env_value);
                        }
                    }

                    println!("{}: {}", env.variable, std::env::var(env.variable.clone()).unwrap_or_default());
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!("Environment action is not defined"))
            }
        } else {
            Err(anyhow::anyhow!("No environments found in the package"))
        }
    }


    pub fn command_load_env(&self, command_name: &str, root_path: &PathBuf) -> Result<(), anyhow::Error> {
        if self.command.is_some() {
            let commands = self.command.as_ref().unwrap();
            if commands.contains_key(command_name) {
                let command = commands.get(command_name);
                if let Some(command) = command {
                    if command.env.is_some() {
                        for env in command.env.as_ref().unwrap() {
                            println!("Loading environment variable: {} \n", &env);
                            self.load_environments(env, root_path)?;
                        }
                    }else{
                        // load all environemts
                        if self.environment.is_some() {
                            let env = self.environment.as_ref().unwrap();
                            for (key, value) in env {
                                println!("Loading environment: {} \n", &key);
                                self.load_environments(key, root_path)?;
                            }
                        }
                    }
                }else{
                    cprintln!("{}", format!("Command not found: {}", command_name).red());
                }
            }else{
                cprintln!("{}", format!("Command not found: {}", command_name).red());
            }
        }
        Ok(())
    }


    pub fn run_only_command(&self, command_name: &str) -> Result<(), anyhow::Error> {
        
        if self.command.is_some() {
            let commands = self.command.as_ref().unwrap();
            if commands.contains_key(command_name) {
                let command = commands.get(command_name);
                if command.is_some() {
                    let script = command.unwrap().command.clone();
                    println!("Running script: {}", &script);
                    let status = std::process::Command::new(&script).status()?;
                    if !status.success() {
                        return Err(anyhow::anyhow!("Command failed: {}", script));
                    }
                }
            }
        }
        Ok(())
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: semver::Version,
    pub version_message: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub edition: Option<String>,
    pub documentation: Option<PathBuf>,
    pub readme: Option<PathBuf>,
    pub license: Option<String>,
    pub license_file: Option<PathBuf>,
    pub build: Option<PathBuf>,
    pub include: Option<Vec<PathBuf>>,
    pub exclude: Option<Vec<PathBuf>>,
    pub metadata: Option<HashMap<String, String>>,
    pub keywords: Option<Vec<String>>,

}

impl PackageInfo{
    pub fn from(name: String) -> Self {
        Self { name,
             version: semver::Version::new(0, 0, 0),
            version_message: None,
            description: None, 
            authors: vec![],
            repository: None,
            edition: None,
            documentation: None,
            readme: None,
            license: None,
            license_file: None,
            build: None,
            include: None,
            exclude: None,
            metadata: None,
            keywords: None
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Example {
    pub name: String,
    pub path: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub command: String,
    pub env: Option<Vec<String>>
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

