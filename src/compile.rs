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
    // Choose the base directory: /dev/shm on Linux, the system temp dir on macOS.
    let mut tmp = if cfg!(target_os = "macos") {
        std::env::temp_dir()
    } else {
        std::path::PathBuf::from("/dev/shm")
    };

    // Use PID + time to generate a unique filename so concurrent runs don't clash.
    let name = format!(
        "tmp_exec_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    tmp.push(name);

    // Create the file so the path is guaranteed to exist on every platform.
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&tmp);
    tmp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_temp_path_creates_file() {
        let temp_path = gen_temp_path();
        assert!(temp_path.exists());
        // Clean up so repeated test runs don't leave files behind.
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_gen_temp_path_is_unique() {
        let first = gen_temp_path();
        let second = gen_temp_path();
        assert_ne!(first, second);
        let _ = std::fs::remove_file(&first);
        let _ = std::fs::remove_file(&second);
    }
}
