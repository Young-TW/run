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
        "zig" => "Zig",
        "cs" => "C#",
        "php" => "PHP",
        "lua" => "Lua",
        "pl" => "Perl",
        "r" => "R",
        "hs" => "Haskell",
        "swift" => "Swift",
        "dart" => "Dart",
        "ex" | "exs" => "Elixir",
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
        assert_eq!(specify_language("zig"), "Zig");
        assert_eq!(specify_language("cs"), "C#");
        assert_eq!(specify_language("php"), "PHP");
        assert_eq!(specify_language("lua"), "Lua");
        assert_eq!(specify_language("pl"), "Perl");
        assert_eq!(specify_language("r"), "R");
        assert_eq!(specify_language("hs"), "Haskell");
        assert_eq!(specify_language("swift"), "Swift");
        assert_eq!(specify_language("dart"), "Dart");
        assert_eq!(specify_language("ex"), "Elixir");
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
    fn test_elixir_aliases() {
        assert_eq!(specify_language("ex"), "Elixir");
        assert_eq!(specify_language("exs"), "Elixir");
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
        // Notably the conventional upper-case `.R` extension is not supported.
        assert_eq!(specify_language("RS"), "Unknown");
        assert_eq!(specify_language("CPP"), "Unknown");
        assert_eq!(specify_language("R"), "Unknown");
    }
}
