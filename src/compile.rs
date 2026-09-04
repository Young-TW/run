use std::path::{Path, PathBuf};
use std::process::Command;

/// Compile a source file for a compiled language into a temporary executable.
///
/// Returns the path to the produced executable on success, or `None` if the
/// language is not a compiled one or compilation failed. The temporary output
/// lives in `/dev/shm` on Linux (no persistent disk access) and the system
/// temp dir on macOS.
pub fn compile_code(language: &str, file: &Path) -> Option<PathBuf> {
    let output = gen_temp_path();

    let mut command = Command::new(compiler(language)?);
    match language {
        "C++" => {
            command
                .arg("-x")
                .arg("c++")
                .arg(file)
                .arg("-o")
                .arg(&output)
                .arg("-pipe")
                .arg("-std=c++20");
        }
        "C" => {
            command
                .arg("-x")
                .arg("c")
                .arg(file)
                .arg("-o")
                .arg(&output)
                .arg("-pipe")
                .arg("-std=c17");
        }
        "Rust" => {
            command.arg(file).arg("-o").arg(&output);
        }
        _ => return None,
    }

    let status = command.status().ok()?;
    if status.success() {
        Some(output)
    } else {
        // Remove the empty placeholder file left behind on a failed build.
        let _ = std::fs::remove_file(&output);
        None
    }
}

/// The compiler executable used for a given compiled language.
///
/// Prefers the portable POSIX names (`cc`, `c++`) so the tool works whether
/// the host provides GCC or Clang. Returns `None` for non-compiled languages.
pub fn compiler(language: &str) -> Option<&'static str> {
    match language {
        "C++" => Some("c++"),
        "C" => Some("cc"),
        "Rust" => Some("rustc"),
        _ => None,
    }
}

pub fn gen_temp_path() -> std::path::PathBuf {
    // Choose the base directory: on Linux the executable lives in memory-backed
    // /dev/shm (no persistent disk access); everywhere else (macOS, Windows,
    // other Unixes) fall back to the system temp dir.
    let mut tmp = if cfg!(target_os = "linux") {
        std::path::PathBuf::from("/dev/shm")
    } else {
        std::env::temp_dir()
    };

    // Use PID + time to generate a unique filename so concurrent runs don't clash.
    let mut name = format!(
        "tmp_exec_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    // Windows executables must carry the .exe extension to be launchable.
    if cfg!(windows) {
        name.push_str(".exe");
    }
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

    #[test]
    fn test_compile_code_rejects_non_compiled_language() {
        // Shell / Python / Unknown are not compiled languages.
        assert!(compile_code("Shell", Path::new("foo.sh")).is_none());
        assert!(compile_code("Python", Path::new("foo.py")).is_none());
    }
}
