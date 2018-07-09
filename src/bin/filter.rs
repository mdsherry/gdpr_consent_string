extern crate gdpr_consent_string;
#[macro_use]
extern crate structopt;

mod grammar;

use gdpr_consent_string::ConsentString;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader, Read};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(name = "FILE", parse(from_os_str))]
    file: Option<PathBuf>,

    #[structopt(short = "e", long = "expression")]
    expression: Option<String>,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    cmdfile: Option<PathBuf>,
}

fn main() {
    let args = Args::from_args();
    let expr = args.expression;
    let fname = args.cmdfile;
    let prog = &expr.or_else(|| {
        fname.map(|fname| {
            let mut f = File::open(fname).expect("Could not open command file");
            let mut buf = String::new();
            f.read_to_string(&mut buf).expect("Error reading file");
            buf
        })
    }).expect("You must provide either an expression or filename");
    let parsed = grammar::ExprTParser::new()
        .parse(prog)
        .expect("Unable to parse input");
    let process = |line: std::io::Result<String>| {
        let s = line.unwrap();
        let line = s.trim();
        let gdpr = ConsentString::parse(line);
        match gdpr {
            Some(gdpr)   => if parsed.eval(&gdpr) {
                println!("{}", line);
            },
            None => (),
        }
    };

    args.file 
        .map(|fname| {
            let f = File::open(fname).expect("Unable to open file");
            BufReader::new(f).lines().for_each(process);
        })
        .unwrap_or_else(|| {
            BufReader::new(stdin()).lines().for_each(process);
        });
}
