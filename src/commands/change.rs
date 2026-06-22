use crate::{crypt, error_string};

pub fn run(path: Option<String>) -> error_string::Result<()> {
    let Some(path) = path else {
        println!("Command change needs a path parameter");
        return Ok(());
    };

    let path = std::path::Path::new(&path);
    assert!(path.is_file(), "Path must be a vault");

    let old_password = crypt::prompt_secret("Enter old password:");
    let plaintext = crypt::decrypt_from_file(path, &old_password)?;

    let new_password = crypt::prompt_secret("Enter new password:");
    let password = crypt::prompt_secret("Please repeat password:");
    assert_eq!(password, new_password);
    crypt::encrypt_to_file(plaintext, path, &password)?;
    Ok(())
}
