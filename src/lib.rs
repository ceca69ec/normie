//! **Just another filename normalizer tool.**
//!
//! Recursively normalize directories and filenames to Unix friendly standard.
//!
//! No dependencies, really simple and fast.
//!
//! ## Example
//!
//! ```console
//! $ touch "B)E(T%T@E*R T*H*I&S W@A*Y#" "G)O(O%@D N*A*M&E@**#"
//! $ ls
//! 'B)E(T%T@E*R T*H*I&S W@A*Y#'  'G)O(O%@D N*A*M&E@**#'
//! $ normie -lra .tgz *
//! $ ls
//! better_this_way.tgz  good_name.tgz
//! ```
//!
//! ## Help
//!
//! ```shell
//! normie 1.0.2
//!
//! USAGE:
//!     normie [FLAG]... DIRECTORY_OR_FILE...
//!
//! FLAGS:
//!     -a: Append the specified text at the end of the filename.
//!     -h: Show this help information.
//!     -i: Insert the specified text at the beginning of the filename.
//!     -l: Transform the resulting filename into all lowercase characters.
//!     -r: Remove these characters: '!"#$%&'()*+,/:;<=>?@[\]^`{|}~ªº'.
//!     -t: Interactively asks for confirmation of each action.
//!     -u: Transform the resulting filename into all uppercase characters.
//!     -v: Show information about the performed actions.
//! ```
//!
//! ## Installation
//!
//! You have to install [rust](https://www.rust-lang.org/tools/install) and a
//!  [linker](https://gcc.gnu.org/wiki/InstallingGCC) if you don't already have them.
//!
//! ```shell
//! $ cargo install normie
//! ```
//!
//! ## Warning
//!
//! Use flag `-t` if you are insecure of the results.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Help to use the binary.
#[doc(hidden)]
pub const USAGE: &str = "[FLAG]... DIRECTORY_OR_FILE...

FLAGS:
    -a: Append the specified text at the end of the filename.
    -h: Show this help information.
    -i: Insert the specified text at the beginning of the filename.
    -l: Transform the resulting filename into all lowercase characters.
    -r: Remove these characters: '!\"#$%&\'()*+,/:;<=>?@[\\]^`{|}~ªº'.
    -t: Interactively asks for confirmation of each action.
    -u: Transform the resulting filename into all uppercase characters.
    -v: Show information about the performed actions";

/// Especial characters to be removed with option 'r'.
const SPECIAL: &str = "!\"#$%&\'()*+,/:;<=>?@[\\]^`{|}~ªº"; // exclude ._-

/// Valid characters used as parameter flags.
const FLAGS: [char; 8] = ['a', 'h', 'i', 'l', 'r', 't', 'u', 'v'];

/// Structure to organize arguments.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd)]
#[doc(hidden)]
pub struct Parsed {
    pub app: String,      // append this to the file name (start)
    pub ins: String,      // insert this to the file name (end)
    pub me: String,       // 'name' of the executed binary
    pub flg: Vec<char>,   // list of argument flags
    pub pos: Vec<String>, // list of positional arguments
}

/// Implementation for the structure Args.
impl Parsed {
    /// Instantiate a new Args.
    fn new() -> Self {
        Self {
            app: String::new(),
            ins: String::new(),
            me: String::new(),
            flg: Vec::new(),
            pos: Vec::new(),
        }
    }
}

/// Organize and validate the arguments.
#[doc(hidden)]
pub fn arg_analyzer(mut args: env::Args) -> Result<Parsed, String> {
    if args.len() <= 1 {
        return Err(String::from("missing file operand"));
    }
    let mut out = Parsed::new();
    out.me = args.next().unwrap_or_default();
    for arg in args {
        if let Some(stripped) = arg.strip_prefix('-') {
            out.flg.append(&mut stripped.chars().collect());
        } else if (out.flg.contains(&'a') || out.flg.contains(&'i')) && !Path::new(&arg).exists() {
            if out.flg.contains(&'a') && out.app.is_empty() {
                out.app = arg;
            } else if out.flg.contains(&'i') && out.ins.is_empty() {
                out.ins = arg;
            }
        } else {
            out.pos.push(arg);
        }
    }
    if out.flg.contains(&'h') {
        return Ok(out);
    }
    if out.pos.is_empty() {
        return Err(String::from("missing file operand"));
    }
    if out.flg.contains(&'l') && out.flg.contains(&'u') {
        return Err(String::from("options 'l' and 'u' not allowed at same time"));
    }
    if out.flg.contains(&'a') && out.app.is_empty() {
        return Err(String::from("missing text to append"));
    }
    if out.flg.contains(&'i') && out.ins.is_empty() {
        return Err(String::from("missing text to insert"));
    }
    for c in &out.flg {
        if !FLAGS.contains(c) {
            return Err(format!("invalid option -- '{}'", c));
        }
    }
    Ok(out)
}

/// Modify a string according to the options. Return a String.
fn mod_str(text: &str, args: &Parsed) -> String {
    let mut out = text.replace(|x| x == '\u{20}' || x == '\u{3000}', "_"); // common or ideographic
    if args.flg.contains(&'a') && !args.app.is_empty() {
        out.push_str(&args.app);
    }
    if args.flg.contains(&'i') && !args.ins.is_empty() {
        out.insert_str(0, &args.ins);
    }
    if args.flg.contains(&'r') {
        for c in SPECIAL.chars() {
            out = out.replace(c, "");
        }
    }
    if args.flg.contains(&'l') {
        out = out.to_lowercase();
    } else if args.flg.contains(&'u') {
        out = out.to_uppercase();
    }
    out
}

