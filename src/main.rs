use clap::{App, Arg};
use elepages::cli::{run, Parameters};
use std::path::PathBuf;

fn main() {
    let matches = App::new("Ele pages")
        .version("0.1")
        .about("Flexible static pages generator")
        .arg(Arg::with_name("source").short("src").long("source").help("source directory to be parsed").takes_value(true))
        .arg(
            Arg::with_name("destination")
                .short("dest")
                .long("destination")
                .help("destination directory where output pages will be written")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("stages configuration file (yaml/json formats)")
                .takes_value(true),
        )
        .get_matches();

    let params = Parameters::new(
        matches.value_of("source").map(PathBuf::from),
        matches.value_of("dest").map(PathBuf::from),
        matches.value_of("config").map(PathBuf::from),
    )
    .unwrap();

    let execution = run(&params).unwrap();
    println!("{:?}", execution);
}
