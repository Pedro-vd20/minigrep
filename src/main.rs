use minigrep::{Flags, search};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead, BufReader, Lines, StdinLock},
    process,
};

struct Config {
    pattern: String,
    file_path: Option<Vec<String>>, // fall back to stdin if none
    flags: Flags,
}

impl Config {
    fn from_args(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments. Pattern and file path are required");
        }

        let query = args[1].clone(); // not the most efficient way to avoid borrow issues
        let mut paths: Vec<String> = Vec::new();
        let file_path = &args[2..];
        paths.extend_from_slice(file_path);

        let ignore_case = !env::var("IGNORE_CASE").is_ok(); // default value is case sensitive (true)
        let flags = Flags::new(ignore_case);

        Ok(Config {
            pattern: query,
            file_path: Some(paths),
            flags,
        })
    }
}

enum Input<'a> {
    StdIn,
    File(&'a str),
}

impl<'a> Input<'a> {
    fn get_name(&self) -> &'a str {
        match self {
            Input::StdIn => "stdin",
            Input::File(file_name) => file_name,
        }
    }
}

enum InputIterator<'a> {
    StdIn(Lines<StdinLock<'a>>),
    File {
        lines: Option<Lines<BufReader<fs::File>>>,
        file_name: &'a str,
    },
}

impl<'a> Iterator for InputIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            InputIterator::StdIn(lines) => lines
                .next()
                .and_then(|line| match line {
                    Ok(value) => Some(value),
                    Err(error) => {
                        eprintln!("Faulure to read from stdin: {error}");
                        None
                    },
                }),
            InputIterator::File {
                lines: maybe_lines,
                file_name,
            } => {
                let Some(lines) = maybe_lines else {
                    return None;
                };
                lines
                    .next()
                    .and_then(|line| match line {
                        Ok(contents) => Some(contents),
                        Err(e) => {
                            eprintln!("Failed to read {file_name}: {e}");
                            None
                        }
                    })
            }
        }
    }
}

impl<'a> Input<'a> {
    fn to_iter(&self) -> InputIterator<'a> {
        match self {
            Input::StdIn => InputIterator::StdIn(io::stdin().lock().lines()),
            Input::File(file_name) => InputIterator::File {
                lines: match fs::File::open(file_name) {
                    Ok(file) => Some(BufReader::new(file).lines()),
                    Err(e) => {
                        eprintln!("Failed to open {file_name}: {e}");
                        None
                    }
                },
                file_name,
            },
        }
    }
}

/// Selects the correct input for the string to match based on config.
/// If no file, reads stdin, otherwise reads file contents
fn to_inputs<'a>(maybe_file_path: &'a Option<Vec<String>>) -> Vec<Input<'a>> {
    match maybe_file_path {
        Some(file_paths) => file_paths
            .iter()
            .map(|file_name| Input::File(file_name))
            .collect(),
        None => vec![Input::StdIn],
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let inputs = to_inputs(&config.file_path);

    let should_print_file = inputs.len() > 1;
    for input in inputs {
        for line in search(
            &config.pattern,
            input.to_iter(),
            Some(config.flags.ignore_case),
        ) {
            match should_print_file {
                true => println!("{}\t{}", input.get_name(), line),
                false => println!("{line}"),
            }
        }
    }

    Ok(())
}

fn main() {
    // TODO args vs args OS
    let args: Vec<String> = env::args().collect();

    let config = Config::from_args(&args).unwrap_or_else(|error| {
        eprintln!("Problem parsing arguments: {error}");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
