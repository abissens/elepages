use clap::{App, Arg, ArgMatches};
use elepages::cli::{Execution, Executor, ExecutorParams};
use elepages::config::Value;
use elepages::pages::{Env, PrintLevel, PRINT_LEVEL_VVV};
use elepages::stages::ProcessingResult;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let matches = App::new("Ele pages")
        .version("0.1")
        .about("Flexible static pages generator")
        .arg(Arg::with_name("source").long("source").help("source directory to be parsed").takes_value(true))
        .arg(
            Arg::with_name("destination")
                .long("destination")
                .help("destination directory where output pages will be written")
                .takes_value(true),
        )
        .arg(Arg::with_name("config").long("config").help("stages configuration file (yaml/json formats)").takes_value(true))
        .arg(Arg::with_name("git_path").long("git-path").help("git metadata path").takes_value(true))
        .arg(
            Arg::with_name("handlebars_str_config")
                .long("handlebars")
                .help("handlebars template local path or remote git url")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("handlebars_path_config")
                .long("handlebars-path")
                .help("handlebars path config. This is local template path or remote git relative folder path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("handlebars_remote_config")
                .long("handlebars-remote")
                .help("handlebars git remote config")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("handlebars_remote_commit_config")
                .long("handlebars-commit")
                .help("handlebars git remote commit config")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("handlebars_remote_tag_config")
                .long("handlebars-tag")
                .help("handlebars git remote tag config")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("handlebars_remote_branch_config")
                .long("handlebars-branch")
                .help("handlebars git remote branch config")
                .takes_value(true),
        )
        .arg(Arg::with_name("v").short("v").multiple(true).help("Sets the level of verbosity"))
        .get_matches();

    let params = ExecutorParams {
        input_dir: matches.value_of("source").map(PathBuf::from),
        output_dir: matches.value_of("destination").map(PathBuf::from),
        config_path: matches.value_of("config").map(PathBuf::from),
        git_repo_path_config: matches.value_of("git_path").map(|v| v.to_string()),
        handlebars_config: make_handlebars_config(&matches),
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

fn make_handlebars_config(matches: &ArgMatches) -> Option<Value> {
    if let Some(v) = matches.value_of("handlebars_str_config") {
        return Some(Value::String(v.to_string()));
    }

    if matches.is_present("handlebars_path_config") || matches.is_present("handlebars_remote_config") {
        let mut result = HashMap::new();
        if let Some(v) = matches.value_of("handlebars_path_config") {
            result.insert("path".to_string(), Value::String(v.to_string()));
        }
        if let Some(v) = matches.value_of("handlebars_remote_config") {
            result.insert("remote".to_string(), Value::String(v.to_string()));
        }
        if let Some(v) = matches.value_of("handlebars_remote_commit_config") {
            result.insert("commit".to_string(), Value::String(v.to_string()));
        }
        if let Some(v) = matches.value_of("handlebars_remote_tag_config") {
            result.insert("tag".to_string(), Value::String(v.to_string()));
        }
        if let Some(v) = matches.value_of("handlebars_remote_branch_config") {
            result.insert("branch".to_string(), Value::String(v.to_string()));
        }

        return Some(Value::Map(result));
    }

    if let Some(v) = matches.value_of("handlebars_str_config") {
        return Some(Value::String(v.to_string()));
    }

    None
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
    env.print_vvv(
        "main",
        &format!("{}{} processed from {} to {}", shift, result.stage_name, result.start.format("%T %f"), result.end.format("%T %f")),
    );
    for sub_result in &result.sub_results {
        print_execution_result(sub_result, env, &format!("{}    ", shift));
    }
}
