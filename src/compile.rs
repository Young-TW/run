use std::path::Path;

pub fn compile_code(language: &str, file: &Path) {
    // Only for compiled languages
    match language {
        "Rust" => {
            // Compilation logic for Rust
        }
        "C++" => {
            // Compilation logic for C++
            // If system is macOS, mktemp -t tmp_cpp_exec
            // If system is Linux, /dev/shm/tmp_cpp_exec
            let tmp_path = if cfg!(target_os = "macos") {
                std::env::temp_dir().join("tmp_cpp_exec")
            } else {
                std::path::PathBuf::from("/dev/shm/tmp_cpp_exec")
            };

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
