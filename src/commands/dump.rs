pub fn run(path: Option<String>) -> crate::Result<()> {
    if let Some(path) = path {
        let path = std::path::Path::new(&path);
        assert!(path.is_file());
        let password = crate::prompt_secret("Please enter password:");
        println!("{}", crate::decrypt_from_file(path, &password)?);
    } else {
        println!("command dump usage:");
        println!("  dump myvault | grep MyBank -A 5");
    }
    Ok(())
}
