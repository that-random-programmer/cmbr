use js_sys::Function;
use owo_colors::OwoColorize;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::{
    parser::Parser,
    tokenizer::Tokenizer,
    treewalker::{RuntimeError, RuntimeErrorType, Treewalker},
    types::{DiagnosticPrinter, Info, InterpreterIO, Span},
};

#[wasm_bindgen]
pub fn run(source: &str, read_line: Function, print_hook: Function) {
    let io = WasmIO {
        read_line_hook: read_line,
        print_hook,
    };

    work(source, io.clone());

    io.println(&"<interpreter terminated>".bright_black().bold().to_string())
}

fn work(source: &str, io: WasmIO) {
    let diag_printer = DiagnosticPrinter::new(source, io.clone());
    let tknr = Tokenizer::new(source.to_string(), "<editor>");
    let mut tkns = Vec::new();
    for tkn in tknr {
        match tkn {
            Ok(v) => {
                if let Some(t) = v {
                    tkns.push(t)
                }
            }
            Err(e) => {
                diag_printer.print_diagnostic(&e);
                return;
            }
        }
    }
    let mut psr = Parser::new(tkns, "<editor>");
    let parsed = match psr.parse() {
        Ok(v) => v,
        Err(e) => {
            diag_printer.print_diagnostic(&e);
            return;
        }
    };
    let mut treewalker = Treewalker::new(&parsed, io);
    if let Err(e) = treewalker.run() {
        diag_printer.print_diagnostic(&e);
        return;
    };
}

#[derive(Clone)]
pub struct WasmIO {
    read_line_hook: js_sys::Function,
    print_hook: js_sys::Function,
}

impl InterpreterIO for WasmIO {
    // i hate how i have to use unwrap() twice here, but, we're working with JS
    fn print(&self, s: &str) {
        self.print_hook
            .call1(&JsValue::NULL, &JsValue::from_str(&s.replace("\n", "\r\n")))
            .unwrap();
    }

    fn read_line(&self, span: Span) -> Result<String, RuntimeError> {
        match self.read_line_hook.call0(&JsValue::NULL) {
            Ok(v) => Ok(v.as_string().unwrap()),
            Err(e) => Err(RuntimeError {
                msg: format!("internal error: {e:?}"),
                span,
                ty: RuntimeErrorType::InternalError,
                info: vec![Info::note("please report this error")],
            }),
        }
    }
}
