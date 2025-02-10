use std::path::{PathBuf, Path};
use std::fs;
use crate::config::VatConfig;
use serde::{Serialize, Deserialize};
use color_print::cprintln;
use std::collections::HashMap;
use crate::package::{Package, PackageVersions};
use std::io::Write;
use fs_extra::dir::CopyOptions;
use zip::read::ZipArchive;
use std::fs::File;
use std::io;
use colored::*;
use semver::Version;
use crate::stack::Stack;
use crate::registry::Registry;
use crate::stack::ExecuteFrom;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VatRepository2{
    pub packages: HashMap<String, PublishedVersions>,
    pub path : PathBuf,
}

impl Default for VatRepository2{
    fn default() -> Self{
        VatRepository2{packages: HashMap::new(), path: PathBuf::new()}
    }
}


impl VatRepository2{
    pub fn initalize_repository(repository_path: &PathBuf) -> Result<Self, anyhow::Error>{
        let repository_config_file = repository_path.join("vat.repository.toml");
        let vat_repo = VatRepository2{
            packages: HashMap::new(),
            path: repository_path.clone(),
        };
        let repository_config_str = toml::to_string(&vat_repo)?;
        fs::write(&repository_config_file, &repository_config_str)?;
        Ok(vat_repo)
    }


    pub fn init() -> Result<Self, anyhow::Error>{
        let current_dir = std::env::current_dir()?;

        let vat_repo = VatRepository2{
            packages: HashMap::new(),
            path: current_dir.clone(),
        };

        let mut config = VatConfig::init()?;
        let repository_path = config.get_repository_path();
        let repository_config_file = current_dir.join("vat.repository.toml");

        if repository_path.is_none(){
            if !current_dir.is_empty(){
                return Err(anyhow::anyhow!("Current directory is not empty to initialize the repository"));
            }else{
                let repository_config_str = toml::to_string(&vat_repo)?;
                fs::write(&repository_config_file, &repository_config_str)?;
                config.set_repository_path(current_dir.clone());
                config.save()?;
                cprintln!("      <green>Repository initialized</green>");
            }
        }else{
            let repository_path = config.get_repository_path().unwrap();
            if !repository_path.exists(){
                let repository_config_str = toml::to_string(&vat_repo)?;
                fs::write(&repository_config_file, &repository_config_str)?;
                config.set_repository_path(repository_path.clone());
                config.save()?;
                cprintln!("      <green>Repository initialized</green>");
            }else{
                if repository_path.is_empty(){
                    let repository_config_str = toml::to_string(&vat_repo)?;
                    fs::write(&current_dir.join("vat.repository.toml"), &repository_config_str)?;
                    config.set_repository_path(repository_path.clone());
                    config.save()?;
                    cprintln!("      <green>Repository initialized</green>");
                }else{
                    return Err(anyhow::anyhow!("Repository already exists, {}", repository_path.display()));
                }
            }   
        }
        Ok(VatRepository2::default())
    }
        

    pub fn add_package(&mut self, package: Package, message: Option<String>, current_dir: PathBuf) -> Result<(), anyhow::Error>{
        if self.packages.contains_key(&package.get_name().to_string()){
            let package_versions = self.packages.get_mut(&package.get_name().to_string()).unwrap();
            if package_versions.versions.contains_key(&package.get_version()){
                return Err(anyhow::anyhow!("Package version has already been published, {}", package.get_version()));
            }else{
                package_versions.versions.insert(package.get_version().clone(), message.unwrap());
                Ok(())
            }
        }else{
            if message.is_none(){
                return Err(anyhow::anyhow!("Message is required, to add a package to the repository `vat publish -m \"message\"`"));
            }
            self.packages.insert(package.get_name().to_string(), PublishedVersions::from(package.clone(), message.unwrap(), current_dir, None));
            Ok(())
        }   
    }

    pub fn package_exists(&self, package_name: &str) -> bool{
        self.packages.contains_key(package_name)
    }

    pub fn get_package_path(&self, package_name: &str) -> Option<PathBuf>{
        if self.packages.contains_key(package_name){
            let package_path = self.packages.get(package_name)?.package_path.clone();
            Some(package_path)
        }else{
            None
        }
    }

