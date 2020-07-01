use clap::Clap;
use publication::{extensions, Emitter, Parser};
use std::convert::TryInto;
use std::fs::{read_to_string, write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clap, Debug)]
struct Options {
    input: PathBuf,
    #[clap(short, long)]
    out: Option<PathBuf>,

    // Built-in extensions
    #[clap(short, long)]
    bold: bool,
    #[clap(short, long)]
    italics: bool,
    #[clap(short, long)]
    list: Option<String>,
}

fn main() {
    let Options {
        input,
        out,
        bold,
        italics,
        list,
    } = Options::parse();

    let output = match input.extension() {
        Some(ext) if ext == "publ" => out.unwrap_or_else(|| {
            let mut output = input.clone();
            output.set_extension("html");
            output
        }),
        Some(i) => {
            eprintln!(
                "Publication files must use the .publ extension, so .{} cannot be used.",
                i.to_string_lossy()
            );
            return;
        }
        None => {
            eprintln!("Publication files must use the .publ extension.");
            return;
        }
    };

    let start = Instant::now();

    let raw = match read_to_string(&input) {
        Err(e) => {
            eprintln!("Could not read {}: {}", input.display(), e);
            return;
        }
        Ok(raw) => raw,
    };

    let emitter: Box<dyn Emitter> = match output.as_path().try_into() {
        Ok(e) => e,
        Err(()) => {
            match output.extension() {
                None => {
                    eprintln!(
                        "To infer emitter, please provide an output file with a known extension."
                    );
                }
                Some(ext) => {
                    eprintln!("No known emitter for .{} files.", ext.to_string_lossy());
                }
            }
            return;
        }
    };

    let mut parser = Parser::new(raw);

    if bold {
        parser.add_extension(extensions::Bold);
    }

    if italics {
        parser.add_extension(extensions::Italics);
    }

    if let Some(bullet) = list {
        parser.add_extension(extensions::Lists::new(bullet));
    }

    let emitted = match parser.emit_with(emitter.as_ref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse {}: {}", input.display(), e);
            return;
        }
    };

    match write(&output, emitted) {
        Err(e) => {
            eprintln!("Could not write to {}: {}", output.display(), e);
            return;
        }
        Ok(()) => {}
    }

    println!(
        "{} → {} ({}µs)",
        input.display(),
        output.display(),
        start.elapsed().as_micros()
    );
}
