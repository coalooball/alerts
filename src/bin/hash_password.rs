use anyhow::Result;
use alerts::AuthService;
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("hash-password")
        .version("1.0")
        .about("Hash passwords for user accounts")
        .arg(
            Arg::new("password")
                .help("Password to hash")
                .required(true)
                .index(1)
        )
        .get_matches();

    let password = matches.get_one::<String>("password").unwrap();
    
    match AuthService::hash_password(password) {
        Ok(hash) => {
            println!("Password hash: {}", hash);
            println!("Copy this hash to your database insert statement.");
        }
        Err(e) => {
            eprintln!("Error hashing password: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}