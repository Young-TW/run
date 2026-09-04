use std::path::Path;
use std::process::{Command, ExitStatus};

/// Exit code returned when the required compiler / interpreter is not installed.
/// Matches the shell convention for "command not found".
pub const EXIT_RUNTIME_NOT_FOUND: i32 = 127;

/// Exit code returned when the language itself is not supported by this tool.
pub const EXIT_UNSUPPORTED: i32 = 2;

/// Exit code returned when a compiled language fails to compile.
pub const EXIT_COMPILE_FAILED: i32 = 1;

/// Compile-and-run (for compiled languages) or invoke the matching runtime (for
/// interpreted / managed languages) for a source file, returning the executed
/// program's exit code. `args` are forwarded to the executed program.
///
/// The runtime is chosen from the file `extension`, so e.g. `.sh` runs with
/// `sh` while `.bash` runs with `bash`. When no execution environment is
/// available the function exits safely and consistently with
/// [`EXIT_RUNTIME_NOT_FOUND`] rather than panicking.
pub fn run(language: &str, extension: &str, file: &Path, args: &[String]) -> i32 {
    match language {
        "C" | "C++" | "Rust" => run_compiled(language, file, args),
        _ => match runtimes(extension) {
            Some(candidates) => run_with_runtime(language, candidates, file, args),
            None => {
                eprintln!("{language} is not supported yet.");
                EXIT_UNSUPPORTED
            }
        },
    }
}

/// Compile a source file and run the resulting executable. Detects a missing
/// compiler up front so it can report it consistently instead of mistaking it
/// for a compilation error.
fn run_compiled(language: &str, file: &Path, args: &[String]) -> i32 {
    let compiler = match crate::compile::compiler(language) {
        Some(compiler) => compiler,
        None => {
            eprintln!("{language} is not supported yet.");
            return EXIT_UNSUPPORTED;
        }
    };

    if !is_available(compiler) {
        return missing_runtime(language, &[compiler]);
    }

    match crate::compile::compile_code(language, file) {
        Some(executable) => {
            let code = run_executable(&executable, args);
            // The executable lives in a temp dir; remove it once it has run.
            let _ = std::fs::remove_file(&executable);
            code
        }
        None => {
            eprintln!("Failed to compile {language} code.");
            EXIT_COMPILE_FAILED
        }
    }
}

/// Candidate runtime invocations for an interpreted / managed language, keyed by
/// file extension. Each candidate is `(invocation, needs_separator)` where
/// `invocation` is the program name followed by any leading arguments (the
/// source file is appended afterwards) and `needs_separator` tells whether
/// user-supplied program arguments must be placed after a `--` separator —
/// e.g. `zig run file.zig -- args` and `dotnet run file.cs -- args` require it,
/// while `python3 file.py args` does not. The first candidate whose program is
/// installed is used, allowing graceful fallbacks.
fn runtimes(extension: &str) -> Option<&'static [(&'static [&'static str], bool)]> {
    match extension {
        "sh" => Some(&[(&["sh"], false)]),
        "bash" => Some(&[(&["bash"], false)]),
        "py" => Some(&[(&["python3"], false), (&["python"], false)]),
        "js" => Some(&[(&["node"], false)]),
        "ts" => Some(&[
            (&["bun"], false),
            (&["tsx"], false),
            (&["ts-node"], false),
            (&["deno", "run"], false),
        ]),
        "rb" => Some(&[(&["ruby"], false)]),
        "go" => Some(&[(&["go", "run"], false)]),
        "java" => Some(&[(&["java"], false)]),
        "zig" => Some(&[(&["zig", "run"], true)]),
        "cs" => Some(&[(&["dotnet-script"], true), (&["dotnet", "run"], true)]),
        "php" => Some(&[(&["php"], false)]),
        "lua" => Some(&[(&["lua"], false), (&["luajit"], false)]),
        "pl" => Some(&[(&["perl"], false)]),
        "r" => Some(&[(&["Rscript"], false)]),
        "hs" => Some(&[(&["runghc"], false), (&["runhaskell"], false)]),
        "swift" => Some(&[(&["swift"], false)]),
        "dart" => Some(&[(&["dart"], false)]),
        "ex" | "exs" => Some(&[(&["elixir"], false)]),
        _ => None,
    }
}

