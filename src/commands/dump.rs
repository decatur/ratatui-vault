use crate::crypt;

pub fn run(path: &std::path::Path) -> crate::Result<()> {
    let password = crypt::prompt_secret("Please enter password:");
    println!("{}", crypt::decrypt_from_file(path, &password)?);
    Ok(())
}
