use anyhow::anyhow;
use clap::Parser;
use env_logger::Env;
use mordle2::dict_loader;
use mordle2::dict_loader::load_dictionary_content;
use mordle2::index::Index;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Args {
    /// Use external dictionary instead of embedded
    #[arg(short, long, value_name = "FILE")]
    dict: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    let args = Args::parse();

    let dict_content = load_dictionary_content(args.dict)?;
    let dict = dict_loader::load(&dict_content)?;
    let index = Index::from_dict(&dict)?;

    let mask = index.full_bitmap();

    match dict.best_choice(&mut rand::rng(), &mask)? {
        Some(best_choice) => println!("Selected word: \"{word}\"", word = best_choice.concat()),
        None => return Err(anyhow!("No words found")),
    }

    Ok(())
}
