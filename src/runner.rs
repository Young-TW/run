use std::path::Path;
use std::process::{Command, ExitStatus};

/// Compile-and-run (for compiled languages) or invoke the matching runtime (for
/// interpreted / managed languages) for a source file, returning the executed
/// program's exit code.
///
/// The runtime is chosen from the file `extension`, so e.g. `.sh` runs with
/// `sh` while `.bash` runs with `bash`. Returns `1` when the language is
/// unsupported or the program could not be launched.
pub fn run(language: &str, extension: &str, file: &Path) -> i32 {
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
        _ => match runtimes(extension) {
            Some(candidates) => run_with_runtime(language, extension, candidates, file),
            None => {
                eprintln!("{language} is not supported yet.");
                1
            }
        },
    }
}

/// Candidate runtime invocations for an interpreted / managed language, keyed by
/// file extension. Each candidate is the program name followed by any leading
/// arguments (the source file is appended afterwards). The first candidate whose
/// program is installed is used, allowing graceful fallbacks.
fn runtimes(extension: &str) -> Option<&'static [&'static [&'static str]]> {
    match extension {
        "sh" => Some(&[&["sh"]]),
        "bash" => Some(&[&["bash"]]),
        "py" => Some(&[&["python3"], &["python"]]),
        "js" => Some(&[&["node"]]),
        "ts" => Some(&[&["bun"], &["tsx"], &["ts-node"], &["deno", "run"]]),
        "rb" => Some(&[&["ruby"]]),
        "go" => Some(&[&["go", "run"]]),
        "java" => Some(&[&["java"]]),
        _ => None,
    }
}

/// Pick the first available runtime candidate and run the file with it.
fn run_with_runtime(
    language: &str,
    extension: &str,
    candidates: &[&[&str]],
    file: &Path,
) -> i32 {
    for invocation in candidates {
        if is_available(invocation[0]) {
            return run_with(invocation, file);
        }
    }
    eprintln!(
        "No runtime found for {language} (.{extension}); tried: {}.",
        candidates
            .iter()
            .map(|c| c[0])
            .collect::<Vec<_>>()
            .join(", ")
    );
    1
}

/// Run a previously compiled executable.
fn run_executable(path: &Path) -> i32 {
    Command::new(path).status().map(exit_code).unwrap_or(1)
}

/// Run a source file through a runtime invocation, e.g. `["go", "run"]` becomes
/// `go run <file>`.
fn run_with(invocation: &[&str], file: &Path) -> i32 {
    Command::new(invocation[0])
        .args(&invocation[1..])
        .arg(file)
        .status()
        .map(exit_code)
        .unwrap_or(1)
}

/// Whether a command exists and can be launched on this host.
fn is_available(cmd: &str) -> bool {
    Command::new(cmd).arg("--version").output().is_ok()
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

    // ---- compiled languages ------------------------------------------------

    #[test]
    fn test_run_c() {
        if !is_available("cc") {
            return;
        }
        let file = write_temp("prog.c", "int main(void) { return 0; }\n");
        assert_eq!(run("C", "c", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_c_propagates_exit_code() {
        if !is_available("cc") {
            return;
        }
        let file = write_temp("ret.c", "int main(void) { return 7; }\n");
        assert_eq!(run("C", "c", &file), 7);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_cpp() {
        if !is_available("c++") {
            return;
        }
        let file = write_temp("prog.cpp", "int main() { return 0; }\n");
        assert_eq!(run("C++", "cpp", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_rust() {
        if !is_available("rustc") {
            return;
        }
        let file = write_temp("prog.rs", "fn main() { std::process::exit(0); }\n");
        assert_eq!(run("Rust", "rs", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    // ---- shell: runtime depends on the extension ---------------------------

    #[test]
    fn test_run_sh() {
        let file = write_temp("ok.sh", "exit 0\n");
        assert_eq!(run("Shell", "sh", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_bash_uses_bash() {
        if !is_available("bash") {
            return;
        }
        // `[[ ... ]]` is a bashism that plain `sh`/dash would reject, proving the
        // .bash extension selected the bash runtime rather than sh.
        let file = write_temp("b.bash", "if [[ 1 == 1 ]]; then exit 0; else exit 1; fi\n");
        assert_eq!(run("Shell", "bash", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_shell_propagates_exit_code() {
        let file = write_temp("fail.sh", "exit 3\n");
        assert_eq!(run("Shell", "sh", &file), 3);
        let _ = std::fs::remove_file(&file);
    }

    // ---- GC / interpreted languages ----------------------------------------

    #[test]
    fn test_run_python() {
        if !is_available("python3") {
            return;
        }
        let file = write_temp("prog.py", "import sys\nsys.exit(0)\n");
        assert_eq!(run("Python", "py", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_python_propagates_exit_code() {
        if !is_available("python3") {
            return;
        }
        let file = write_temp("ret.py", "import sys\nsys.exit(5)\n");
        assert_eq!(run("Python", "py", &file), 5);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_javascript() {
        if !is_available("node") {
            return;
        }
        let file = write_temp("prog.js", "process.exit(0)\n");
        assert_eq!(run("JavaScript", "js", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_ruby() {
        if !is_available("ruby") {
            return;
        }
        let file = write_temp("prog.rb", "exit 0\n");
        assert_eq!(run("Ruby", "rb", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_go() {
        if !is_available("go") {
            return;
        }
        let file = write_temp("prog.go", "package main\nfunc main() {}\n");
        assert_eq!(run("Go", "go", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_java() {
        if !is_available("java") {
            return;
        }
        let file = write_temp(
            "prog.java",
            "class Main { public static void main(String[] a) {} }\n",
        );
        assert_eq!(run("Java", "java", &file), 0);
        let _ = std::fs::remove_file(&file);
    }

    // ---- error handling ----------------------------------------------------

    #[test]
    fn test_unsupported_language_returns_error() {
        let file = write_temp("x.cs", "// c#\n");
        assert_eq!(run("C#", "cs", &file), 1);
        let _ = std::fs::remove_file(&file);
    }
}
