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
        "sh" | "bash" => "Shell",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_languages() {
        assert_eq!(specify_language("rs"), "Rust");
        assert_eq!(specify_language("py"), "Python");
        assert_eq!(specify_language("js"), "JavaScript");
        assert_eq!(specify_language("java"), "Java");
        assert_eq!(specify_language("c"), "C");
        assert_eq!(specify_language("rb"), "Ruby");
        assert_eq!(specify_language("go"), "Go");
        assert_eq!(specify_language("ts"), "TypeScript");
    }

    #[test]
    fn test_cpp_aliases() {
        assert_eq!(specify_language("cpp"), "C++");
        assert_eq!(specify_language("cc"), "C++");
        assert_eq!(specify_language("cxx"), "C++");
    }

    #[test]
    fn test_shell_aliases() {
        assert_eq!(specify_language("sh"), "Shell");
        assert_eq!(specify_language("bash"), "Shell");
    }

    #[test]
    fn test_unknown_extension() {
        assert_eq!(specify_language("txt"), "Unknown");
        assert_eq!(specify_language(""), "Unknown");
        assert_eq!(specify_language("rust"), "Unknown");
    }

    #[test]
    fn test_case_sensitive() {
        // Extensions are matched case-sensitively; upper-case is not recognised.
        assert_eq!(specify_language("RS"), "Unknown");
        assert_eq!(specify_language("CPP"), "Unknown");
    }
}
