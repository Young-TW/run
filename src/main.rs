mod compile;
mod language;
mod file_extension;

fn main() {
    let extension = file_extension::parse_extension("example.cpp").unwrap_or("unknown");
    let language = language::specify_language(extension);
    println!("The language for .{} files is {}", extension, language);
}
