use clap::Parser;
use vat::repository::VatRepository;
use colored::*;

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
}

fn main(){

    let cli = Cli::parse();


    let mut vat_repository = VatRepository::read_repository().unwrap();


    let package_name = cli.package_name;


    if let Some(package_name) = package_name {
        let package = vat_repository.get_latest_package_version(&package_name).unwrap();
        if let Some(command) = cli.command {
            let env_load = package.command_load_env(&command);
                if !cli.append.is_empty() {
                    for name in cli.append{
                        if vat_repository.get_package(&name).is_some() {
                            let package = vat_repository.get_latest_package_version(&name).unwrap();
                            let result = package.load_all_environments();
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

    if cli.list {
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


}