    pub fn pretty_list(&self) {
        if !self.packages.is_empty() {
            for (package_name, package_versions) in &self.packages {
                println!("{}", package_name);
                let sorted_versions = package_versions.versions.iter()
                    .collect::<Vec<(&semver::Version, &String)>>();    

                for (version, message) in sorted_versions {
                    println!("  {} - {}", version, message);
                }
            }
        } else {
            println!("{}", format!("No packages found in the repository"));
        }
    }

    pub fn remove_package(&mut self, package_name: &str) -> Result<(), anyhow::Error>{
        if self.packages.contains_key(package_name){
            self.packages.remove(package_name);
            fs::remove_dir_all(&self.path.join("packages").join(package_name))?;
            self.save()?;
            Ok(())
        }else{
            Err(anyhow::anyhow!("Package not found: {}", package_name))
        }
    }

    pub fn get_package(&self, package_name: &str, version: &str) -> Result<Package, anyhow::Error>{
        if self.packages.contains_key(package_name){
            let package_path = self.path.join("packages").join(package_name).join(version);
            let package = Package::read(&package_path);
            match package{
                Ok(package) => {
                    Ok(package)
                }
                Err(e) => {
                    Err(anyhow::anyhow!("Error reading package: {}", e))
                }
            }
        }else{
            Err(anyhow::anyhow!("Package not found: {}", package_name))
        }
    }

    pub fn resolve_package_version(&self, package_name: &str) -> Result<Package, anyhow::Error>{
        if package_name.contains("/"){
            let package_and_version = package_name.split_once("/");
            if package_and_version.is_some(){
                let (package_name, version) = package_and_version.unwrap();
                let package = self.get_package(package_name, version);
                match package{
                    Ok(package) => Ok(package),
                    Err(e) => {
                        Err(e)
                    }
                }
            }else{
                Err(anyhow::anyhow!("Invalid package name: {}", package_name))
            }
        }else{
            let package = self.get_latest_package(package_name);
            match package{
                Some(package) => Ok(package),
                None => {
                    cprintln!("{}", format!("Package not found with version: {}", package_name).red());
                    cprintln!("{}", format!("Fallback to latest version").yellow());
                    Err(anyhow::anyhow!("Package not found: {}", package_name))
                }
            }
        }
    }

    pub fn resolve_package_version_option(&self, package_name: &str) -> Option<Package>{
        let package = self.resolve_package_version(package_name);
        match package{
            Ok(package) => Some(package),
            Err(_) => None
        }
    }


    pub fn get_latest_package(&self, package_name: &str) -> Option<Package>{
        if self.packages.contains_key(package_name){
            let latest_version = self.get_latest_version(package_name);
            let package_path = self.path.join("packages").join(package_name).join(latest_version.unwrap().to_string());
            let package = Package::read(&package_path);
            match package {
                Ok(package) => {
                    cprintln!("{}", format!("Package loaded: {}", package_name).green());
                    Some(package)
                }
                Err(e) => {
                    eprintln!("{}", format!("Error reading package: {}", e));
                    None
                }
            }
        }else{
            cprintln!("{}", format!("Package not found: {}", package_name).red());
            None
        }
    }

    pub fn get_latest_version(&self, package_name: &str) -> Option<semver::Version>{
        let version_list = self.packages.get(package_name)?.versions.keys().cloned().collect::<Vec<semver::Version>>();
        // Use semver to find the latest version
        let latest_version = version_list.iter()
            .max() // Get the maximum version
            .cloned(); // Convert back to String
        
        latest_version
    }

    pub fn get_latest_package_path(&self, package_name: &str) -> Option<PathBuf>{
        if self.packages.contains_key(package_name){
            let latest_version = self.get_latest_version(package_name);
            let package_path = self.path.join("packages").join(package_name).join(latest_version.unwrap().to_string());
            Some(package_path)
        }else{
            None
        }
    }


    pub fn read_repository() -> Result<Self, anyhow::Error> {
        let vat_config = VatConfig::init()?;
        let repository_path = vat_config.get_repository_path();
        let repository_path = if let Some(repository_path) = repository_path {
            repository_path
        } else {
            return Err(anyhow::anyhow!("Repository path not found, Run vat repo-init to initialize the repository"));
        };
        dbg!(&repository_path);
        let repository_config_str = fs::read_to_string(repository_path.join("vat.repository.toml"))?;
        let repository_config: VatRepository2 = toml::from_str(&repository_config_str)?;
        Ok(repository_config)
    }

