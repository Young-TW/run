use std::path::Path;

pub fn parse_extension(file_path: &str) -> Option<&str> {
    std::path::Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extension() {
        assert_eq!(parse_extension("file.rs"), Some("rs"));
        assert_eq!(parse_extension("main.cpp"), Some("cpp"));
        assert_eq!(parse_extension("file.txt"), Some("txt"));
        assert_eq!(parse_extension("file"), None);
    }
}