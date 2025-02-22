use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::Write;
use color_print::cprintln;
use git2::Repository as GitRepository;
use colored::*;
use crate::git::Git;
use crate::registry::Registry;
use crate::git::GitTags;
use crate::stack::Stack;
use crate::vat_repository::VatRepo;

const VAT_TOML: &str = "vat.toml";


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
        Self { package_info: PackageInfo::from("".to_string()), dependencies: None, command: Some(HashMap::new()), environment: Some(HashMap::new()), examples: None }
    }
}

impl Package {  

    pub fn from_package_info(package_info: PackageInfo) -> Self{
        Self { package_info, dependencies: None, command: None, environment: None, examples: None }
    }


    pub fn default(
        name: String,   
    ) -> Self 
    {

        let _env = Environtment::new();

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

    pub fn append_env(&mut self, env_name: &str, env: Environtment) {
        if self.environment.is_some() {
            self.environment.as_mut().unwrap().insert(env_name.to_string(), env);
        }else{
            let mut environemts = HashMap::new();
            environemts.insert(env_name.to_string(), env);
            self.environment = Some(environemts);
        }
    }

    pub fn append_command(&mut self, command_name: &str, command: Command) {
        if self.command.is_some() {
            self.command.as_mut().unwrap().insert(command_name.to_string(), command);
        }else{
            let mut commands = HashMap::new();
            commands.insert(command_name.to_string(), command);
            self.command = Some(commands);
        }
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

    
    pub fn get_env(&self, env_name: &str) -> Option<&Environtment>{
        if self.environment.is_some() {
            let envs = self.environment.as_ref().unwrap();
            envs.get(env_name)
        }else{
            None
        }
    }

    pub fn get_cmd(&self, cmd_name: &str) -> Option<&Command>{
        if self.command.is_some(){
            let cmds = self.command.as_ref().unwrap();
            cmds.get(cmd_name)
        }else{
            None
        }
    }

    pub fn init(current_dir: PathBuf, package_name: Option<String>) -> Result<Self, anyhow::Error> {
            
        let mut directory = current_dir.clone();
        let mut folder_name = current_dir.file_name().unwrap_or_default().to_str().unwrap_or_default().to_string();

        if package_name.is_some() {
            directory = current_dir.join(package_name.clone().unwrap());
            folder_name = package_name.clone().unwrap();
            std::fs::create_dir_all(&directory)?;
        }



        let vat_yaml_path = directory.join(VAT_TOML);
        if vat_yaml_path.exists() {
            return Err(anyhow::anyhow!("{} already exists" , VAT_TOML));
        }

        // create yaml file
        let toml_string = toml::to_string(&Package::default(folder_name.to_string()))?;


        let git_repo = directory.join(".git");
        if !git_repo.exists() {
            let _repo = match GitRepository::init(&directory) {
                Ok(repo) => {
                    repo.git_ignore(&directory)?;
                    // repo.create_main_branch()?;
                }
                Err(e) => panic!("failed to init: {}", e),
            };
        }

        let mut toml_file = std::fs::File::create(vat_yaml_path)?;
        toml_file.write_all(toml_string.as_bytes())?;

        cprintln!("      <green>Created</green> vat package, `{}`", &folder_name);

        Ok(Self::default(folder_name.to_string()))

    }


    pub fn new(current_dir: PathBuf, package_name: String) -> Result<Self, anyhow::Error> {
        let package = Self::init(current_dir, Some(package_name))?;
        Ok(package)
    }


    pub fn read(package_path: &PathBuf) -> Result<Package, anyhow::Error> {
        let vat_toml_path = package_path.join(VAT_TOML);
        if !vat_toml_path.exists() {
            return Err(anyhow::anyhow!("{} given path is not a vat package", package_path.to_str().unwrap_or_default()));
        }
        let toml_string = std::fs::read_to_string(vat_toml_path)?;
        let package: Package = toml::from_str(&toml_string)?;
        Ok(package)
    }


    pub fn save(&self, package_path: &PathBuf) -> Result<Self, anyhow::Error> {
        let toml_string = toml::to_string(self)?;
        let mut toml_file = std::fs::File::create(package_path.join(VAT_TOML))?;
        toml_file.write_all(toml_string.as_bytes())?;
        Ok(self.clone())
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
        } else if patch{
            // Increment patch version
            self.package_info.version = semver::Version::new(major_version, minor_version, patch_version + 1);
        } 
    }

    pub fn set_version_message(&mut self, message: String) {
        self.package_info.version_message = Some(message);
    }

    pub fn parse_root_path(value: &str, root_path: &PathBuf) -> String{
        if value.contains("{root}") {
            let mut value = value.to_string();
            value = value.replace("{root}", root_path.to_str().unwrap());
            value
        }else{
            value.to_string()
        }
    }

    pub fn process_env(&self, environment_variables: &mut HashMap<String, String>, envs: Option<Vec<String>>, root_path: &PathBuf) {
        if envs.is_some(){
            for env_name in envs.unwrap(){
                let env = self.get_env(&env_name);
                if env.is_some(){
                    println!("Resolving Environment Variable: {}", env_name.yellow());
                    let env = env.unwrap();
                    let existing_env_values = if environment_variables.contains_key(&env.variable){
                        environment_variables.get(&env.variable).unwrap().clone()
                    }else{
                        std::env::var(&env.variable).unwrap_or_default()
                    };
                    match env.action{
                        Some(EnvAction::Prepend) => {
                            let new_env_value = Self::parse_root_path(&env.value, root_path);
                            let new_env_value = format!("{};{}", new_env_value, existing_env_values);
                            environment_variables.insert(env.variable.clone(), new_env_value.clone());
                            let message = format!("   Prepended {} - {}", env.variable.bright_cyan(), new_env_value.bright_black());
                            println!("{}",message);
                        }
                        Some(EnvAction::Append) => {
                            let new_env_value = Self::parse_root_path(&env.value, root_path);
                            let new_env_value = format!("{};{}", existing_env_values, new_env_value);
                            environment_variables.insert(env.variable.clone(), new_env_value.clone());
                            let message = format!("   Appended {} - {}", env.variable.bright_cyan(), new_env_value.bright_black());
                            println!("{}",message);
                        }
                        Some(EnvAction::Define) => {
                            let new_env_value = Self::parse_root_path(&env.value, root_path);
                            environment_variables.insert(env.variable.clone(), new_env_value.clone());
                            let message = format!("   Defined {} - {}", env.variable.bright_cyan(), new_env_value.bright_black());
                            println!("{}", message);
                        }
                        None => {}  
                    }
                }
            }
        }else{
            if self.environment.is_some(){
                let environmets = self.environment.as_ref().unwrap();
                for (env_name, env) in environmets {
                    println!("Resolving Environemnt Variable: {}", env_name.yellow());
                    let existing_env_values = if environment_variables.contains_key(&env_name.to_string()){
                        environment_variables.get(&env_name.to_string()).unwrap().clone()
                    }else{
                        std::env::var(&env_name.to_string()).unwrap_or_default()
                    };

                    match env.action{
                        Some(EnvAction::Prepend) => {
                            let new_env_value = Self::parse_root_path(&env.value, root_path);
                            let new_env_value = format!("{};{}", new_env_value, existing_env_values);
                            environment_variables.insert(env.variable.clone(), new_env_value.clone());
                            let message = format!("   Prepended {} - {}", env.variable.bright_cyan(), new_env_value.bright_black());
                            println!("{}",message);
                        }
                        Some(EnvAction::Append) => {
                            let new_env_value = Self::parse_root_path(&env.value, root_path);
                            let new_env_value = format!("{};{}", existing_env_values, new_env_value);
                            environment_variables.insert(env.variable.clone(), new_env_value.clone());
                            let message = format!("   Appended {} - {}", env.variable.bright_cyan(), new_env_value.bright_black());
                            println!("{}",message);
                        }
                        Some(EnvAction::Define) => {
                            let new_env_value = Self::parse_root_path(&env.value, root_path);
                            environment_variables.insert(env.variable.clone(), new_env_value.clone());
                            let message = format!("   Defined {} - {}", env.variable.bright_cyan(), new_env_value.bright_black());
                            println!("{}", message);
                        }
                        None => {}
                    }

                }
            }
        }

    }


    pub fn run_command(&self, command_name: &str, _root_path: &PathBuf, current_dir: Option<PathBuf>) -> Result<(), anyhow::Error> {
        
        if self.command.is_some() {
            let commands = self.command.as_ref().unwrap();
            if commands.contains_key(command_name) {
                let command = commands.get(command_name);
                if command.is_some() {
                    let script = command.unwrap().command.clone();
                    println!("Running Command script: {}", &script);

                    // println!("Current Environment Variables:");
                    // for (key, value) in std::env::vars() {
                    //     println!("{}: {}", key, value);
                    // }


                    let mut command = std::process::Command::new(&script);
                    command.env("NODURO", "PINKMAN");
                    // command.command_load_env(&self, command_name, root_path);
                    if current_dir.is_some(){
                        command.current_dir(current_dir.unwrap());
                    }

                    let _evn = command.get_envs();
                    // dbg!(&evn);




                    // command.env("HOUDINI_USER_PREF_DIR", "C:\\Users\\Deepak\\Documents\\houdini20.5");
                    // command.args(&["/C", "start", "cmd", "/K", &script]);
                    // command.args(&[&script]);
                    command.stdout(std::process::Stdio::null());
                    command.stderr(std::process::Stdio::null());
                    command.stdin(std::process::Stdio::null());

                    let _child = command.spawn();

                }
            }
        }
        Ok(())
    }

    pub fn clone_package(git_url: &str, package_path: &PathBuf) -> Result<Package, anyhow::Error> {
        // clone with progress
        let clone = std::process::Command::new("git").args(&["clone", git_url, &package_path.to_str().unwrap()]).output().unwrap();
        if !clone.status.success() {
            return Err(anyhow::anyhow!("Failed to clone package: {}", git_url));
        }
        let result_read_package = Package::read(&package_path);
        match result_read_package {
            Ok(package) => {
                // register the package
                let registry = Registry::init();
                match registry {
                    Ok(mut registry) => {
                        let result = registry.add_package(package.clone(), package_path.clone());
                        match result {
                            Ok(_) => Ok(package),
                            Err(e) => Err(e)
                        }
                    }
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn get_package_git_tags(package_path: &PathBuf) -> Option<Vec<String>> {
        if Self::is_vat_package(package_path) {
            let package = Package::read(package_path);
            match package {
                Ok(_) => {
                    let repo = GitRepository::open(package_path);
                    match repo {
                        Ok(repo) => {
                            let tags = repo.get_tags();
                            match tags {
                                Ok(tags) => Some(tags),
                                Err(_) => None
                            }
                        }
                        Err(_) => None
                    }
                }
                Err(_) => {
                    return None;
                }
            }
        }else{
            return None;
        }
    }

    pub fn get_package_latest_tag(package_path: &PathBuf) -> Option<String> { let tags = Self::get_package_git_tags(package_path); if tags.is_some(){ let git_tags = GitTags::new(tags.unwrap());
            if !git_tags.tags.is_empty(){
                let latest_tag = git_tags.get_latest();
                if latest_tag.is_some(){
                    return Some(latest_tag.unwrap().to_string());
                }
            }
        }
        None
    }


    // package_name is not just package name, it can take package name and version and env
    // vat run <subcommand> --package <package_name>/<version>[env1,evn2] --append <package_name>/<version>[env1,evn2]
    // var run <subcommand> will check for current directory for vat.toml file
    pub fn resolve_package(package_name: Option<String>, check_current_dir: bool) -> Result<PackageResolver, anyhow::Error>{
        if package_name.is_some(){
            let package_resolver = PackageResolver::parse_package_string(&package_name.unwrap());


            let vat_repo = VatRepo::init()?;
            let package = vat_repo.get_package(&package_resolver.unwrap());

            match package {
                Ok(package_resolver) => {
                    return Ok(package_resolver);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }else{
            if check_current_dir {
                let current_dir = std::env::current_dir();
                if current_dir.is_ok() {
                    let current_dir = current_dir.unwrap();
                    let package = Package::read(&current_dir);
                    match package {
                        Ok(package) => {

                            let package_resolver = PackageResolver::from_package(package,
                                current_dir,
                                PackageFrom::Main,
                                None);
                            return Ok(package_resolver);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }else{
                return Err(anyhow::anyhow!("Either package name should be provided or the current directory should be a vat package"));
            }
        }

        Err(anyhow::anyhow!("Failed to resolve package"))
    }


    pub fn run_stack(stack: Stack, current_dir: Option<PathBuf>) -> Result<(), anyhow::Error>{

        let package_name = format!("{}/{}", stack.package_name.unwrap(), stack.package_version.unwrap());
        let mut append_packages: Vec<String> = vec![];
        for append_package in stack.append{
            let package_name = format!("{}/{}", append_package.package_name, append_package.package_version);
            if append_package.env.len() > 0{
                let mut envs: String = String::new();
                for env in append_package.env{
                    envs.push_str(&env);
                    envs.push(',');
                }
                append_packages.push(format!("{}[{}]", package_name, envs));
            }else{
                append_packages.push(package_name);
            }
        }

        dbg!(&package_name);
        dbg!(&append_packages);

        Self::run(&stack.command.unwrap(), Some(package_name), Some(append_packages), true)?;
        
        Ok(())
    }



    pub fn run(command: &str, package:Option<String>, append: Option<Vec<String>>, detach: bool) -> Result<(), anyhow::Error>{

        let package = match package {
            Some(package) => {
                Package::resolve_package(Some(package), true)
            }
            None => {
                Package::resolve_package(None, true)
            }
        };

        if package.is_err(){
            let message = format!("{}", package.err().unwrap()); 
            return Err(anyhow::anyhow!(message.red()));
        }

        let package_resolver = package.unwrap();
        let package_root_path = package_resolver.package_path.unwrap();
        if package_resolver.package.is_none(){
            let message = format!("Failed to resolve the main package {}", package_resolver.package_name);
            return Err(anyhow::anyhow!(message.red()));
        }
        let package = package_resolver.package.unwrap();

        // Message
        let packge_version = package.package_info.version.to_string().clone();
        let message = format!("Package : {} - Version: {}", package_resolver.package_name, packge_version); println!("{}", message.green());
        let message = format!("Package Path : {}", package_root_path.to_str().unwrap()); println!("{}", message.green());


        let mut environment_variables: HashMap<String, String> = HashMap::new();
        let cmd = package.get_cmd(command);
        if cmd.is_none(){
            let message = format!("Command {} not found in package {}", command, package_resolver.package_name); 
            return Err(anyhow::anyhow!(message.red()));
        }

        let cmd = cmd.unwrap();
        if cmd.env.is_some(){
            package.process_env(&mut environment_variables, cmd.env.clone(), &package_root_path);
        }else{
            package.process_env(&mut environment_variables, None, &package_root_path);
        }
        

        let mut append_packages: Vec<PackageResolver> = vec![];
        if append.is_some(){
            for append_package in append.unwrap() {
                let package_resolver = Package::resolve_package(Some(append_package), true);

            match package_resolver {
                Ok(package_resolver) => {
                    append_packages.push(package_resolver);
                }
                Err(e) => {
                    let message = format!("Failed to resolve package: {}", e);
                    println!("{}", message.yellow());
                    }
                }
            }
        }


        // first go through append packages
        for append_package in append_packages {
            if append_package.package.is_some(){
                let package_root_path = append_package.package_path.unwrap();
                let package = append_package.package.unwrap();
                if append_package.env.is_some(){
                    package.process_env(&mut environment_variables, append_package.env.clone(), &package_root_path);
                }else{
                    package.process_env(&mut environment_variables, None, &package_root_path);
                }
            }
        }


        // run the command from main package
        let mut command_std = std::process::Command::new(&cmd.command);
        command_std.envs(environment_variables);
        if !detach{
            let message = format!("Running Command: {}", cmd.command);
            println!("{}", message.green());
            command_std.output().unwrap();  
        }else{
            let message = format!("Detaching the process");
            println!("{}", message.yellow());
            let message = format!("Running Command: {}", cmd.command);
            println!("{}", message.green());
            let child = command_std.spawn();
            match child{
                Ok(mut child) => {
                    let _ = child.wait();
                }
                Err(e) => {
                    let message = format!("{}: {}", cmd.command, e.to_string());
                    return Err(anyhow::anyhow!(message.red()));
                }
            }
            // if child.is_ok(){
            //     let _ = child.unwrap().wait();
            // }else{
            //     let message = format!("Failed to run command: {}", cmd.command);
            //     return Err(anyhow::anyhow!(message.red()));
            // }
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

impl Command{
    pub fn from(command: String, env: Option<Vec<String>>) -> Self {
        Self { command, env }
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

    pub fn from(variable: String, value: String, action: Option<EnvAction>) -> Self {
        Self { variable, value, action }
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




#[derive(Debug, Clone)]
pub struct PackageResolver{
    pub package_name: String,
    pub from: PackageFrom,
    pub package_path: Option<PathBuf>,
    pub package: Option<Package>,
    pub env: Option<Vec<String>>,
}


impl PackageResolver{

    pub fn new(package_name: String, from: PackageFrom) -> Self{
        Self { package_name, from, package_path: None, package: None, env: None }
    }

    pub fn from_package(package: Package, package_path: PathBuf, from: PackageFrom, env: Option<Vec<String>>) -> Self{
        Self { package_name: package.package_info.name.clone(), from, package_path: Some(package_path), package: Some(package), env }
    }

    pub fn parse_package_string(package_string: &str) -> Option<Self>{


        let mut package_resolver = PackageResolver{
            package_name: String::new(),
            from: PackageFrom::Main,
            package_path: None,
            package: None,
            env: None,

        };

        let (package_str, env_vars) = if let (Some(start), Some(end)) = (package_string.find('['), package_string.find(']')) {
            if start < end {  // Ensure valid bracket order
                let env_str = &package_string[start + 1..end];
                let package_part = &package_string[..start];
                (package_part.to_string(), Some(env_str.split(',').map(|s| s.trim().to_string()).collect::<Vec<String>>()))
            } else {
                (package_string.to_string(), None)
            }
        } else {
            (package_string.to_string(), None)
        };

        // dbg!(&package_str);
        // dbg!(&env_vars);


        let pattern = regex::Regex::new(r"^([a-zA-Z0-9-_]+)(?:/([a-zA-Z0-9.-]+))?$").unwrap();

        pattern.captures(&package_str).map(|caps| {
            package_resolver.package_name = caps.get(1).unwrap().as_str().to_string();
            package_resolver.from = caps.get(2)
                .map_or(PackageFrom::Latest, |m| match m.as_str() {

                    "latest" => PackageFrom::Latest,

                    s => match semver::Version::parse(s) {
                        Ok(version) => PackageFrom::Version(version),
                        Err(_) => PackageFrom::Main
                    }
                });
            package_resolver.env = env_vars;
            package_resolver
        })


    }
    
}



#[derive(Debug, Clone)]
pub enum PackageFrom{
    Latest,
    Version(semver::Version),
    Main,
}



