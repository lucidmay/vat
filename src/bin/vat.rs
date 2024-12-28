use clap::{Parser, Subcommand};
use colored::*;
use vat::init::Vat;
use std::process::Command;
use vat::config::VatConfig;
use vat::repository::VatRepository;
use git2::Repository as GitRepository;

/// Simple program to demonstrate colored CLI
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to greet
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Read,
    Houdini,
    Publish,
    Repo,
    RepoInit,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            if let Err(e) = Vat::init() {
                eprintln!("{}", format!("Error initializing Vat: {}", e).red()); 
            }
        }
        Some(Commands::Read) => {
            let current_dir = std::env::current_dir().unwrap();
            let vat = Vat::read(current_dir);
            match vat {
                Ok(package) => {
                    println!("Package: {:?}", package);
                }
                Err(e) => {
                    eprintln!("{}", format!("Error reading Vat: {}", e).red());
                }
            }
        },
        Some(Commands::Houdini) => {
            // append path env variable
            let houdini_path = "C:\\Program Files\\Side Effects Software\\Houdini 20.5.314\\bin";
            let current_path = std::env::var("PATH").unwrap_or_default();
            let new_path = format!("{};{}", current_path, houdini_path);
            std::env::set_var("PATH", new_path);

             let output = Command::new("houdini")
                .status()
                .expect("Failed to execute Houdini command");

            if output.success() {
                println!("Houdini launched successfully.");
            } else {
                eprintln!("Failed to launch Houdini.");
            }

            },
        Some(Commands::Publish) => {
             let current_dir = std::env::current_dir().unwrap();

        // Execute the git command to get the latest tag
        // let output = Command::new("git")
        //     .arg("describe")
        //     .arg("--tags")
        //     .arg("--abbrev=0") // Get the most recent tag
        //     .current_dir(&current_dir) // Set the current directory for the command
        //     .output() // Use output() to capture the command's output
        //     .expect("Failed to execute git command");

        //     if output.status.success() {
        //         let stdout = String::from_utf8_lossy(&output.stdout); // Create a longer-lived value
        //         let latest_tag = stdout.trim(); // Trim whitespace
        //         println!("Latest Git tag version: {}", latest_tag);
        //     } else {
        //         let error_message = String::from_utf8_lossy(&output.stderr);
        //         eprintln!("Failed to get latest Git tag: {}", error_message);
        //     }

        let repo = GitRepository::open(&current_dir).unwrap();
        // let tag = repo.describe().unwrap();
        let tags = repo.tag_names(None).unwrap();
        let tags = tags.iter().collect::<Vec<_>>();
        dbg!(tags);

            
        },
        Some(Commands::Repo) => {
            let config = VatConfig::init().unwrap();
            println!("Repository path: {:?}", config.get_repository_path());
        },
        Some(Commands::RepoInit) => {
            let repository = VatRepository::init().unwrap();
        },
        None => {
            println!("Vat");
            println!("      Have you watched the Vat of Acid episode? https://en.wikipedia.org/wiki/The_Vat_of_Acid_Episode");
        },
    }
}