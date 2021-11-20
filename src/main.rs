use chrono::{DateTime, NaiveDateTime, Utc};
use clap::{App, Arg};
use elepages::cli::{Execution, Executor, ExecutorParams};
use elepages::pages::{Env, PrintLevel, PRINT_LEVEL_VVV};
use elepages::stages::ProcessingResult;
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
        .arg(Arg::with_name("v").short("v").multiple(true).help("Sets the level of verbosity"))
        .get_matches();

    let params = ExecutorParams {
        input_dir: matches.value_of("source").map(PathBuf::from),
        output_dir: matches.value_of("dest").map(PathBuf::from),
        config_path: matches.value_of("config").map(PathBuf::from),
        print_level: match matches.occurrences_of("v") {
            0 => None,
            1 => Some(PrintLevel::V),
            2 => Some(PrintLevel::VV),
            _ => Some(PrintLevel::VVV),
        },
    };

    let executor = Executor::new(params).unwrap();
    executor.env.print_vv("main", "program started");
    let execution_result = executor.execute();
    match execution_result {
        Err(err) => panic!("{}", err),
        Ok(execution) => print_execution(execution, &executor.env),
    }
    executor.env.print_v("main", "finished !");
}

fn print_execution(execution: Execution, env: &Env) {
    env.print_v("main", &format!("loading duration : {} millis", execution.loading_elapsed.as_millis()));
    env.print_v("main", &format!("stage making duration : {} millis", execution.stage_making_elapsed.as_millis()));
    env.print_v("main", &format!("processing duration : {} millis", execution.processing_elapsed.as_millis()));
    env.print_v("main", &format!("writing duration : {} millis", execution.writing_elapsed.as_millis()));

    if !env.can_print(&PRINT_LEVEL_VVV) {
        return;
    }
    print_execution_result(&execution.processing_result, env, "");
}

fn print_execution_result(result: &ProcessingResult, env: &Env, shift: &str) {
    let start = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(result.end, 0), Utc);
    let end = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(result.end, 0), Utc);
    env.print_vvv("main", &format!("{}{} processed from {} to {}", shift, result.stage_name, start.format("%T"), end.format("%T")));
    for sub_result in &result.sub_results {
        print_execution_result(sub_result, env, &format!("{}    ", shift));
    }
}
