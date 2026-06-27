#[macro_use] extern crate pamsm;

use pamsm::{ 
    PamServiceModule, Pam, PamError,
    PamFlags, PamLibExt
};
use std::process::Command;

struct PamFaceScape;

impl PamServiceModule for PamFaceScape {
    fn authenticate(pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        let user = match pamh.get_user(None) {
            Ok(Some(u)) => u.to_string_lossy().into_owned(),
            _ => return PamError::USER_UNKNOWN,
        };

        println!("[PAM FaceScape] Initializing biometric check for user: {}", user);

        let status = Command::new("/usr/local/bin/face-scape")
            .arg("auth")
            .arg("--user")
            .arg(&user)
            .status();

        match status {
            Ok(stat) if stat.success() => {
                println!("[PAM FaceScape] Biometric verification successful.");
                PamError::SUCCESS
            }
            _ => {
                eprintln!("[PAM FaceScape] Biometric validation rejected or timed out");
                PamError::AUTH_ERR
            }
        }
    }
}

pam_module!(PamFaceScape);
