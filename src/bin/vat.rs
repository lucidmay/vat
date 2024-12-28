use clap::{Parser, Subcommand};
use colored::*;
use vat::init::Vat;
use std::process::Command;

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
            println!("Publish");
        },
        None => {
            println!("Vat");
            println!("      Have you watched the Vat of Acid episode? https://en.wikipedia.org/wiki/The_Vat_of_Acid_Episode");
        },
    }
}