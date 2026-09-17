mod address;
mod json;
mod parse;

use std::io::{self, Read};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return ExitCode::SUCCESS;
    }
    let json_output = args.iter().any(|a| a == "--json");

    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read address from stdin: {e}");
        return ExitCode::FAILURE;
    }

    match parse::parse(&input) {
        Ok(address) => {
            if json_output {
                println!("{}", address.to_json());
            } else {
                println!("{}", address.pretty_print());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            if json_output {
                println!("{{\"error\":{}}}", json::string(&e.to_string()));
            } else {
                eprintln!("error: {e}");
            }
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        "addrfmt: read a US or Canadian postal address from stdin, validate it, print it back\n\
         \n\
         usage:\n\
         \x20\x20cat address.txt | addrfmt [--json]\n\
         \n\
         input is one address field per line, e.g.:\n\
         \x20\x20Jane Doe\n\
         \x20\x20500 Market St\n\
         \x20\x20Suite 210\n\
         \x20\x20San Francisco, CA 94105\n\
         \n\
         or, for a Canadian address:\n\
         \x20\x20100 Queen St W\n\
         \x20\x20Toronto, ON M5H 2N2\n\
         \n\
         the country is inferred from the two-letter region code on the\n\
         last line (a US state or a Canadian province). the recipient and\n\
         unit lines are optional. with --json the result (or the error) is\n\
         printed as a single JSON object instead of the human-readable,\n\
         re-formatted address."
    );
}
