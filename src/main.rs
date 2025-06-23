mod config;
mod explorer;
mod module_info;
mod npm_client;
mod output_format;
mod parser;
mod tree_formatter;
mod utils;

use clap::{Parser, Subcommand};
use std::process;
use tokio::signal;

#[derive(Parser)]
#[command(name = "pretty-node")]
#[command(about = "A Node.js package tree explorer for LLMs (and humans)")]
#[command(version)]
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

    // Set up Ctrl+C handler
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let main_task = async {
        match cli.command {
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
        }
    };

    // Run main task with Ctrl+C handling
    tokio::select! {
        result = main_task => {
            if let Err(e) = result {
                if !e.to_string().is_empty() {
                    eprintln!("Error: {}", e);
                }
                process::exit(1);
            }
        }
        _ = ctrl_c => {
            eprintln!("\nReceived interrupt signal, exiting...");
            process::exit(130);
        }
    }
}

async fn handle_tree_command(
    package: &str,
    depth: usize,
    quiet: bool,
    output: OutputFormat,
) -> anyhow::Result<()> {
    use crate::explorer::NodeModuleExplorer;
    use crate::output_format::create_formatter;

    // Validate that package doesn't contain colon (which would be for sig command)
    if package.contains(':') {
        return Err(anyhow::anyhow!(
            "Invalid module path '{}' for tree command. Module paths with ':' syntax are for signatures. Use 'pretty-node sig {}' instead.",
            package, package
        ));
    }

    let format_str = match output {
        OutputFormat::Pretty => "pretty",
        OutputFormat::Json => "json",
    };
    let formatter = create_formatter(format_str);

    let explorer = NodeModuleExplorer::new(package.to_string(), depth, quiet);
    let tree_result = explorer.explore().await;

    match tree_result {
        Ok(tree) => {
            let output = formatter.format_tree(&tree)?;
            println!("{}", output);
        }
        Err(_) => {
            // Gracefully handle package not found
            if !quiet {
                eprintln!("⚠️  Package '{}' not found or could not be explored", package);
            }
            // Still show a basic tree structure
            let fallback_tree = module_info::NodeModuleInfo::new(package.to_string());
            let output = formatter.format_tree(&fallback_tree)?;
            println!("{}", output);
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
    use crate::output_format::create_formatter;

    let format_str = match output {
        OutputFormat::Pretty => "pretty",
        OutputFormat::Json => "json",
    };
    let formatter = create_formatter(format_str);

    let signature_result = extract_signature(import_path, quiet).await;
    
    match signature_result {
        Ok(signature) => {
            let output = formatter.format_signature(&signature)?;
            println!("{}", output);
        }
        Err(_) => {
            // Gracefully handle signature extraction failures
            let symbol_name = import_path.split(':').last().unwrap_or("unknown");
            let output = formatter.format_signature_not_available(symbol_name);
            println!("{}", output);
        }
    }

    Ok(())
}
