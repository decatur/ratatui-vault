pub fn run(path: Option<String>) -> crate::Result<()> {
    let Some(path) = path else {
        println!("Command change needs a path parameter");
        return Ok(());
    };

    let path = std::path::Path::new(&path);
    assert!(path.is_file(), "Path must be a vault");

    let old_password = crate::prompt_secret("Enter old password:");
    let plaintext = crate::decrypt_from_file(path, &old_password)?;

    let new_password = crate::prompt_secret("Enter new password:");
    let password = crate::prompt_secret("Please repeat password:");
    assert_eq!(password, new_password);
    crate::encrypt_to_file(plaintext, path, &password)?;
    Ok(())
}
