use crate::runtime::Runtime;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;

mod generator;
mod runtime;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_COMPRESS: i32 = 5;

struct Cli {
    target_binary: Option<PathBuf>,
    compression_level: i32,
    extra_libs: Vec<PathBuf>,
    extra_bins: Vec<PathBuf>,
    additional_files: Vec<String>,
}

impl Cli {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut args = env::args().skip(1);
        if args.len() == 0 {
            return Err(Cli::print_help().into());
        }

        let mut cli = Self {
            target_binary: None,
            compression_level: DEFAULT_COMPRESS,
            extra_libs: vec![],
            extra_bins: vec![],
            additional_files: vec![],
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-t" => cli.target_binary = Some(Self::expect_path(&mut args)?),
                "-L" => cli.compression_level = Self::expect_value(&mut args)?.parse()?,
                "-l" => cli.extra_libs.push(Self::expect_path(&mut args)?),
                "-b" => cli.extra_bins.push(Self::expect_path(&mut args)?),
                "-f" => cli.additional_files.push(Self::expect_value(&mut args)?),
                _ => return Err(Cli::print_help().into()),
            }
        }

        Ok(cli)
    }

    fn expect_value(args: &mut impl Iterator<Item = String>) -> Result<String, Box<dyn Error>> {
        args.next().ok_or("Missing value".into())
    }

    fn expect_path(args: &mut impl Iterator<Item = String>) -> Result<PathBuf, Box<dyn Error>> {
        Ok(PathBuf::from(Self::expect_value(args)?))
    }

    fn print_help() -> String {
        format!(
            "Rex {VERSION} - static Rust EXecutable generator and runtime\n
Usage: rex <options>\n
Options:
  -t <file>  Path to the main target binary to bundle
  -L <num>   Compression level (1–22, default {DEFAULT_COMPRESS})
  -l <file>  Additional libraries to include
  -b <file>  Additional binaries to include
  -f <path>  Extra files or folders to include"
        )
    }
}

fn rex_main(runtime: &mut Runtime) -> Result<(), Box<dyn Error>> {
    if runtime.is_bundled() {
        return runtime.run();
    }

    let cli = Cli::parse()?;

    let args = generator::BundleArgs {
        target_binary: cli.target_binary.ok_or("Error: -t <file> is required")?,
        compression_level: cli.compression_level,
        extra_libs: cli.extra_libs,
        extra_bins: cli.extra_bins,
        additional_files: cli.additional_files,
    };

    generator::generate_bundle(args)
}

fn main() {
    let i = match Runtime::new() {
        Ok(mut runtime) => match rex_main(&mut runtime) {
            Ok(_) => 0,
            Err(e) => {
                if !runtime.has_run() {
                    eprintln!("{e}");
                }
                1
            }
        },
        Err(e) => {
            eprintln!("Error: {e}");
            1
        }
    };
    exit(i);
}
