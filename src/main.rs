use core::fmt;
use std::{io::{Read, BufReader, self, Write}, fs::File, process, str::FromStr, borrow::Cow, fmt::Display};
use toml::Value;
use shell_escape::escape as sh_escape;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Opts {
    #[clap(short = 'e', long = "export")]
    exports: Vec<String>,
    #[clap(short = 's', long = "parse")]
    parse_as: Vec<String>,
    input: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Invalid export specification")]
    ExportSpecError,
    #[error("No such key: {key}")]
    NoSuchKey { key: String },
    #[error("IOError: {source}")]
    IOError { #[from] source: io::Error },
}

struct ExportSpec {
    as_var: String,
    path: Vec<String>
}

impl FromStr for ExportSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (as_var, path_s) = s.split_once('=').ok_or(Error::ExportSpecError)?;
        let path = path_s.split(".").map(|s| s.to_string()).collect();
        let as_var = as_var.to_string();
        Ok(ExportSpec { as_var, path })
    }
}

fn get_path<'a, S>(mut obj: &'a Value, path: &[S]) -> Result<&'a Value, Error>
    where S: AsRef<str>
{
    for part in path.iter() {
        obj = obj.get(part.as_ref()).ok_or_else(|| {
            Error::NoSuchKey {
                key: path.iter().map(|s| s.as_ref()).collect::<Vec<_>>().join(".")
            }
        })?;
    }
    Ok(obj)
}

fn is_atomic(obj: &Value) -> bool {
    use Value::*;
    matches!(obj, String(_) | Boolean(_) | Integer(_) | Datetime(_) | Float(_))
}

fn write_atom(f: &mut fmt::Formatter<'_>, obj: &Value) -> fmt::Result {
    match obj {
        Value::String(s) => write!(f, r#""{}""#, sh_escape(Cow::from(s)))?,
        Value::Integer(i) => write!(f, "{i}")?,
        Value::Boolean(b) => write!(f, "{b}")?,
        Value::Float(fl) => write!(f, "{fl}")?,
        Value::Datetime(dt) => write!(f, "{dt}")?,
        _ => unreachable!(),
    }
    Ok(())
}

struct FmtBash<'a> {
    as_var: String,
    value: &'a toml::Value
}

impl<'a> Display for FmtBash<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.as_var;
        match self.value {
            Value::Array(arr) => {
                write!(f, "declare -a {name}=(")?;
                let mut had_one = false;
                for elem in arr.iter() {
                    if !is_atomic(elem) { continue }

                    if had_one {
                        write!(f, " ")?;
                    } else {
                        had_one = true;
                    }

                    write_atom(f, elem)?;
                }
                writeln!(f, ")")?;
            }
            Value::Table(tbl) => {
                write!(f, "declare -A {name}=(")?;
                let mut had_one = false;
                for (key, value) in tbl.iter() {
                    if !is_atomic(value) { continue }

                    if had_one {
                        write!(f, " ")?;
                    } else {
                        had_one = true;
                    }

                    write!(f, r#"["{}"]="#, sh_escape(Cow::from(key)))?;
                    write_atom(f, value)?;
                }
                writeln!(f, ")")?;
            }
            Value::Integer(i) => writeln!(f, r#"declare -i {name}={i}"#)?,
            x => {
                write!(f, r#"declare {name}="#)?;
                write_atom(f, x)?;
            }
        }
        Ok(())
    }
}

fn doit(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let mut input_file = BufReader::new(
        opts.input.map(|f| -> Result<Box<dyn Read>, io::Error> {
            Ok(Box::new(File::open(f)?))
        }).unwrap_or_else(|| Ok(Box::new(io::stdin())))?);
    let mut input = String::new();
    input_file.read_to_string(&mut input)?;
    let obj: toml::Value = toml::from_str(&input)?;
    for export in opts.exports.iter() {
        let ExportSpec { as_var, path } = export.parse()?;
        let value = get_path(&obj, &path)?;
        print!("{}", FmtBash { as_var, value })
    }
    Ok(())
}

fn main() {
    let opts: Opts = Opts::parse();
    if let Err(e) = doit(opts) {
        eprintln!("{e}");
        process::exit(1);
    }
}
