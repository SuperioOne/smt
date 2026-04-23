use std::{io::Read as _, path::Path};

pub fn read_input<T>(path: Option<T>) -> Result<String, std::io::Error>
where
  T: AsRef<Path>,
{
  match path {
    Some(path) => std::fs::read_to_string(path),
    None => {
      let mut buffer = String::new();
      let mut stdin = std::io::stdin();

      match stdin.read_to_string(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(err) => Err(err),
      }
    }
  }
}
