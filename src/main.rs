mod app;

use app::App;
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> color_eyre::Result<()> {
    // parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                print_version();
                return Ok(());
            }
            "--help" | "-h" => print_help(),
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                std::process::exit(1);
            }
        }
    }

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new()?.run(terminal);
    ratatui::restore();
    result
}

fn print_help() {
    todo!()
}

fn print_version() {
    println!("{NAME} - {VERSION}");
}
