use clap::Parser;
use vat::repository::VatRepository2;
use colored::*;
use git2::Repository as GitRepository;
use vat::git::GitTags;
use std::path::PathBuf;
use std::process::Command;
use vat::package::Package;
use vat::git::Git;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    package_name: Option<String>,

    command: Option<String>,

    #[arg(short, long, num_args = 1..)]
    append: Vec<String>, // This will capture multiple package names to append

    #[arg(short, long)]
    list: bool, // Boolean flag for listing

    #[arg(short, long, num_args = 1..)]
    remove: Vec<String>,

    #[arg(long)]
    list_cmds: bool,

    #[arg(short, long)]
    status: bool,

    #[arg(short, long)]
    clone: Option<String>,

    #[arg(short, long)]
    update: Option<String>,
}

fn main(){

    let cli = Cli::parse();


    let mut vat_repository = VatRepository2::read_repository().unwrap();



    let package_name = cli.package_name;

    if let Some(package_name) = package_name {
        let package = vat_repository.get_latest_package(&package_name);
        if let Some(package) = package.clone() {
            if let Some(command) = cli.command {
                let latest_package_path = vat_repository.get_latest_package_path(&package_name).unwrap();
                // load the env for given command
                let env_load = package.command_load_env(&command, &latest_package_path);
                // if apped is not empty, load the env for the given packages
                if !cli.append.is_empty() {
                    for name in cli.append{
                        if vat_repository.get_latest_package(&name).is_some() {
                            let package = vat_repository.get_latest_package(&name).unwrap();
                            let result = package.load_all_environments(&latest_package_path);
                            match result {
                                Ok(_) => {
                                    println!("{}", format!("Package {} loaded successfully", name).green());
                                }
                                Err(e) => {
                                    eprintln!("{}", format!("Error loading package: {}", e).red());
                                }
                            }
                        }else{
                            println!("{}", format!("Package not found").red());
                        }
                    }
                }

                // run the command
                match env_load {
                    Ok(_) => {
                        let command = package.run_only_command(&command);
                        match command {
                            Ok(_) => {
                                println!("Command run successfully");
                            }
                            Err(e) => {
                                eprintln!("{}", format!("Error running command: {}", e).red());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", format!("Error loading environment: {}", e).red());
                    }
                }
            }

        }

        if let Some(package) = &package {
            if cli.list_cmds {
                package.list_commands();
            }
        }

        // STATUS
        if let Some(package) = &package {
            if cli.status {
                let package_path = vat_repository.get_package_main_path(&package_name).unwrap();
                println!("{}", format!("Package <main> path: {}", package_path.display()));

                let repo = GitRepository::open(&package_path).unwrap();
                let tags = &repo.tag_names(None).unwrap();
                let tags = tags.iter().collect::<Vec<_>>();
                let tags = tags.iter().map(|tag| tag.unwrap().to_string()).collect::<Vec<_>>();
                let git_tags = GitTags::new(tags);
                if !git_tags.tags.is_empty() {

                    let repo_latest_tag = git_tags.get_latest().unwrap();
                    let package_latest_version = package.get_current_version();

                    if repo_latest_tag < package_latest_version {
                        println!("{}", format!("Package {} is ahead of the latest published version", package_name).red());
                    } else if repo_latest_tag > package_latest_version {
                        println!("{}", format!("Package {} is behind the latest published version", package_name).yellow());
                        println!("{}", format!("The latest published tag in the main repository is {}\n", repo_latest_tag).yellow());
                        println!("Run `vat publish` in {} to publish the latest tagged version in to the vat repository", package_path.display());
                    } else {
                        println!("{}", format!("Package {} is up to date with the latest published version", package_name).green());
                    }
                } else {
                    println!("{}", format!("No tags found").red());
                }
            }
        }

    }

    // UPDATE
    if let Some(package_name) = cli.update {
        println!("{}", format!("Updating package {}", package_name).green());
        if vat_repository.package_exists(&package_name) {
            let package_path = vat_repository.get_package_path(&package_name).unwrap();
            let package_parent_path = package_path.parent().unwrap();
            let repo_path = vat_repository.path.clone().join("packages").join(&package_name);

            if package_parent_path == repo_path {

                dbg!(&package_path);
                let cmd_result = Command::new("git").args(&["fetch", "--all"]).current_dir(&package_path).output().unwrap();
                if cmd_result.status.success() {
                    println!("{}", format!("Git fetch successful").green());
                }else{
                    eprintln!("{}", format!("Git fetch failed: {}", cmd_result.status).red());
                }


                let repo = GitRepository::open(&package_path).unwrap();
                let tags = repo.get_tags().unwrap();
                let git_tags = GitTags::new(tags);
                if !git_tags.tags.is_empty() {


                    // check out to latest tag
                    let latest_tag = git_tags.get_latest().unwrap();
                    let cmd_result = Command::new("git").args(&["checkout", latest_tag.to_string().as_str()]).current_dir(&package_path).output().unwrap();
                    if cmd_result.status.success() {
                        println!("{}", format!("Git checkout successful").green());
                    }else{
                        eprintln!("{}", format!("Git checkout failed: {}", cmd_result.status).red());
                        return;
                    }


                    let package = Package::read(&package_path).unwrap();

                    let result = vat_repository.add_package(package.clone(), package.get_version_message(), package_path.clone());
                    match result {
                        Ok(_) => {
                            let result = vat_repository.package_data_update(&package, package_path.clone());
                            match result {
                                Ok(_) => {
                                    let result = vat_repository.save();
                                    match result {
                                        Ok(_) => {
                                            println!("{}", format!("Package {} updated successfully", package_name).green());
                                        }
                                        Err(e) => {
                                            eprintln!("{}", format!("Error saving repository: {}", e).red());
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}", format!("Error updating package: {}", e).red());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", format!("Error updating package: {}", e).red());
                        }
                    }

                }else{
                    println!("{}", format!("No tags found").red());
                }


            }else{
                println!("The package is not cloned in the repository");
                println!("Run `vat publish` in {}", package_parent_path.display());
            }
        }else{
            println!("{}", format!("Package {} does not exist", package_name).red());
        }
    }


    


    if cli.list {
        println!("{}", format!("Listing all packages").green());
        vat_repository.pretty_list();
    }

    if !cli.remove.is_empty() {
        for name in cli.remove{
            let result = vat_repository.remove_package(&name);
            if let Err(e) = result {
                eprintln!("{}", format!("Error removing package: {}", e).red());
            }
        }
    }


    if let Some(clone) = cli.clone {
        // let result = vat_repository.clone_package(&cli.clone);
        let git_path = PathBuf::from(&clone);
        let name = git_path.file_name().unwrap().to_str().unwrap();
        // remove ext
        let package_name = name.split('.').next().unwrap(); 

        if !vat_repository.package_exists(package_name) {
            println!("{}", format!("Cloning package {}", clone).green());
            println!("{}", format!("Package name: {}", name).green());

            let package_path = vat_repository.path.join("packages").join(package_name).join("main");
            // let git_repo = GitRepository::clone(&git_path.to_str().unwrap(), &package_path);
            let clone = Command::new("git").args(&["clone", git_path.to_str().unwrap(), &package_path.to_str().unwrap()]).output().unwrap();
            if clone.status.success() {
                println!("{}", format!("Package cloned successfully").green());
            }else{
                eprintln!("{}", format!("Error cloning package: {}", clone.status).red());
            }

            let package_toml_file = &package_path.join("vat.toml");
            if !package_toml_file.exists() {
                let result = std::fs::remove_dir_all(&package_path);
                if let Err(e) = result {
                    eprintln!("{}", format!("Error removing package: {}", e).red());
                }

                println!("{}", format!("Not a valid vat package"));
                println!("{}", format!("Package {} does not contain a vat.toml file", package_name).red());
            }

            let package = Package::read(&package_path);
            match package {
                Ok(package) => {
                    let result =  vat_repository.add_package(package.clone(), "empty publish".to_string(), package_path.clone());
                    match result {
                        Ok(_) => {
                            let result = vat_repository.package_data_update(&package, package_path.clone());
                            match result {
                                Ok(_) => {
                                    let result = vat_repository.save();
                                    match result {
                                        Ok(_) => {
                                            println!("{}", format!("Package {} added successfully", package_name).green());
                                        }
                                        Err(e) => {
                                            eprintln!("{}", format!("Error saving repository: {}", e).red());
                                        }
                                    }
                                },
                                Err(e) => {
                                    eprintln!("{}", format!("Error updating package: {}", e).red());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", format!("Error adding package: {}", e).red());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("Error reading package: {}", e).red());
                }
            }
        }else{
            println!("{}", format!("Package {} already exists", package_name).red());
        }

        

        // let git_repo = GitRepository::clone(&cli.clone, &cli.clone).unwrap();
    }
}


