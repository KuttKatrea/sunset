use clap::{ArgAction, Parser, Subcommand};

use sunset::shimmer;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Shim {
        #[clap(long, action=ArgAction::SetTrue)]
        win: Option<bool>,

        #[clap(value_parser)]
        path: Option<String>,

        #[clap(value_parser)]
        name: Option<String>,
    },

    Path {
        #[clap(value_parser)]
        name: Option<String>,
    },

    Rm {
        #[clap(value_parser)]
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Shim {
            win,
            path: target_path,
            name,
        } => shimmer::shim(target_path, name, win),
        Commands::Path { name } => shimmer::shim_path(name),
        Commands::Rm { name } => shimmer::shim_remove(name),
    };
}
