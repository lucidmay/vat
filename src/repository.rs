use std::path::{PathBuf, Path};
use std::fs;
use crate::config::VatConfig;
use serde::{Serialize, Deserialize};
use color_print::cprintln;
use std::collections::HashMap;
use crate::package::{Package, PackageVersions};
use std::io::Write;
use fs_extra::{copy_items, dir::{copy, CopyOptions}};
use zip::read::ZipArchive;
use std::fs::File;
use std::io;
use colored::*;

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

    pub fn add_package(&mut self, package: Package, message: String, current_dir: PathBuf) -> Result<(), anyhow::Error>{
        if self.packages.contains_key(&package.get_name()){
            let package_versions = self.packages.get_mut(&package.get_name()).unwrap();
            if package_versions.versions.contains_key(&package.get_version()){
                return Err(anyhow::anyhow!("Package version has already been published, {}", package.get_version()));
            }else{
                package_versions.versions.insert(package.get_version(), message);
                Ok(())
            }
        }else{
            self.packages.insert(package.get_name(), PublishedVersions::from(package.clone(), message, current_dir, None));
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
        fs::write(&self.path.join("vat.repository.toml"), &repository_config_str)?;
        Ok(())
    }


}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublishedVersions{
    pub versions: HashMap<semver::Version, String>,
    pub package_path: PathBuf,
    pub repository: Option<PathBuf>
}

impl PublishedVersions{
    pub fn from(package: Package, message: String, package_path: PathBuf, repository: Option<PathBuf>) -> Self{
        let version  = HashMap::from([(package.get_version(), message)]);
        PublishedVersions{versions: version, package_path, repository}
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VatRepository {
    pub packages: HashMap<String, PackageVersions>
}

impl Default for VatRepository {
    fn default() -> Self {
        VatRepository { packages: HashMap::new() }
    }
}

impl VatRepository {
    pub fn new() -> Self{
        VatRepository::default()
    }

    pub fn init() -> Result<Self, anyhow::Error> {
        let mut config = VatConfig::init()?;
        let repository_path = config.get_repository_path();
        if repository_path.is_none(){
            let current_dir = std::env::current_dir()?;
            let repository_config_file = current_dir.join("vat.repository.toml");
            if !current_dir.is_empty(){
                if repository_config_file.exists(){
                    return Err(anyhow::anyhow!("Repository already exists, {}", repository_path.unwrap().display()));
                }else{
                    return Err(anyhow::anyhow!("Current directory is not empty to initialize the repository"));
                }
            }else{
                let repository_config = VatRepository::default();
                let repository_config_str = toml::to_string(&repository_config)?;
                fs::write(repository_config_file, repository_config_str)?;
                config.set_repository_path(current_dir.clone());
                config.save()?;
                cprintln!("      <green>Repository initialized</green>");
            }
        }else{
            let repository_path = config.get_repository_path().unwrap();
            let repository_config_file = repository_path.join("vat.repository.toml");
            if repository_config_file.exists(){
                return Err(anyhow::anyhow!("Repository config file already exists, {}", repository_config_file.display()));
            }else{
                let repository_config = VatRepository::default();
                let repository_config_str = toml::to_string(&repository_config)?;
                fs::write(repository_config_file, repository_config_str)?;
                config.set_repository_path(repository_path.clone());
                config.save()?;
                cprintln!("      <green>Repository initialized</green>");
            }
        }
    
        Ok(VatRepository::default())
    }

  

    pub fn read_repository() -> Result<Self, anyhow::Error> {
        let vat_config = VatConfig::init()?;
        let repository_path = vat_config.get_repository_path();
        let repository_path = if let Some(repository_path) = repository_path {
            repository_path
        } else {
            return Err(anyhow::anyhow!("Repository path not found, Run vat repo-init to initialize the repository"));
        };
        let repository_config_file = repository_path.join("vat.repository.toml");
        let repository_config_str = fs::read_to_string(repository_config_file)?;
        let repository_config: VatRepository = toml::from_str(&repository_config_str)?;
        Ok(repository_config)
    }

    pub fn add_package(&mut self, package: Package) -> Result<Package, anyhow::Error> {
        if self.packages.contains_key(&package.get_name()){
            let package_versions = self.packages.get_mut(&package.get_name()).unwrap();
            if package_versions.publishes.contains_key(&package.get_version()){
                return Err(anyhow::anyhow!("Package version has already been published, {}", package.get_version()));
            }
            package_versions.append_version(package.clone());
            self.save()?;
            Ok(package)
        }else{
            let package_name = package.get_name();
            let package_versions = PackageVersions::from(package.clone());
            self.packages.insert(package_name, package_versions);
            self.save()?;
            Ok(package)
        }

    }

    pub fn get_package(&self, package_name: &str) -> Option<&PackageVersions> {
        if self.packages.contains_key(package_name) {
            self.packages.get(package_name)
        }else{
            None
        }
    }

    pub fn remove_package(&mut self, package_name: &str) -> Result<(), anyhow::Error> {
        if self.packages.contains_key(package_name) {
            self.packages.remove(package_name);
            self.save()?;
            Ok(())
        }else{
            Err(anyhow::anyhow!("Package not found: {}", package_name))
        }
    }

    pub fn get_latest_package_version(&self, package_name: &str) -> Option<&Package> {
        let package_versions = self.packages.get(package_name)?;
        let latest_version = package_versions.get_latest_version();
        latest_version
    }

    

 



    pub fn save(&self) -> Result<(), anyhow::Error> {
        let vat_config = VatConfig::init()?;
        let repository_path = vat_config.get_repository_path();
        let repository_path = if let Some(repository_path) = repository_path {
            repository_path
        } else {
            return Err(anyhow::anyhow!("Repository path not found, Run vat repo-init to initialize the repository"));
        };
        let toml_string = toml::to_string(self)?;
        let mut toml_file = std::fs::File::create(repository_path.join("vat.repository.toml"))?;
        toml_file.write_all(toml_string.as_bytes())?;
        Ok(())
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