    pub fn get_package_main_path(&self, package_name: &str) -> Option<PathBuf>{
        if self.packages.contains_key(package_name){
            let package_path = self.packages.get(package_name)?.package_path.clone();
            Some(package_path)
        }else{
            None
        }
    }


    pub fn package_data_update(&self, package: &Package, git_dir: PathBuf) -> Result<(), anyhow::Error> {
        let repository_path = self.path.clone();
        let package_path = repository_path.join("packages").join(&package.get_name());

        let current_dir = git_dir;

        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;  

        let currect_version = package.get_version();
        let zip_file_name = format!("{}.zip", currect_version);
        let source_zip_file = current_dir.join(&zip_file_name);


        println!("Source: {:?}", &zip_file_name);
        println!("Target: {:?}", &package.get_version());


        //"git archive --format=zip -o archive.zip 0.0.3"

        let command = std::process::Command::new("git")
            .arg("archive")
            .arg("--format=zip")
            .arg("-o")
            .arg(&zip_file_name)
            .arg(package.get_version().to_string())
            .current_dir(&current_dir)
            .status()
            .expect("Failed to execute git archive");


        let mut source = Vec::new();
        source.push(&source_zip_file);

        println!("Source: {:?}", &source);

        let target_path = package_path.join(package.get_version().to_string());
        println!("Target: {:?}", &target_path);
        if !target_path.exists(){
            fs::create_dir_all(&target_path)?;
        }
        // copy_items(&source, &target_path, &options)?;


        unzip_file(&source_zip_file.to_str().unwrap(), &target_path.to_str().unwrap())?;
        fs::remove_file(source_zip_file)?;

        println!("Zip successfully extracted");

        Ok(())
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let repository_config_str = toml::to_string(self)?;
        dbg!(&repository_config_str);
        dbg!(&self.path);
        fs::write(&self.path.join("vat.repository.toml"), &repository_config_str)?;
        Ok(())

    }


    // this is to use on tauri app
    // TODO: just testing for now
    pub fn run_command(stack: &Stack, current_dir: Option<PathBuf>) -> Result<(), anyhow::Error>{

        let vat_repository = VatRepository2::read_repository()?;

        match stack.execute_from{
            ExecuteFrom::Registry => {
                let registry = Registry::init()?;
                let package = registry.read_package(&stack.package_name)?;
            }
            ExecuteFrom::Repository => {

            }
        }

        let package_name = &stack.package_name;
        let command_name = &stack.cmd;
        let package = vat_repository.resolve_package_version_option(package_name);
        dbg!(&package);
        if let Some(package) = package{
            dbg!(&package);
            let latest_package_path = vat_repository.get_latest_package_path(package_name).unwrap();

            // let env_load = package.command_load_env(&command_name, &latest_package_path);
            let command = package.run_command(&command_name, &latest_package_path, current_dir);
            match command{
                Ok(_) => {
                    Ok(())
                }
                Err(e) => {
                    Err(e)
                }
            }

        }else{
            Err(anyhow::anyhow!("Package not found: {}", package_name))
        }

    }

}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublishedVersions{
    pub versions: HashMap<Version, String>,
    pub package_path: PathBuf,
    pub repository: Option<PathBuf>
}

impl PublishedVersions{
    pub fn from(package: Package, message: String, package_path: PathBuf, repository: Option<PathBuf>) -> Self{
        let version  = HashMap::from([(package.get_version().clone(), message)]);
        PublishedVersions{versions: version, package_path, repository}
    }
}



trait PathUtil {
    fn is_empty(&self) -> bool;
}

impl PathUtil for PathBuf {
    fn is_empty(&self) -> bool {
        let files = std::fs::read_dir(self).unwrap();
        if files.count() == 0 {
            return true;
        }
        false
    }
}


fn unzip_file(zip_path: &str, output_dir: &str) -> io::Result<()> {
    // Open the ZIP file
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Extract all files in the ZIP archive
    archive.extract(Path::new(output_dir))?;

    println!("Successfully extracted to {}", output_dir);
    Ok(())
}