/// Pick the first available runtime candidate and run the file with it, or
/// report a missing runtime consistently if none are installed.
fn run_with_runtime(
    language: &str,
    candidates: &[(&[&str], bool)],
    file: &Path,
    args: &[String],
) -> i32 {
    for (invocation, needs_separator) in candidates {
        if is_available(invocation[0]) {
            return run_with(invocation, *needs_separator, file, args);
        }
    }
    let tools: Vec<&str> = candidates.iter().map(|c| c.0[0]).collect();
    missing_runtime(language, &tools)
}

/// Report that no execution environment is installed for `language` and return
/// a consistent exit code. `tools` lists the commands that were looked for.
fn missing_runtime(language: &str, tools: &[&str]) -> i32 {
    eprintln!(
        "No runtime found for {language}: please install {}.",
        tools.join(" or ")
    );
    EXIT_RUNTIME_NOT_FOUND
}

/// Run a previously compiled executable.
fn run_executable(path: &Path, args: &[String]) -> i32 {
    Command::new(path)
        .args(args)
        .status()
        .map(exit_code)
        .unwrap_or(1)
}

/// Run a source file through a runtime invocation, e.g. `["go", "run"]` becomes
/// `go run <file>`. When `needs_separator` is set and the user supplied
/// arguments, a `--` is inserted between the file and the user arguments —
/// required by runtimes like `zig run` and `dotnet run` that would otherwise
/// mistake the program arguments for their own.
fn run_with(invocation: &[&str], needs_separator: bool, file: &Path, args: &[String]) -> i32 {
    let mut command = Command::new(invocation[0]);
    command.args(&invocation[1..]).arg(file);
    if needs_separator && !args.is_empty() {
        command.arg("--");
    }
    command.args(args).status().map(exit_code).unwrap_or(1)
}

/// Whether a command exists and can be launched on this host, resolved by
/// searching PATH rather than spawning the command.
fn is_available(cmd: &str) -> bool {
    if cmd.contains(['/', '\\']) {
        return is_executable(Path::new(cmd));
    }
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(cmd)))
    })
}

