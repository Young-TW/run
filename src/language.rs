pub fn specify_language(extension: &str) -> &'static str {
    match extension {
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "java" => "Java",
        "cpp" | "cc" | "cxx" => "C++",
        "c" => "C",
        "rb" => "Ruby",
        "go" => "Go",
        "ts" => "TypeScript",
        _ => "Unknown",
    }
}
