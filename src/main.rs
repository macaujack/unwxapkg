use clap::Parser;
use std::path::Path;

const DEFAULT_OUTPUT_PATH: &str = "Same name with input file (with suffix stripped)";

#[derive(Debug, Parser)]
#[command(name = "unwxapkg")]
#[command(author = "Jack Y. <seigino.mikata@outlook.com>")]
#[command(about = "A tool to decode wxapkg-formatted file.", long_about = None)]
struct CommandLineArgs {
    /// The input wxapkg file path
    #[arg(short, long)]
    input: String,

    /// The output directory path
    #[arg(short, long, default_value_t = String::from(DEFAULT_OUTPUT_PATH))]
    output: String,

    /// Set this to remove the output directory before proceeding if it already exists
    #[arg(short, long, action)]
    force: bool,
}

fn main() {
    let args = CommandLineArgs::parse();
    let input_wxapkg = Path::new(&args.input);
    if !input_wxapkg.is_file() {
        eprintln!(
            "Path '{}' doesn't exist or it isn't a file",
            input_wxapkg.to_str().unwrap()
        );
        return;
    }

    let output_dir: &str = if args.output == DEFAULT_OUTPUT_PATH {
        input_wxapkg.file_stem().unwrap().to_str().unwrap()
    } else {
        &args.output
    };
    let output_dir = Path::new(output_dir);
    if output_dir.is_file() {
        eprintln!(
            "Path '{}' is a file, which is not expected",
            output_dir.to_str().unwrap()
        );
        return;
    }
    if output_dir.is_dir() {
        if !args.force {
            eprintln!(
                "Path '{}' already exists. Add '-f' option to remove it before proceeding",
                output_dir.to_str().unwrap()
            );
            return;
        }
        std::fs::remove_dir_all(output_dir).expect("Fail removing output directory");
    }
    std::fs::create_dir(output_dir).expect("Fail creating output directory");

    let mut input_wxapkg =
        std::fs::File::open(input_wxapkg).expect("Error reading input wxapkg file");
    let miniapp_files = match unwxapkg::decode_wxapkg(&mut input_wxapkg) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error decoding wxapkg file: {}", e);
            return;
        }
    };

    for miniapp_file in &miniapp_files {
        if miniapp_file.filename.len() == 0 {
            eprintln!("Filename with 0 length encountered");
            continue;
        }
        let file_path = match miniapp_file.filename.as_bytes()[0] as char {
            '/' => &miniapp_file.filename[1..],
            _ => &miniapp_file.filename,
        };
        let file_path = Path::new(file_path);
        let file_path = output_dir.join(file_path);
        let dir_path = file_path.parent().unwrap();
        std::fs::create_dir_all(&dir_path).expect(&format!(
            "Cannot create directory '{}'",
            dir_path.clone().to_str().unwrap()
        ));
        std::fs::write(&file_path, &miniapp_file.content).expect(&format!(
            "Error writing to file '{}'",
            file_path.to_str().unwrap()
        ));
    }
}
