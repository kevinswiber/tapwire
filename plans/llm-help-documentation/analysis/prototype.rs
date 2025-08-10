// Prototype for testing Clap introspection capabilities
// This file demonstrates how to extract comprehensive documentation from a Clap CLI

use clap::{Command, Arg, ArgAction, builder::PossibleValue};
use serde_json::json;

fn main() {
    // Build a sample CLI structure similar to Shadowcat
    let cmd = build_cli();
    
    // Test different documentation generation approaches
    println!("=== Testing Clap Introspection ===\n");
    
    // 1. Test basic metadata extraction
    println!("1. Basic Metadata:");
    println!("   Name: {}", cmd.get_name());
    println!("   Version: {:?}", cmd.get_version());
    println!("   About: {:?}", cmd.get_about().map(|s| s.to_string()));
    println!();
    
    // 2. Test traversing subcommands
    println!("2. Subcommands:");
    for subcmd in cmd.get_subcommands() {
        println!("   - {}: {:?}", 
            subcmd.get_name(), 
            subcmd.get_about().map(|s| s.to_string())
        );
    }
    println!();
    
    // 3. Test argument extraction
    println!("3. Global Arguments:");
    for arg in cmd.get_arguments() {
        println!("   - {}: {:?}", 
            arg.get_id(), 
            arg.get_help().map(|s| s.to_string())
        );
        if let Some(short) = arg.get_short() {
            println!("     Short: -{}", short);
        }
        if let Some(long) = arg.get_long() {
            println!("     Long: --{}", long);
        }
        println!("     Required: {}", arg.is_required_set());
    }
    println!();
    
    // 4. Generate JSON documentation
    println!("4. JSON Documentation:");
    let json_doc = generate_json_doc(&cmd);
    println!("{}", serde_json::to_string_pretty(&json_doc).unwrap());
    println!();
    
    // 5. Generate Markdown documentation
    println!("5. Markdown Documentation:");
    let markdown_doc = generate_markdown_doc(&cmd, 0);
    println!("{}", markdown_doc);
}

fn build_cli() -> Command {
    Command::new("shadowcat")
        .version("0.1.0")
        .about("High-performance MCP proxy")
        .long_about("Shadowcat is a high-performance Model Context Protocol (MCP) proxy with recording and interception capabilities")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .help("Set log level")
                .value_name("LEVEL")
                .value_parser(["trace", "debug", "info", "warn", "error"])
                .default_value("info")
        )
        .subcommand(
            Command::new("forward")
                .about("Run forward proxy")
                .long_about("Creates a forward proxy that intercepts MCP traffic between client and server")
                .arg(
                    Arg::new("transport")
                        .help("Transport type to use")
                        .value_name("TRANSPORT")
                        .value_parser(["stdio", "http"])
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Port to bind to")
                        .value_name("PORT")
                        .value_parser(clap::value_parser!(u16))
                        .default_value("8080")
                )
                .arg(
                    Arg::new("command")
                        .help("Command to execute")
                        .value_name("COMMAND")
                        .required(true)
                        .num_args(1..)
                        .last(true)
                )
        )
        .subcommand(
            Command::new("reverse")
                .about("Run reverse proxy")
                .long_about("Creates a reverse proxy with authentication and policy enforcement")
                .arg(
                    Arg::new("bind")
                        .long("bind")
                        .help("Address to bind to")
                        .value_name("ADDRESS")
                        .default_value("127.0.0.1:8080")
                )
                .arg(
                    Arg::new("upstream")
                        .long("upstream")
                        .help("Upstream server URL")
                        .value_name("URL")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("tape")
                .about("Manage recorded tapes")
                .subcommand(
                    Command::new("list")
                        .about("List all tapes")
                )
                .subcommand(
                    Command::new("info")
                        .about("Show tape information")
                        .arg(
                            Arg::new("tape-id")
                                .help("Tape ID to inspect")
                                .required(true)
                                .index(1)
                        )
                )
                .subcommand(
                    Command::new("export")
                        .about("Export tape to file")
                        .arg(
                            Arg::new("tape-id")
                                .help("Tape ID to export")
                                .required(true)
                                .index(1)
                        )
                        .arg(
                            Arg::new("output")
                                .short('o')
                                .long("output")
                                .help("Output file path")
                                .value_name("FILE")
                        )
                )
        )
}

