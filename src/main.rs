mod core;
mod globals;
mod model;
mod pipelines;

use crate::pipelines::enroll::Enroll;
use crate::pipelines::authenticate::Authenticator;
use crate::core::auth_manager::AuthManager;
use clap::{ Parser, Subcommand };

#[derive(Parser)]
#[command(name = "facescape", about = "Hardware-agnostic facial auth for Linux")]
struct Cli {
    #[arg(long, default_value = "/dev/video0", global = true)]
    device: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Enroll {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },
    Auth {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },
    Manage {
        #[command(subcommand)]
        action: Manage
    }
}

#[derive(Subcommand)]
enum Manage {
    Start {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },
    Stop {
        #[arg(long, default_value_t = whoami())]
        user: String,
    }
}

fn whoami() -> String {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "unknown".into());

    return user
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Enroll { user } => {
            Enroll::enroll();
        }
        Commands::Auth { user } => {
            Authenticator::run(user);
        }
        Commands::Manage { action } => {
            match action {
                Manage::Start { user } => {
                    AuthManager::start();
                }
                Manage::Stop { user } => {
                    AuthManager::stop();
                }
            }
        }
    }
}
