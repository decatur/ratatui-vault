use std::path::Path;

pub fn run(args: Vec<String>) {
    if args.len() == 2 {
        diff_lines(Path::new(&args[0]), Path::new(&args[1]));
    } else {
        println!("Command diff needs two path arguments");
    }
}

fn diff_lines(a: &Path, b: &Path) {
    let password = crate::prompt_secret("Please enter password:");
    let left = crate::decrypt_from_file(a, &password).unwrap();
    let right = crate::decrypt_from_file(b, &password).unwrap();

    let mut last_match = None;
    for diff in diff::lines(&left, &right) {
        match diff {
            diff::Result::Left(l) => {
                if let Some(cr) = last_match {
                    println!(" {cr}");
                    last_match = None;
                }
                println!("- {l}");
            }
            diff::Result::Both(l, _) => {
                last_match = Some(l);
                // println!(" {}", l)
            }
            diff::Result::Right(r) => {
                if let Some(cr) = last_match {
                    println!(" {cr}");
                    last_match = None;
                }
                println!("+ {r}");
            }
        }
    }
}
