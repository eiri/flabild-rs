use clap::Parser;

use flabild::Chooser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Generator of fake pronounceable words")]
struct Args {
    /// number of words to generate
    #[arg(short, long, default_value_t = 1)]
    number: u8,
}

fn main() {
    let args = Args::parse();

    let mut chooser = Chooser::new();
    let _ = chooser.from_file("src/choices.cbor");

    for _ in 0..args.number {
        match chooser.word() {
            Ok(word) => println!("{}", word),
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1)
            }
        }
    }
}
