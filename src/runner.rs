use std::path::Path;
use std::process::{Command, ExitStatus};

/// Compile-and-run (for compiled languages) or interpret (for scripting
/// languages) a source file, returning the executed program's exit code.
///
/// Returns `1` when the language is unsupported or the program could not be
/// launched.
pub fn run(language: &str, file: &Path) -> i32 {
    match language {
        "C" | "C++" | "Rust" => match crate::compile::compile_code(language, file) {
            Some(executable) => {
                let code = run_executable(&executable);
                // The executable lives in a temp dir; remove it once it has run.
                let _ = std::fs::remove_file(&executable);
                code
            }
            None => {
                eprintln!("Failed to compile {language} code.");
                1
            }
        },
        "Shell" => run_with("sh", file),
        _ => {
            eprintln!("{language} is not supported yet.");
            1
        }
    }
}

/// Run a previously compiled executable.
fn run_executable(path: &Path) -> i32 {
    Command::new(path).status().map(exit_code).unwrap_or(1)
}

/// Run a source file through an interpreter (e.g. `sh script.sh`).
fn run_with(interpreter: &str, file: &Path) -> i32 {
    Command::new(interpreter)
        .arg(file)
        .status()
        .map(exit_code)
        .unwrap_or(1)
}

/// Extract a numeric exit code, defaulting to `1` when the process was
/// terminated by a signal (no code available).
fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("run_test_{}_{}", std::process::id(), name));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        path
    }

    /// Skip a toolchain-dependent test gracefully if the compiler is absent.
    fn available(cmd: &str) -> bool {
        Command::new(cmd).arg("--version").output().is_ok()
    }

    #[test]
    fn test_run_shell_success() {
        let file = write_temp("ok.sh", "exit 0\n");
        assert_eq!(run("Shell", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_shell_propagates_exit_code() {
        let file = write_temp("fail.sh", "exit 3\n");
        assert_eq!(run("Shell", &file), 3);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_c() {
        if !available("cc") {
            return;
        }
        let file = write_temp("prog.c", "int main(void) { return 0; }\n");
        assert_eq!(run("C", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_c_propagates_exit_code() {
        if !available("cc") {
            return;
        }
        let file = write_temp("ret.c", "int main(void) { return 7; }\n");
        assert_eq!(run("C", &file), 7);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_cpp() {
        if !available("c++") {
            return;
        }
        let file = write_temp("prog.cpp", "int main() { return 0; }\n");
        assert_eq!(run("C++", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_rust() {
        if !available("rustc") {
            return;
        }
        let file = write_temp("prog.rs", "fn main() { std::process::exit(0); }\n");
        assert_eq!(run("Rust", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_unsupported_language_returns_error() {
        let file = write_temp("x.py", "print('hi')\n");
        assert_eq!(run("Python", &file), 1);
        let _ = std::fs::remove_file(&file);
    }
}
