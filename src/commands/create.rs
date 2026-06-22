use crate::crypt;

pub fn run(path: Option<String>) -> crate::Result<()> {
    let Some(path) = path else {
        println!("Command create needs a path parameter");
        return Ok(());
    };

    let path = std::path::Path::new(&path);
    assert!(!path.exists(), "Path must not exist");

    let password = crypt::prompt_secret("Enter password for new vault:");
    let password2 = crypt::prompt_secret("Please repeat password:");
    assert_eq!(password, password2);
    crypt::encrypt_to_file("[Hello World]".to_owned(), path, &password)?;
    Ok(())
}
