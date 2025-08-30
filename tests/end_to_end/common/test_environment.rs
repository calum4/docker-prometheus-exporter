use rand::Rng;
use std::env::temp_dir;
use std::fs::{create_dir, remove_dir_all};
use std::path::PathBuf;

fn get_random_alphanumeric_id() -> String {
    const ID_POSTPEND: &str = "_dpe_test";
    const RANDOM_CHARS_LEN: usize = 10;

    let mut id = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(RANDOM_CHARS_LEN)
        .fold(
            String::with_capacity(RANDOM_CHARS_LEN + ID_POSTPEND.len()),
            |mut s, c| {
                s.push(c as char);
                s
            },
        );

    id.push_str(ID_POSTPEND);
    id
}

pub struct TestEnvironment {
    pub id: String,
    pub temp_dir: PathBuf,
}

impl Default for TestEnvironment {
    fn default() -> Self {
        let mut id = get_random_alphanumeric_id();
        id.make_ascii_lowercase(); // Docker Compose project names are lowercase

        let mut temp_dir = temp_dir();
        temp_dir.push(format!("{id}/"));

        Self { id, temp_dir }
    }
}

impl TestEnvironment {
    pub fn setup(&self) {
        create_dir(self.temp_dir.as_path()).unwrap();
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if let Err(error) = remove_dir_all(self.temp_dir.as_path()) {
            eprintln!(
                "unable to remove the test's temporary directory at '{:#?}', error: {error}",
                self.temp_dir.as_path()
            );
        }
    }
}