/// Asks the user about renaming or not (option 't').
fn interactive(me: &str, old: &str, new: &str) -> Result<(), io::Error> {
    print!("\x1b[1m{}\x1b[0m: rename '{}' to '{}'? ", me, old, new);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim()[..1].to_lowercase() != "y" {
        return Err(io::Error::new(io::ErrorKind::Other, "user said 'no'"));
    }
    Ok(())
}

/// Rename a single directory/file 'p' using options 'args'.
fn rename(p: &str, args: &Parsed) -> Result<(), String> {
    if Path::new(p).exists() {
        let path = Path::new(p);
        let name = match path.file_name() {
            Some(n) => n.to_str().unwrap_or_default(),
            None => {
                return Err(format!(
                    "\x1b[1m{}\x1b[0m: {} has no valid name\n",
                    args.me, p
                ))
            }
        };
        let target = mod_str(name, args);
        if name == target {
            return Err(format!(
                "\x1b[1m{}\x1b[0m: nothing to do with '{}'\n",
                args.me, p
            ));
        }
        let res = format!("{}{}", p.strip_suffix(name).unwrap_or_default(), target);
        if args.flg.contains(&'t') && interactive(&args.me, p, &res).is_err() {
            return Ok(());
        }
        match fs::rename(&path, &res[..]) {
            Ok(_) => {
                if args.flg.contains(&'v') {
                    println!("renamed '{}' to '{}'.", p, res);
                }
            }
            Err(e) => {
                return Err(format!(
                    "\x1b[1m{}\x1b[0m: cannot rename '{}' to '{}': {}.\n",
                    args.me,
                    p,
                    res,
                    e.to_string().split_once(" (").unwrap_or_default().0
                ))
            }
        };
    } else {
        return Err(format!(
            "\x1b[1m{}\x1b[0m: '{}' is not a valid directory/file.\n",
            args.me, p
        ));
    }
    Ok(())
}

/// Start the program according to parameters.
#[doc(hidden)]
pub fn run(args: Parsed) -> Result<(), String> {
    let mut e = String::new();
    for path in &args.pos {
        if let Err(errs) = rename(path, &args) {
            e.push_str(&errs);
        }
    }
    if !e.is_empty() {
        return Err(format!(
            "{}\x1b[1m{}\x1b[0m: \x1b[1;31merror\x1b[0m, some actions could not be performed",
            e, args.me
        ));
    }
    Ok(())
}

/// Tests for the library.
#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_mod_str() {
        let mut args = Parsed::new();
        args.flg = vec!['u'];
        assert_eq!(mod_str("upper-case", &args), "UPPER-CASE");
        args.flg = vec!['u', 'a'];
        args.app = String::from("-case");
        assert_eq!(mod_str("upper", &args), "UPPER-CASE");
        args.flg = vec!['u', 'i', 'r'];
        args.app = String::new();
        args.ins = String::from("upper");
        assert_eq!(mod_str("-case", &args), "UPPER-CASE");
        args.flg = vec!['l'];
        assert_eq!(mod_str("Ho lA.LaY", &args), "ho_la.lay");
        args.flg = vec!['r', 'u'];
        assert_eq!(mod_str("Ho lA.LaY", &args), "HO_LA.LAY");
        assert_eq!(mod_str("u'e&pª\".lay", &args), "UEP.LAY");
        assert_eq!(mod_str("バンドメイド", &args), "バンドメイド");
        assert_eq!(mod_str("ぽ\u{3000}", &args), "ぽ_");
    }

    #[test]
    fn test_rename() {
        let p = "/tmp/=>SUCKS<=";
        let mut args = Parsed::new();
        fs::File::create(p).unwrap();
        args.flg = vec!['i', 'l', 'r'];
        args.ins = String::from("THIS-");
        assert!(rename(p, &args).is_ok());
        fs::remove_file("/tmp/this-sucks").unwrap();

        let p = "/tmp/=>IS<=";
        let mut args = Parsed::new();
        fs::File::create(&p).unwrap();
        args.flg = vec!['a', 'l', 'r'];
        args.app = String::from("-GOOD");
        assert!(rename(&p, &args).is_ok());
        fs::remove_file("/tmp/is-good").unwrap();

        let p = "/tmp/B)E(T%T@E*R T*H*I&S W@A*Y#";
        let mut args = Parsed::new();
        fs::File::create(&p).unwrap();
        args.flg = vec!['l', 'r', 'a'];
        args.app = String::from(".tgz");
        assert!(rename(&p, &args).is_ok());
        fs::remove_file("/tmp/better_this_way.tgz").unwrap();

        let p = "/tmp/G)O(O%@D N*A*M&E@**#";
        let mut args = Parsed::new();
        fs::File::create(&p).unwrap();
        args.flg = vec!['l', 'r', 'a'];
        args.app = String::from(".tgz");
        assert!(rename(&p, &args).is_ok());
        fs::remove_file("/tmp/good_name.tgz").unwrap();
    }
}
