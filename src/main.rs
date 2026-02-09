use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Generator of fake pronounceable words")]
struct Args {
    /// number of words to generate
    #[arg(short, long, default_value_t = 1)]
    number: u8,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.number {
        println!("{}", String::from("flabild"));
    }
}