fn generate_json_doc(cmd: &Command) -> serde_json::Value {
    let args: Vec<_> = cmd.get_arguments()
        .map(|arg| {
            let mut arg_doc = json!({
                "name": arg.get_id().to_string(),
                "help": arg.get_help().map(|s| s.to_string()),
                "required": arg.is_required_set(),
            });
            
            if let Some(short) = arg.get_short() {
                arg_doc["short"] = json!(short.to_string());
            }
            if let Some(long) = arg.get_long() {
                arg_doc["long"] = json!(long);
            }
            if let Some(value_name) = arg.get_value_names() {
                arg_doc["value_name"] = json!(value_name[0]);
            }
            
            // Check for default values
            if let Some(defaults) = arg.get_default_values() {
                let default_strs: Vec<String> = defaults
                    .map(|v| v.to_string_lossy().to_string())
                    .collect();
                if !default_strs.is_empty() {
                    arg_doc["default"] = json!(default_strs[0]);
                }
            }
            
            arg_doc
        })
        .collect();
    
    let subcommands: Vec<_> = cmd.get_subcommands()
        .map(|subcmd| generate_json_doc(subcmd))
        .collect();
    
    json!({
        "name": cmd.get_name(),
        "version": cmd.get_version(),
        "about": cmd.get_about().map(|s| s.to_string()),
        "long_about": cmd.get_long_about().map(|s| s.to_string()),
        "usage": cmd.render_usage().to_string(),
        "arguments": args,
        "subcommands": subcommands
    })
}

fn generate_markdown_doc(cmd: &Command, depth: usize) -> String {
    let mut doc = String::new();
    let indent = "#".repeat(depth + 1);
    
    // Command header
    doc.push_str(&format!("{} `{}`", indent, cmd.get_name()));
    if let Some(about) = cmd.get_about() {
        doc.push_str(&format!(" - {}", about));
    }
    doc.push_str("\n\n");
    
    // Long description
    if let Some(long_about) = cmd.get_long_about() {
        doc.push_str(&format!("{}\n\n", long_about));
    }
    
    // Usage
    if depth == 0 || !cmd.get_subcommands().next().is_none() {
        doc.push_str(&format!("**Usage:** `{}`\n\n", cmd.render_usage()));
    }
    
    // Arguments
    let args: Vec<_> = cmd.get_arguments().collect();
    if !args.is_empty() {
        doc.push_str("**Options:**\n");
        for arg in args {
            doc.push_str(&format!("- "));
            
            // Format the option
            let mut opt_parts = Vec::new();
            if let Some(short) = arg.get_short() {
                opt_parts.push(format!("-{}", short));
            }
            if let Some(long) = arg.get_long() {
                opt_parts.push(format!("--{}", long));
            }
            if opt_parts.is_empty() {
                opt_parts.push(format!("<{}>", arg.get_id()));
            }
            
            doc.push_str(&format!("`{}`", opt_parts.join(", ")));
            
            if let Some(value_name) = arg.get_value_names() {
                doc.push_str(&format!(" <{}>", value_name[0]));
            }
            
            if let Some(help) = arg.get_help() {
                doc.push_str(&format!(" - {}", help));
            }
            
            if let Some(defaults) = arg.get_default_values() {
                let default_strs: Vec<String> = defaults
                    .map(|v| v.to_string_lossy().to_string())
                    .collect();
                if !default_strs.is_empty() {
                    doc.push_str(&format!(" (default: {})", default_strs[0]));
                }
            }
            
            doc.push_str("\n");
        }
        doc.push_str("\n");
    }
    
    // Subcommands
    for subcmd in cmd.get_subcommands() {
        doc.push_str(&generate_markdown_doc(subcmd, depth + 1));
    }
    
    doc
}

// Example output structures that would be used in the actual implementation

#[derive(serde::Serialize)]
struct CliDocumentation {
    schema_version: String,
    tool_name: String,
    version: Option<String>,
    description: Option<String>,
    commands: Vec<CommandDoc>,
    global_options: Vec<ArgumentDoc>,
}

#[derive(serde::Serialize)]
struct CommandDoc {
    name: String,
    description: Option<String>,
    long_description: Option<String>,
    usage: String,
    arguments: Vec<ArgumentDoc>,
    subcommands: Vec<CommandDoc>,
    examples: Vec<Example>,
}

#[derive(serde::Serialize)]
struct ArgumentDoc {
    name: String,
    short: Option<char>,
    long: Option<String>,
    description: Option<String>,
    value_name: Option<String>,
    required: bool,
    multiple: bool,
    default_value: Option<String>,
    possible_values: Option<Vec<String>>,
    arg_type: String,
}

#[derive(serde::Serialize)]
struct Example {
    description: String,
    command: String,
    output: Option<String>,
}