use std::path::Path;

pub fn compile_code(language: &str, file: &Path) {
    // Only for compiled languages
    match language {
        "Rust" => {
            // Compilation logic for Rust
        }
        "C++" => {
            // Compilation logic for C++
            let tmp_path = gen_temp_path();

            // Use clang++ to compile and run (in-memory compilation, no disk access)
            let status = std::process::Command::new("clang++")
                .arg("-x")
                .arg("c++")
                .arg(file)
                .arg("-o")
                .arg(&tmp_path)
                .arg("-pipe")
                .arg("-std=c++20")
                .status()
                .expect("Failed to compile C++ code");
        }
        "C" => {
            // Compilation logic for C
        }
        _ => {
            println!("{} is not a compiled language.", language);
        }
    }
}

pub fn gen_temp_path() -> std::path::PathBuf {
    // If system is macOS, mktemp -t tmp_cpp_exec
    // If system is Linux, /dev/shm/tmp_cpp_exec
    if cfg!(target_os = "macos") {
        // use PID + time to generate a unique filename
        let mut tmp = std::env::temp_dir();
        let name = format!(
            "tmp_exec_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        tmp.push(name);
        // try to create the file
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&tmp);
        tmp
    } else {
        std::path::PathBuf::from("/dev/shm/tmp_exec")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_temp_path() {
        let temp_path = gen_temp_path();
        assert!(temp_path.exists());
    }
}
