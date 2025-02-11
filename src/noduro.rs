use installed_pkg::list as app_list;
use crate::package::{Package, Environtment, Command, EnvAction, PackageInfo};
use crate::registry::Registry;
use anyhow::Result;
use std::env;
use std::path::PathBuf;


#[derive(Debug, Clone)]
pub struct Application{
    pub name: String,
    pub version: Option<String>,
    pub path: PathBuf,
}

impl Application{
    pub fn new(name: String, version: Option<String>, path: PathBuf) -> Self{
        Self{name, version, path}
    }
}

// Function to find alll installed applications and save to dcc package
pub fn find_installed_apps() -> Vec<Application>{
    let installed = app_list();
    let mut apps = Vec::new();
    if let Ok(installed) = installed{
        for app in installed.apps.iter(){
            apps.push(Application{
                name: app.name.clone(),
                version: None,
                path: PathBuf::from(app.root.clone()),
            });
        }
    }
    apps
}


pub fn filter_apps(apps: Vec<Application>) -> Vec<Application>{
    let mut filtered_apps = Vec::new();

    for app in apps.iter(){
        // Zbrush
        if app.name.starts_with("ZBrush"){
            let version = app.name.strip_prefix("ZBrush ").unwrap();
            let app = Application::new(app.name.clone(), Some(version.to_string()), app.path.clone());
            filtered_apps.push(app);
        }
        // Autodesk Maya
        else if app.name.starts_with("Autodesk Maya"){
            let version = app.name.strip_prefix("Autodesk Maya ").unwrap();
            let folder_name = format!("Maya{}", version);
            let path = app.path.join(folder_name).join("bin");
            if path.exists(){
                let app = Application::new(app.name.clone(), Some(version.to_string()), path);
                filtered_apps.push(app);
            }
        }
        // Shotgun RV
        else if app.name.starts_with("Shotgun RV"){
            let path = app.path.join("bin");
            if path.exists(){
                let version = app.name.strip_prefix("Shotgun RV ").unwrap();
                let app = Application::new(app.name.clone(), Some(version.to_string()), path);
                filtered_apps.push(app);
            }
        }
        // OpenRV
        else if app.name.starts_with("OpenRV"){    
            filtered_apps.push(app.clone());
        }
        // PureRef
        else if app.name.starts_with("PureRef"){
            filtered_apps.push(app.clone());
        }
        // Adobe Photoshop
        else if app.name.starts_with("Adobe Photoshop"){
            filtered_apps.push(app.clone());
        }
        // Nuke
        else if app.name.starts_with("Nuke"){
            filtered_apps.push(app.clone());
        }
        // Adobe Illustrator
        else if app.name.starts_with("Adobe Illustrator"){
            filtered_apps.push(app.clone());
        }
        // Houdini
        else if app.name.starts_with("Houdini"){
        }
    }

    filtered_apps
}



pub fn manual_checking() -> Vec<Application>{

    let mut manual_apps : Vec<Application> = Vec::new();

    #[cfg(target_os = "macos")]
    {

    }

    #[cfg(target_os = "windows")]
    {
        let program_files = env::var("ProgramFiles").unwrap();

        // check houdini
        {
            let path = PathBuf::from(program_files).join("Side Effects Software");
            if path.exists(){
                // list dir
                let dir = std::fs::read_dir(path).unwrap();
                for entry in dir{
                    let entry = entry.unwrap();
                    if entry.path().is_dir(){
                        if entry.file_name().to_str().unwrap().starts_with("Houdini"){
                            let version = entry.file_name().to_str().unwrap().strip_prefix("Houdini ").unwrap().to_string();
                            let first_number = version.chars().next().unwrap();
                            let int_version = first_number.to_digit(10);
                            if let Some(_int_version) = int_version{
                                let bin_path = entry.path().join("bin");
                                if bin_path.exists(){
                                    let app = Application::new("Houdini".to_string(), Some(version), entry.path());
                                    manual_apps.push(app);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    manual_apps
}



pub fn dcc_package_from_apps(apps: Vec<Application>) -> Result<Package, anyhow::Error>{
    let registry = Registry::init();
    if registry.is_err(){
        return Err(anyhow::anyhow!("Failed to read registry"));
    }
    let dcc_package = registry.unwrap().read_package("dcc");
    // if dcc_package.is_err(){
    //     return Err(anyhow::anyhow!("Failed to read dcc package"));
    // }

    // let mut dcc_package = dcc_package.unwrap();

    let mut dcc_package = match dcc_package{
        Ok(package) => package,
        Err(_) => {

            let package_info = PackageInfo{
                name: "dcc".to_string(),
                version: semver::Version::new(0, 0, 1),
                version_message: Some("Initialized on Noduro launch".to_string()),
                description: Some("Installed DCC applications".to_string()),
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
                keywords: None,
            };

            let dcc_package = Package::from_package_info(package_info);
            dcc_package
        }

    };





    for app in apps.iter(){
        if app.name.starts_with("Houdini"){
            let version = app.version.as_ref().unwrap();
            let first_two_char = version.chars().take(2).collect::<String>();
            let fourth_char = version.chars().nth(3).unwrap();
            let path = app.path.clone();

            let bin_path = path.join("bin");
            let env_name = format!("houdinibin{}", version);
            let env = Environtment::from("PATH".to_string(), bin_path.to_str().unwrap().to_string(), Some(EnvAction::Append));
            dcc_package.append_env(&env_name, env);

            let command = Command::from(format!("houdini"), Some(vec![env_name]));
            let command_name = format!("{}{}{}", "houdini",  first_two_char, fourth_char);

            dcc_package.append_command(&command_name, command); 
        }

    }


    Ok(dcc_package)
}

