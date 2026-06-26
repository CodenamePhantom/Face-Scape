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
    /// Enroll a new face model into FaceScape.
    Enroll {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },

    /// Authenticate against a FaceScape enrolled user.
    Auth {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },

    /// Manage FaceScape.
    Manage {
        #[command(subcommand)]
        action: Manage
    }
}

#[derive(Subcommand)]
enum Manage {
    /// Start FaceScape.
    Start {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },

    /// Stop FaceScape.
    Stop {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },

    /// Reloads FaceScape
    Reload {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },

    /// List enrolled models.
    List,

    /// Delete a model.
    Delete {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },

    /// Updates a model.
    Update {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },

    /// Re-enrolls a user and restarts the model.
    Rebase {
        #[arg(long, default_value_t = whoami())]
        user: String,
    },
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
            Enroll::enroll(user);
        }
        Commands::Auth { user } => {
            Authenticator::run(user);
        }
        Commands::Manage { action } => {
            match action {
                Manage::Start { user } => {
                    AuthManager::start(user);
                }
                Manage::Stop { user } => {
                    AuthManager::stop(user);
                }
                Manage::Reload { user } => {
                    AuthManager::reload(user);
                }
                Manage::List => {
                    AuthManager::list();
                }
                Manage::Delete { user } => {
                    AuthManager::delete(user);
                }
                Manage::Update { user } => {
                    AuthManager::update(user);
                }
                Manage::Rebase { user } => {
                    AuthManager::delete(user.clone());
                    AuthManager::stop(user.clone());
                    AuthManager::start(user);
                }
            }
        }
    }
}
