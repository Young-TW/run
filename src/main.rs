mod compile;
mod language;
mod file_extension;

fn main() {
    let extension = "rs";
    println!("The language for .{} files is {}", extension, language::specify_language(extension));
}
