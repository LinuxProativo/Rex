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
                "-t" | "--target-binary" => {
                    cli.target_binary = Some(Self::expect_path(&mut args, "--target-binary")?)
                }
                "-L" | "--compress-level" => {
                    cli.compression_level =
                        Self::expect_value(&mut args, "--compression-level")?.parse()?
                }
                "-l" | "--extra-libs" => cli
                    .extra_libs
                    .push(Self::expect_path(&mut args, "--extra-libs")?),
                "-b" | "--extra-bins" => cli
                    .extra_bins
                    .push(Self::expect_path(&mut args, "--extra-bins")?),
                "-f" | "--extra-files" => cli
                    .additional_files
                    .push(Self::expect_value(&mut args, "--extra-files")?),
                _ => return Err(Cli::print_help().into()),
            }
        }

        Ok(cli)
    }

    fn expect_value<I: Iterator<Item = String>>(
        args: &mut I,
        name: &str,
    ) -> Result<String, Box<dyn Error>> {
        args.next()
            .ok_or_else(|| format!("missing value for {name}").into())
    }

    fn expect_path<I: Iterator<Item = String>>(
        args: &mut I,
        name: &str,
    ) -> Result<PathBuf, Box<dyn Error>> {
        Ok(PathBuf::from(Self::expect_value(args, name)?))
    }

    fn print_help() -> String {
        format!(
            r#"Rex {VERSION} - Static Rust Executable Generator and Runtime

Usage: rex [OPTIONS]

Options:
  -t, --target-binary <FILE>     Path to the main target binary to bundle
  -L, --compression-level <NUM>  Compression level (1–22, default {DEFAULT_COMPRESS})
  -l, --extra-libs <FILE>        Additional libraries to include
  -b, --extra-bins <FILE>        Additional binaries to include
  -f, --extra-files <PATH>       Extra files or directories to include"#
        )
    }
}

fn rex_main(runtime: &mut Runtime) -> Result<(), Box<dyn Error>> {
    if runtime.is_bundled() {
        return runtime.run()
    }

    let cli = match Cli::parse() {
        Ok(c) => c,
        Err(e) => return Err(e.into()),
    };

    let args = generator::BundleArgs {
        target_binary: cli.target_binary.unwrap_or_default(),
        compression_level: cli.compression_level,
        extra_libs: cli.extra_libs,
        extra_bins: cli.extra_bins,
        additional_files: cli.additional_files,
    };

    generator::generate_bundle(args)
}

fn main() {
    match Runtime::new() {
        Ok(mut runtime) => {
            if let Err(e) = rex_main(&mut runtime) {
                if !runtime.has_run() {
                    eprintln!("{e}");
                }
                exit(1);
            }
        }
        Err(e) => eprintln!("Error creating runtime: {e}"),
    }
}
