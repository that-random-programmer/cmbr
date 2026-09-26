use std::{fs::File, io::Read, process::exit};

use crate::{
    parser::Parser,
    tokenizer::Tokenizer,
    treewalker::Treewalker,
    types::{CLIInterpreterIO, DiagnosticPrinter},
};
use clap::Parser as ClapParser;
use std::path::PathBuf;
mod parser;
mod tokenizer;
mod treewalker;
mod types;
use owo_colors::OwoColorize;



fn main() {
    let cli = Cli::parse();

    let mut buf = String::new();
    match File::open(&cli.path) {
        Ok(mut f) => match f.read_to_string(&mut buf) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}{}", "error".red().bold(), format!(": {}", e.bold()));
                exit(1)
            }
        },
        Err(e) => {
            eprintln!("{}{}", "error".red().bold(), format!(": {}", e.bold()));
            exit(1);
        }
    };

    let fp = cli.path.to_string_lossy();
    let tk = Tokenizer::new(buf.clone(), fp.clone());
    let diag_printer = DiagnosticPrinter::new(buf, CLIInterpreterIO);
    let mut tkns = Vec::new();
    for tkn in tk {
        match tkn {
            Ok(v) => {
                if let Some(tkn) = v {
                    tkns.push(tkn)
                }
            }
            Err(e) => {
                diag_printer.print_diagnostic(&e);
                exit(1)
            }
        }
    }

    let mut psr = Parser::new(tkns, fp);
    let ast = match psr.parse() {
        Ok(v) => v,
        Err(e) => {
            diag_printer.print_diagnostic(&e);
            exit(1)
        }
    };

    let mut walker = Treewalker::new(&ast, CLIInterpreterIO);
    if let Err(e) = walker.run() {
        diag_printer.print_diagnostic(&e);
        exit(1)
    };
}

#[derive(ClapParser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: PathBuf,
    #[arg(short, long)]
    interactive: bool,
}
