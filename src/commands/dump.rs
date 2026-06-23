use crate::crypt;

pub fn run(path: &str) -> crate::Result<()> {
    let path = std::path::Path::new(path);
    assert!(path.is_file());
    let password = crypt::prompt_secret("Please enter password:");
    println!("{}", crypt::decrypt_from_file(path, &password)?);
    Ok(())
}
