mod compile;
mod language;

fn main() {
    let extension = "rs";
    println!("The language for .{} files is {}", extension, language::specify_language(extension));
}
