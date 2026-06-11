mod compile;
mod file_extension;
mod language;
mod runner;

use clap::{Arg, Command};

fn main() {
    let matches = Command::new("run")
        .about("Compile/interpret and execute code snippets quickly")
        .arg(
            Arg::new("file")
                .help("Path to the source file to run")
                .required(true)
                .index(1),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").expect("file is required");

    let extension = match file_extension::parse_extension(file_path) {
        Some(ext) => ext,
        None => {
            eprintln!("Could not determine the file extension of '{file_path}'.");
            std::process::exit(1);
        }
    };

    let language = language::specify_language(extension);
    let code = runner::run(language, extension, std::path::Path::new(file_path));
    std::process::exit(code);
}
