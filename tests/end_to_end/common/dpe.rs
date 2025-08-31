use std::env::current_dir;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Dpe {
    compose_file_path: PathBuf,
}

impl Dpe {
    pub fn new(dir_path: &Path) -> Self {
        let compose_file_path = dir_path.join("compose.dpe.yml");

        Self { compose_file_path }
    }

    pub fn start(&self, port: u16, compose_contents: &str) {
        let compose_contents = compose_contents
            .replace("container_name: docker-socket-proxy", "")
            .replace("container_name: socket-proxy", "")
            .replace(
                "image: calum4/docker-prometheus-exporter:latest",
                format!("build: {}", current_dir().unwrap().to_str().unwrap()).as_str()
            )
            .replace("container_name: docker-prometheus-exporter", "")
            .replace(r##"127.0.0.1:9000:9000"##, format!("127.0.0.1:{}:9000", port).as_str());

        let mut compose_file = File::create(&self.compose_file_path).unwrap();
        compose_file.write_all(compose_contents.as_bytes()).unwrap();

        compose_file.flush().unwrap();
        compose_file.sync_all().unwrap();

        Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("up")
            .arg("-d")
            .arg("--build")
            .output()
            .unwrap();
    }
}

impl Drop for Dpe {
    fn drop(&mut self) {
        let down = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("down")
            .arg("--rmi")
            .arg("local") // delete image
            .output();

        match down {
            Ok(mut out) if !out.status.success() => {
                out.stdout.push(b'\n');
                out.stdout.extend(out.stderr);

                match String::from_utf8(out.stdout) {
                    Ok(s) => {
                        eprintln!("failed to teardown docker-prometheus-exporter containers: {s}")
                    }
                    Err(_) => eprintln!("failed to teardown docker-prometheus-exporter containers"),
                }
            }
            Err(error) => {
                eprintln!("failed to teardown docker-prometheus-exporter containers: {error}");
            }
            _ => {}
        }

        if let Err(error) = remove_file(&self.compose_file_path) {
            eprintln!("unable to remove docker-prometheus-exporter compose file: {error}");
        }
    }
}
