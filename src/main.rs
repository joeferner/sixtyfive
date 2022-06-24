use clap::{Parser, Subcommand};
use std::{fmt::Debug, path::PathBuf, process};

mod disassemble;
mod linker_file;
use disassemble::{disassemble, DisassembleOptions};

#[derive(Debug, Parser)]
#[clap(name = "sixtyfive")]
#[clap(about = "A 6502 disassembler/assembler", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(arg_required_else_help = true, about = "disassemble a binary")]
    D {
        #[clap(
            short = 'l',
            long = "link",
            required = true,
            value_parser,
            help = "the linker file (built in: nes)"
        )]
        linker_file: String,

        #[clap(
            short = 'o',
            long = "out",
            value_parser,
            help = "output file otherwise stdout"
        )]
        out: Option<PathBuf>,

        #[clap(value_parser, help = "path to binary to disassemble otherwise stdin")]
        in_file: Option<PathBuf>,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::D {
            linker_file,
            in_file,
            out,
        } => {
            if let Result::Err(err) = disassemble(DisassembleOptions {
                linker_file,
                in_file,
                out,
            }) {
                eprintln!("Error disassembling: {}", err);
                process::exit(1);
            }
        }
    }
}
