mod config;
mod explorer;
mod module_info;
mod npm_client;
mod parser;
mod tree_formatter;
mod utils;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "pretty-node")]
#[command(about = "A Node.js package tree explorer for LLMs (and humans)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display Node.js package tree structure
    Tree {
        /// Package name (e.g., 'express', '@types/node')
        package: String,
        /// Maximum depth to explore
        #[arg(long, default_value_t = 2)]
        depth: usize,
        /// Suppress warnings and informational messages
        #[arg(short, long)]
        quiet: bool,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Pretty)]
        output: OutputFormat,
    },
    /// Display function/class signature
    Sig {
        /// Import path to the function (e.g., 'express:Router')
        import_path: String,
        /// Suppress download messages
        #[arg(short, long)]
        quiet: bool,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Pretty)]
        output: OutputFormat,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Pretty,
    Json,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Tree {
            package,
            depth,
            quiet,
            output,
        } => handle_tree_command(&package, depth, quiet, output).await,
        Commands::Sig {
            import_path,
            quiet,
            output,
        } => handle_sig_command(&import_path, quiet, output).await,
    };

    if let Err(e) = result {
        if !e.to_string().is_empty() {
            eprintln!("Error: {}", e);
        }
        process::exit(1);
    }
}

async fn handle_tree_command(
    package: &str,
    depth: usize,
    quiet: bool,
    output: OutputFormat,
) -> anyhow::Result<()> {
    use crate::explorer::NodeModuleExplorer;

    let explorer = NodeModuleExplorer::new(package.to_string(), depth, quiet);
    let tree = explorer.explore().await?;

    match output {
        OutputFormat::Pretty => {
            let formatter = tree_formatter::TreeFormatter::new();
            println!("{}", formatter.format_tree(&tree));
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&tree)?);
        }
    }

    Ok(())
}

async fn handle_sig_command(
    import_path: &str,
    quiet: bool,
    output: OutputFormat,
) -> anyhow::Result<()> {
    use crate::parser::signature::extract_signature;

    let signature = extract_signature(import_path, quiet).await?;

    match output {
        OutputFormat::Pretty => {
            let formatter = tree_formatter::TreeFormatter::new();
            println!("{}", formatter.format_signature(&signature));
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&signature)?);
        }
    }

    Ok(())
}
