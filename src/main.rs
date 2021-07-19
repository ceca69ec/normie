//! Remove spaces from filenames (some additional renaming options available).

use std::env;

use normie::{arg_analyzer, run, USAGE};

/// Whirlpool of the binary.
fn main() {
    let me = env::args().next().unwrap();

    let args = arg_analyzer(env::args()).unwrap_or_else(|err| {
        eprintln!(
            "{}: {}.\nTry '{} -h' for more information.", me, err, me
        );
        std::process::exit(96);
    });

    if args.flg.contains(&'h') {
        println!("normie 0.1.0\n\nUSAGE:\n\t{} {}.", me, USAGE);
    } else {
        run(args).unwrap_or_else(|err| {
            eprintln!("{}.", err);
            std::process::exit(96);
        });
    }
}
