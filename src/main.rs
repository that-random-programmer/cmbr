use std::{fs::File, io::Read, process::exit, rc::Rc};

use crate::{parser::Parser, tokenizer::Tokenizer, treewalker::Treewalker, types::Failure};
use clap::Parser as ClapParser;
use std::path::PathBuf;
mod parser;
mod tokenizer;
mod treewalker;
mod types;
use owo_colors::OwoColorize;

struct DiagnosticPrinter {
    file: Rc<str>,
}
impl DiagnosticPrinter {
    fn print_diagnostic(&self, diag: &impl Failure) {
        println!(
            "{}{}",
            "error".red().bold(),
            format!(": {}", diag.msg().bold())
        );

        if let Some(span) = &diag.span() {
            println!("at {span}\n");
            let start = span.ln.saturating_sub(3);
            let end = match span.endln {
                Some(v) => v + 3,
                None => span.ln + 3,
            }
            .clamp(0, self.file.lines().count());
            let line_count = end - start;
            let mut arrow_drawn = false;
            let no_endln = match span.endln {
                Some(v) => v == span.ln,
                None => true,
            };
            for (ln, i) in self
                .file
                .lines()
                .skip(start)
                .take(line_count)
                .zip(start..end)
            {
                let start;
                if let Some(endln) = span.endln
                    && !no_endln
                    && span.ln < i
                    && i <= endln
                    && !arrow_drawn
                {
                    start = "-> ";
                    arrow_drawn = true;
                } else if span.ln == i {
                    start = "-> ";
                    arrow_drawn = true;
                } else {
                    start = "   "
                }
                if span.endln.is_none()
                    || span.endln.is_some_and(|endln| !(endln > i && i > span.ln))
                {
                    println!(
                        "{}{}{}",
                        start.red().bold(),
                        format!("{:4} | ", i + 1).blue().bold(),
                        ln
                    );
                }
                if let Some(endln) = span.endln
                    && span.ln <= i
                    && i <= endln
                    && let Some(endcol) = span.endcol
                    && !no_endln
                {
                    if i == span.ln {
                        println!(
                            "        {} {}{} {}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".repeat(ln.chars().count() - span.col + 1).red().bold(),
                            "from here".purple().bold()
                        );
                    } else if i == span.ln + 1 {
                        println!("        {}", "| ...".blue().bold())
                    } else if i == endln {
                        println!(
                            "        {} {}{} {}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".repeat(endcol).red().bold(),
                            "to here".purple().bold()
                        );
                    }
                } else if span.ln == i && no_endln {
                    if let Some(endcol) = span.endcol {
                        println!(
                            "        {} {}{}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".repeat(endcol - span.col + 1).red().bold()
                        )
                    } else {
                        println!(
                            "        {} {}{}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".red().bold()
                        )
                    }
                }
            }
            println!()
        }
        for info in diag.info() {
            println!("{}{}", info.ty.bold().blue(), format!(": {}", info.msg));
        }
    }
}
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
    let diag_printer = DiagnosticPrinter { file: buf.into() };
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


    let mut walker = Treewalker::new(&ast);
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
