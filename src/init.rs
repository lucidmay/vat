use std::path::PathBuf;
use anyhow::Result;
use crate::package::Package;
use std::io::Write;
use color_print::cprintln;
use git2::Repository;

pub struct Vat {
    pub package_path: PathBuf,
    pub package_name: String,
}

impl Vat {
    pub fn init() -> Result<Self> {

        cprintln!("      <green>Creating</green> vat default package");

        // get the current working directory
        let current_dir = std::env::current_dir()?;
        let current_dir_name = current_dir.file_name().unwrap().to_str().unwrap();

        match current_dir.is_empty() {
            false => return Err(anyhow::anyhow!("The current directory is not empty")),
            true => (),
        }

        let vat_yaml_path = current_dir.join("Vat.toml");
        if vat_yaml_path.exists() {
            return Err(anyhow::anyhow!("Vat.toml already exists, looks like you already initialized the package"));
        }

        // create yaml file
        let toml_string = toml::to_string(&Package::new(current_dir_name.to_string()))?;

        let mut toml_file = std::fs::File::create(vat_yaml_path)?;
        toml_file.write_all(toml_string.as_bytes())?;

           // Initialize a new Git repository
        // let output = Command::new("git")
        //     .arg("init")
        //     .current_dir(&current_dir) // Set the current directory for the command
        //     .output() // Use output() to capture the command's output
        //     .expect("Failed to initialize Git repository");

        // if output.status.success() {
        //     cprintln!("      <green>Initialized Git repository</green>");
        // } else {
        //     let error_message = String::from_utf8_lossy(&output.stderr);
        //     eprintln!("Failed to initialize Git repository: {}", error_message);
        // }

        let repo = match Repository::init(&current_dir) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to init: {}", e),
        };


        Ok(Self { package_path: current_dir.clone(), package_name: current_dir_name.to_string() })

    }

    pub fn read(package_path: PathBuf) -> Result<Package> {
        let toml_string = std::fs::read_to_string(package_path.join("Vat.toml"))?;
        let package: Package = toml::from_str(&toml_string)?;
        Ok(package)
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

