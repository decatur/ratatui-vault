use crate::crypt::{self, SecretString};
use std::path::Path;

/// Apply source onto target, i.e. a line in source not in target is marked with a '+'.
pub fn merge(target: &Path, source: &Path, password: &SecretString) -> String {
    let left = crypt::decrypt_from_file(target, password).unwrap();
    let right = crypt::decrypt_from_file(source, password).unwrap();
    let mut buf = vec![];
    for diff in diff::lines(&left, &right) {
        buf.push(match diff {
            diff::Result::Left(l) => format!("-{l}"),
            diff::Result::Both(l, _) => l.to_owned(),
            diff::Result::Right(r) => format!("+{r}"),
        });
    }
    buf.join("\n")
}
