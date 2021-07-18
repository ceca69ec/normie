//! **Rename directories and files into Unix friendly standard.**

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Help to use the binary.
#[doc(hidden)]
pub const USAGE: &str =
"[FLAG]... DIRECTORY_OR_FILE...

FLAGS:
\t-a: append the specified text at the end of the filename.
\t-i: insert the specified text at the beginning of the filename.
\t-l: transform the resulting filename into all lowercase characters.
\t-r: remove these characters: '!\"#$%&\'()*+,/:;<=>?@[\\]^`{|}~ªº'.
\t-t: interactively asks for confirmation of each action.
\t-u: transform the resulting filename into all uppercase characters.
\t-v: show information about the actions performed";

/// Especial characters to be removed with option 'r'.
const SPECIAL: &str = "!\"#$%&\'()*+,/:;<=>?@[\\]^`{|}~ªº"; // Exclude ._-

/// Valid characters used as optional parameters.
const OPT: [char; 7] = ['a', 'i', 'l', 'r', 't', 'u', 'v'];

/// Valid strings used as long optional parameters.
const LONG: [&str; 1] = ["help"];

/// Structure to organize arguments.
#[doc(hidden)]
pub struct Parsed {
    pub app: String,
    pub ins: String,
    pub lop: Vec<String>,
    pub me: String,
    pub opt: Vec<char>,
    pub pos: Vec<String>
}

/// Sugested by 'cargo clippy'
impl Default for Parsed {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation for the structure Args.
impl Parsed {
    /// Instantiate a new Args.
    fn new() -> Self {
        Self {
            app: String::new(),
            ins: String::new(),
            lop: Vec::new(),
            me: String::new(),
            opt: Vec::new(),
            pos: Vec::new()
        }
    }
}

/// Organize and validate the arguments.
#[doc(hidden)]
pub fn arg_analyzer(mut args: env::Args) -> Result<Parsed, String> {
    if args.len() <= 1 {
        return Err("missing file operand".to_string());
    }
    let mut out = Parsed::new();
    out.me = args.next().unwrap();
    for arg in args {
        if let Some(stripped) = arg.strip_prefix("--") {
            out.lop.push(stripped.to_string());
        } else if let Some(stripped) = arg.strip_prefix("--") {
            out.opt.append(&mut stripped.chars().collect());
        } else if (out.opt.contains(&'a') || out.opt.contains(&'i')) && !Path::new(&arg).exists() {
            if out.opt.contains(&'a') && out.app.is_empty() {
                out.app = arg;
            } else if out.opt.contains(&'i') && out.ins.is_empty() {
                out.ins = arg;
            }
        } else {
            out.pos.push(arg);
        }
    }
    if out.lop.contains(&"help".to_string()) { return Ok(out) }
    if out.pos.is_empty() { return Err("missing file operand".to_string()) }
    if out.opt.contains(&'l') && out.opt.contains(&'u') {
        return Err("options 'l' and 'u' not allowed at same time".to_string())
    }
    if out.opt.contains(&'a') && out.app.is_empty() {
        return Err("missing text to append".to_string())
    }
    if out.opt.contains(&'i') && out.ins.is_empty() {
        return Err("missing text to insert".to_string())
    }
    for c in &out.opt {
        if !OPT.contains(c) {
            return Err(format!("invalid option -- '{}'", c))
        }
    }
    for long_opt in &out.lop {
        if !LONG.contains(&long_opt.as_str()) {
            return Err(format!("unrecognized option '--{}'", long_opt))
        }
    }
    Ok(out)
}

/// Modify a string according to the options. Return a String.
fn mod_str(text: &str, args: &Parsed) -> String {
    let mut out = text.replace(' ', "_");
    if args.opt.contains(&'a') && !args.app.is_empty() { out.push_str(&args.app); }
    if args.opt.contains(&'i') && !args.ins.is_empty() { out.insert_str(0, &args.ins); }
    if args.opt.contains(&'r') {
        for c in SPECIAL.chars() {
            out = out.replace(c, "");
        }
    }
    if args.opt.contains(&'l') {
        out = out.to_lowercase();
    } else if args.opt.contains(&'u') {
        out = out.to_uppercase();
    }
    out
}

/// Asks the user about renaming or not (option 't').
fn interactive(me: &str, old: &str, new: &str) -> Result<(), io::Error> {
    print!("{}: rename '{}' to '{}'? ", me, old, new);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim()[..1].to_lowercase() != "y" {
        return Err(io::Error::new(io::ErrorKind::Other, "user said 'no'"))
    }
    Ok(())
}

/// Rename a single directory/file 'p' using options 'args'.
fn rename(p: &str, args: &Parsed) -> Result<(), String> {
    if Path::new(p).exists() {
        let path = Path::new(p);
        let name = match path.file_name() {
            Some(n) => n.to_str().unwrap(),
            None => return Err(format!("{}: {} has no valid name\n", args.me, p))
        };
        let target = mod_str(name, args);
        if name == target {
            return Err(format!("{}: nothing to do with '{}'\n", args.me, p))
        }
        let res = format!("{}{}", p.strip_suffix(name).unwrap(), target);
        if args.opt.contains(&'t') && interactive(&args.me, p, &res).is_err() {
            return Ok(()) 
        }
        match fs::rename(&path, &res[..]) {
            Ok(_) => {
                if args.opt.contains(&'v') {
                    println!("renamed '{}' to '{}'.", p, res);
                }
            },
            Err(e) => return Err(format!(
                "{}: cannot rename '{}' to '{}': {}.\n", args.me, p, res,
                e.to_string().split(" (").next().unwrap()
            ))
        };
    } else {
        return Err(
            format!("{}: '{}' is not a valid directory/file.\n", args.me, p)
        )
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
            "{}{} error: some actions could not be performed", e,args.me
        ))
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
        args.opt = vec!['u'];
        assert_eq!("UPPER-CASE", mod_str("upper-case", &args));
        args.opt = vec!['u', 'a'];
        args.app = "-case".to_string();
        assert_eq!("UPPER-CASE", mod_str("upper", &args));
        args.opt = vec!['u', 'i', 'r'];
        args.app = String::new();
        args.ins = "upper".to_string();
        assert_eq!("UPPER-CASE", mod_str("-case", &args));
        args.opt = vec!['l'];
        assert_eq!("ho_la.lay", mod_str("Ho lA.LaY", &args));
        args.opt = vec!['r', 'u'];
        assert_eq!("HO_LA.LAY", mod_str("Ho lA.LaY", &args));
        assert_eq!("UEP.LAY", mod_str("u'e&pª\".lay", &args));
    }

    #[test]
    fn test_rename() {
        let p = "/tmp/=>SUCKS<=";
        let mut args = Parsed::new();
        fs::File::create(p).unwrap();
        args.opt = vec!['i', 'l', 'r'];
        args.ins = String::from("THIS-");
        assert!(rename(p, &args).is_ok());
        fs::remove_file("/tmp/this-sucks").unwrap();

        let p = "/tmp/=>IS<=".to_string();
        let mut args = Parsed::new();
        fs::File::create(&p).unwrap();
        args.opt = vec!['a', 'l', 'r'];
        args.app = String::from("-GOOD");
        assert!(rename(&p, &args).is_ok());
        fs::remove_file("/tmp/is-good").unwrap()
    }
}