/// Whether `path` points to an executable file.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Windows has no executable bit; probe for `cmd.<ext>` using PATHEXT instead,
/// since PATH entries list commands without their extension (e.g. `dotnet`
/// actually lives on disk as `dotnet.EXE`).
#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    if path.extension().is_some() {
        return path.is_file();
    }
    std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
        .split(';')
        .filter(|ext| !ext.is_empty())
        .any(|ext| path.with_extension(ext.trim_start_matches('.')).is_file())
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
        assert_eq!(run("C", "c", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_c_propagates_exit_code() {
        if !is_available("cc") {
            return;
        }
        let file = write_temp("ret.c", "int main(void) { return 7; }\n");
        assert_eq!(run("C", "c", &file, &[]), 7);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_cpp() {
        if !is_available("c++") {
            return;
        }
        let file = write_temp("prog.cpp", "int main() { return 0; }\n");
        assert_eq!(run("C++", "cpp", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_rust() {
        if !is_available("rustc") {
            return;
        }
        let file = write_temp("prog.rs", "fn main() { std::process::exit(0); }\n");
        assert_eq!(run("Rust", "rs", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    // ---- shell: runtime depends on the extension ---------------------------

    #[test]
    fn test_run_sh() {
        let file = write_temp("ok.sh", "exit 0\n");
        assert_eq!(run("Shell", "sh", &file, &[]), 0);
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
        assert_eq!(run("Shell", "bash", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_shell_propagates_exit_code() {
        let file = write_temp("fail.sh", "exit 3\n");
        assert_eq!(run("Shell", "sh", &file, &[]), 3);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_forwards_program_args() {
        let file = write_temp("args.sh", "exit \"$1\"\n");
        assert_eq!(run("Shell", "sh", &file, &["4".to_string()]), 4);
        let _ = std::fs::remove_file(&file);
    }

    // ---- GC / interpreted languages ----------------------------------------

    #[test]
    fn test_run_python() {
        if !is_available("python3") {
            return;
        }
        let file = write_temp("prog.py", "import sys\nsys.exit(0)\n");
        assert_eq!(run("Python", "py", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_python_propagates_exit_code() {
        if !is_available("python3") {
            return;
        }
        let file = write_temp("ret.py", "import sys\nsys.exit(5)\n");
        assert_eq!(run("Python", "py", &file, &[]), 5);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_javascript() {
        if !is_available("node") {
            return;
        }
        let file = write_temp("prog.js", "process.exit(0)\n");
        assert_eq!(run("JavaScript", "js", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_ruby() {
        if !is_available("ruby") {
            return;
        }
        let file = write_temp("prog.rb", "exit 0\n");
        assert_eq!(run("Ruby", "rb", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_go() {
        if !is_available("go") {
            return;
        }
        let file = write_temp("prog.go", "package main\nfunc main() {}\n");
        assert_eq!(run("Go", "go", &file, &[]), 0);
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
        assert_eq!(run("Java", "java", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_zig() {
        if !is_available("zig") {
            return;
        }
        let file = write_temp("prog.zig", "pub fn main() void {}\n");
        assert_eq!(run("Zig", "zig", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_csharp() {
        // The first candidate is dotnet-script; `dotnet run` on a loose .cs
        // file requires a .NET 10+ SDK (file-based programs), so the plain
        // presence check used for other runtimes is not sufficient here.
        if !is_available("dotnet-script") && !dotnet_supports_file_based_programs() {
            return;
        }
        let file = write_temp("prog.cs", "System.Console.WriteLine();\n");
        assert_eq!(run("C#", "cs", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    /// Whether an installed dotnet SDK is new enough (>= 10) to run a loose
    /// .cs file via `dotnet run`. Test-only; the runner itself keeps the
    /// simple "first available candidate wins" model.
    fn dotnet_supports_file_based_programs() -> bool {
        let Ok(output) = Command::new("dotnet").arg("--version").output() else {
            return false;
        };
        if !output.status.success() {
            return false;
        }
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .split('.')
            .next()
            .and_then(|major| major.parse::<u32>().ok())
            .is_some_and(|major| major >= 10)
    }

    #[test]
    fn test_run_php() {
        if !is_available("php") {
            return;
        }
        let file = write_temp("prog.php", "<?php exit(0);\n");
        assert_eq!(run("PHP", "php", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_lua() {
        if !is_available("lua") {
            return;
        }
        let file = write_temp("prog.lua", "os.exit(0)\n");
        assert_eq!(run("Lua", "lua", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_perl() {
        if !is_available("perl") {
            return;
        }
        let file = write_temp("prog.pl", "exit 0;\n");
        assert_eq!(run("Perl", "pl", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_perl_propagates_exit_code() {
        if !is_available("perl") {
            return;
        }
        let file = write_temp("ret.pl", "exit 4;\n");
        assert_eq!(run("Perl", "pl", &file, &[]), 4);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_r() {
        if !is_available("Rscript") {
            return;
        }
        let file = write_temp("prog.r", "quit(save = \"no\", status = 0)\n");
        assert_eq!(run("R", "r", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_haskell() {
        if !is_available("runghc") {
            return;
        }
        let file = write_temp("prog.hs", "main = return ()\n");
        assert_eq!(run("Haskell", "hs", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_swift() {
        if !is_available("swift") {
            return;
        }
        let file = write_temp("prog.swift", "// top-level script\n");
        assert_eq!(run("Swift", "swift", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_dart() {
        if !is_available("dart") {
            return;
        }
        let file = write_temp("prog.dart", "void main() {}\n");
        assert_eq!(run("Dart", "dart", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_elixir() {
        if !is_available("elixir") {
            return;
        }
        let file = write_temp("prog.ex", ":ok\n");
        assert_eq!(run("Elixir", "ex", &file, &[]), 0);
        let _ = std::fs::remove_file(&file);
    }

    // ---- missing runtime / error handling ----------------------------------

    #[test]
    fn test_is_available_detects_present_and_missing() {
        assert!(is_available("sh"));
        assert!(!is_available("definitely-not-a-real-binary-zzzqx"));
    }

    #[test]
    fn test_missing_runtime_exit_code() {
        // Consistent, safe exit code when an execution environment is absent.
        assert_eq!(
            missing_runtime("Imaginary", &["nope", "alsonope"]),
            EXIT_RUNTIME_NOT_FOUND
        );
    }

    #[test]
    fn test_unsupported_language_returns_error() {
        let file = write_temp("x.kt", "// kotlin\n");
        assert_eq!(run("Kotlin", "kt", &file, &[]), EXIT_UNSUPPORTED);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn test_run_compiled_without_known_compiler_is_safe() {
        // The compiled path must exit safely (no panic) for a language that has
        // no known compiler, rather than trying to spawn a missing program.
        let file = write_temp("x.txt", "");
        assert_eq!(run_compiled("Nonexistent", &file, &[]), EXIT_UNSUPPORTED);
        let _ = std::fs::remove_file(&file);
    }
}
