# Task 006: CLI Tape Management

**File:** `src/cli/tape.rs`  
**Estimated Effort:** 1.5 days  
**Priority:** High  
**Dependencies:** TapeRecorder, TapePlayer, Enhanced Tape Format

---

## Overview

Implement comprehensive CLI commands for managing recorded MCP session tapes, providing developers with powerful tools for inspection, replay, and maintenance of their tape collections.

---

## Requirements

### Core Commands
1. **list**: Display all available tapes with filtering and formatting options
2. **show**: Display detailed information about a specific tape
3. **replay**: Play back a recorded tape with various options
4. **delete**: Remove tapes with confirmation
5. **export**: Convert tapes to other formats (HAR, Postman, cURL)
6. **validate**: Check tape integrity and format compliance
7. **compress**: Reduce tape file size using compression

### User Experience Features
1. **Rich Formatting**: Colored tables, progress bars, and clear output
2. **Interactive Confirmations**: Safe destructive operations
3. **Machine-Readable Output**: JSON and CSV formats for automation
4. **Comprehensive Help**: Detailed usage examples and descriptions
5. **Auto-completion**: Shell completion support (future)

---

## Technical Specification

### Command Structure
```bash
shadowcat tape <subcommand> [options] [arguments]

# Subcommands:
shadowcat tape list [--format=table|json|csv] [--filter=<criteria>]
shadowcat tape show <tape-id> [--frames] [--stats] [--format=table|json]
shadowcat tape replay <tape-id> [--speed=1.0] [--from=<pos>] [--to=<pos>] [--step]
shadowcat tape delete <tape-id> [--confirm] [--force]
shadowcat tape export <tape-id> [--format=har|postman|curl] [--output=<file>]
shadowcat tape validate <tape-id> [--fix] [--verbose]
shadowcat tape compress <tape-id> [--level=6] [--keep-original]
```

### CLI Module Structure
```rust
// src/cli/tape.rs
use clap::{Args, Subcommand};
use crate::recorder::{TapeRecorder, TapePlayer, Tape, TapeId};
use crate::error::Result;

#[derive(Args)]
pub struct TapeCommand {
    #[command(subcommand)]
    pub command: TapeSubcommand,
}

#[derive(Subcommand)]
pub enum TapeSubcommand {
    List(ListArgs),
    Show(ShowArgs),
    Replay(ReplayArgs),
    Delete(DeleteArgs),
    Export(ExportArgs),
    Validate(ValidateArgs),
    Compress(CompressArgs),
}

impl TapeCommand {
    pub async fn execute(&self, recorder: &TapeRecorder) -> Result<()> {
        match &self.command {
            TapeSubcommand::List(args) => self.list_tapes(args, recorder).await,
            TapeSubcommand::Show(args) => self.show_tape(args, recorder).await,
            TapeSubcommand::Replay(args) => self.replay_tape(args, recorder).await,
            TapeSubcommand::Delete(args) => self.delete_tape(args, recorder).await,
            TapeSubcommand::Export(args) => self.export_tape(args, recorder).await,
            TapeSubcommand::Validate(args) => self.validate_tape(args, recorder).await,
            TapeSubcommand::Compress(args) => self.compress_tape(args, recorder).await,
        }
    }
}
```

---

## Implementation Plan

### Day 1: Core Commands (list, show, delete)

#### Morning: Project Setup & List Command
```rust
// 1. Create CLI module structure
#[derive(Args)]
pub struct ListArgs {
    /// Output format
    #[arg(short, long, default_value = "table")]
    format: OutputFormat,
    
    /// Filter tapes by criteria
    #[arg(long)]
    filter: Option<String>,
    
    /// Sort by field
    #[arg(long, default_value = "created")]
    sort: SortField,
    
    /// Reverse sort order
    #[arg(long)]
    reverse: bool,
    
    /// Limit number of results
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

// 2. Implement list command with rich formatting
impl TapeCommand {
    async fn list_tapes(&self, args: &ListArgs, recorder: &TapeRecorder) -> Result<()> {
        let tapes = recorder.list_tapes().await?;
        let filtered_tapes = self.apply_filters(&tapes, &args.filter);
        let sorted_tapes = self.apply_sorting(filtered_tapes, &args.sort, args.reverse);
        
        match args.format {
            OutputFormat::Table => self.display_table(sorted_tapes),
            OutputFormat::Json => self.display_json(sorted_tapes),
            OutputFormat::Csv => self.display_csv(sorted_tapes),
        }
    }
    
    fn display_table(&self, tapes: Vec<TapeMetadata>) -> Result<()> {
        use comfy_table::{Table, Cell, Color, Attribute};
        
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("ID").add_attribute(Attribute::Bold),
            Cell::new("Name").add_attribute(Attribute::Bold),
            Cell::new("Transport").add_attribute(Attribute::Bold),
            Cell::new("Frames").add_attribute(Attribute::Bold),
            Cell::new("Duration").add_attribute(Attribute::Bold),
            Cell::new("Size").add_attribute(Attribute::Bold),
            Cell::new("Created").add_attribute(Attribute::Bold),
        ]);
        
        for tape in tapes {
            table.add_row(vec![
                Cell::new(&tape.id.to_string()[..8]).fg(Color::Cyan),
                Cell::new(&tape.name),
                Cell::new(&format!("{:?}", tape.transport_type)).fg(Color::Green),
                Cell::new(&tape.frame_count.to_string()),
                Cell::new(&format_duration(tape.duration_ms)),
                Cell::new(&format_bytes(tape.total_bytes)).fg(Color::Yellow),
                Cell::new(&format_timestamp(tape.created_at)),
            ]);
        }
        
        println!("{}", table);
        Ok(())
    }
}
```

#### Afternoon: Show Command
```rust
// 3. Implement detailed tape display
#[derive(Args)]
pub struct ShowArgs {
    /// Tape ID to display
    tape_id: String,
    
    /// Show individual frames
    #[arg(long)]
    frames: bool,
    
    /// Show detailed statistics
    #[arg(long)]
    stats: bool,
    
    /// Output format
    #[arg(short, long, default_value = "table")]
    format: OutputFormat,
    
    /// Limit number of frames to show
    #[arg(long, default_value = "20")]
    frame_limit: usize,
}

impl TapeCommand {
    async fn show_tape(&self, args: &ShowArgs, recorder: &TapeRecorder) -> Result<()> {
        let tape_id = TapeId::parse(&args.tape_id)?;
        let tape = recorder.load_tape(&tape_id).await?;
        
        match args.format {
            OutputFormat::Table => {
                self.display_tape_info_table(&tape);
                
                if args.stats {
                    self.display_tape_statistics(&tape);
                }
                
                if args.frames {
                    self.display_frame_table(&tape, args.frame_limit);
                }
            }
            OutputFormat::Json => {
                self.display_tape_json(&tape, args.frames);
            }
        }
        
        Ok(())
    }
    
    fn display_tape_info_table(&self, tape: &Tape) -> Result<()> {
        use comfy_table::{Table, Cell, Color};
        
        let mut table = Table::new();
        table.add_row(vec![
            Cell::new("Field").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);
        
        table.add_row(vec!["ID", &tape.metadata.id.to_string()]);
        table.add_row(vec!["Name", &tape.metadata.name]);
        table.add_row(vec!["Transport", &format!("{:?}", tape.metadata.transport_type)]);
        table.add_row(vec!["Session ID", &tape.metadata.session_id.to_string()]);
        table.add_row(vec!["Frame Count", &tape.metadata.frame_count.to_string()]);
        table.add_row(vec!["Duration", &format_duration(tape.metadata.duration_ms)]);
        table.add_row(vec!["Total Size", &format_bytes(tape.metadata.total_bytes)]);
        table.add_row(vec!["Created", &format_timestamp(tape.metadata.created_at)]);
        
        if let Some(description) = &tape.metadata.description {
            table.add_row(vec!["Description", description]);
        }
        
        if !tape.metadata.tags.is_empty() {
            table.add_row(vec!["Tags", &tape.metadata.tags.join(", ")]);
        }
        
        println!("{}", table);
        Ok(())
    }
}
```

### Day 2: Action Commands (replay, delete, export)

#### Morning: Replay Command
```rust
// 4. Implement interactive replay command
#[derive(Args)]
pub struct ReplayArgs {
    /// Tape ID to replay
    tape_id: String,
    
    /// Playback speed multiplier
    #[arg(long, default_value = "1.0")]
    speed: f64,
    
    /// Start position (frame number, timestamp, or percentage)
    #[arg(long)]
    from: Option<String>,
    
    /// End position (frame number, timestamp, or percentage)
    #[arg(long)]
    to: Option<String>,
    
    /// Enable step-by-step mode
    #[arg(long)]
    step: bool,
    
    /// Disable progress display
    #[arg(long)]
    quiet: bool,
}

impl TapeCommand {
    async fn replay_tape(&self, args: &ReplayArgs, recorder: &TapeRecorder) -> Result<()> {
        let tape_id = TapeId::parse(&args.tape_id)?;
        let tape = recorder.load_tape(&tape_id).await?;
        
        println!("🎬 Replaying tape: {}", tape.metadata.name);
        println!("📊 {} frames, {} duration", 
                 tape.metadata.frame_count, 
                 format_duration(tape.metadata.duration_ms));
        
        let mut player = TapePlayer::new(tape);
        player.set_speed(args.speed)?;
        
        if let Some(from) = &args.from {
            let start_pos = parse_position(from, &player)?;
            player.seek(start_pos).await?;
        }
        
        if args.step {
            self.run_stepped_replay(player).await?;
        } else {
            self.run_continuous_replay(player, args.quiet).await?;
        }
        
        println!("✅ Replay completed successfully");
        Ok(())
    }
    
    async fn run_continuous_replay(&self, mut player: TapePlayer, quiet: bool) -> Result<()> {
        use indicatif::{ProgressBar, ProgressStyle};
        
        let progress = if !quiet {
            let pb = ProgressBar::new(player.total_frames() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} frames ({percent}%) ETA: {eta}")
                .progress_chars("#>-"));
            Some(pb)
        } else {
            None
        };
        
        let (frame_tx, mut frame_rx) = mpsc::channel(100);
        player.set_frame_sender(frame_tx);
        
        // Start playback
        let playback_task = tokio::spawn(async move {
            player.play().await
        });
        
        // Update progress
        while let Some(frame) = frame_rx.recv().await {
            if let Some(pb) = &progress {
                pb.inc(1);
                pb.set_message(format!("Frame {} - {}", 
                                     frame.id, 
                                     frame.message.method().unwrap_or("unknown")));
            }
        }
        
        if let Some(pb) = &progress {
            pb.finish_with_message("Replay completed");
        }
        
        playback_task.await??;
        Ok(())
    }
    
    async fn run_stepped_replay(&self, mut player: TapePlayer) -> Result<()> {
        use std::io::{self, Write};
        
        println!("🔍 Step-by-step replay mode");
        println!("Commands: [n]ext, [p]rev, [s]eek <pos>, [q]uit, [h]elp");
        
        loop {
            let progress = player.progress();
            println!("\n📍 Frame {}/{} - {:.1}% complete", 
                     progress.current_frame + 1,
                     progress.total_frames,
                     (progress.current_frame as f64 / progress.total_frames as f64) * 100.0);
            
            if let Some(current_frame) = player.current_frame() {
                self.display_frame_details(current_frame);
            }
            
            print!(">>> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim().to_lowercase().as_str() {
                "n" | "next" => {
                    if let Err(e) = player.step().await {
                        println!("❌ Error: {}", e);
                    }
                }
                "p" | "prev" => {
                    if let Err(e) = player.step_back().await {
                        println!("❌ Error: {}", e);
                    }
                }
                "q" | "quit" => break,
                "h" | "help" => self.display_step_help(),
                cmd if cmd.starts_with("s ") || cmd.starts_with("seek ") => {
                    let pos_str = cmd.split_whitespace().nth(1).unwrap_or("");
                    match parse_position(pos_str, &player) {
                        Ok(pos) => {
                            if let Err(e) = player.seek(pos).await {
                                println!("❌ Seek error: {}", e);
                            }
                        }
                        Err(e) => println!("❌ Invalid position: {}", e),
                    }
                }
                _ => println!("❓ Unknown command. Type 'h' for help."),
            }
        }
        
        Ok(())
    }
}
```

#### Afternoon: Delete & Export Commands
```rust
// 5. Implement delete command with confirmation
#[derive(Args)]
pub struct DeleteArgs {
    /// Tape ID to delete
    tape_id: String,
    
    /// Skip confirmation prompt
    #[arg(long)]
    confirm: bool,
    
    /// Force deletion even if tape is corrupted
    #[arg(long)]
    force: bool,
}

impl TapeCommand {
    async fn delete_tape(&self, args: &DeleteArgs, recorder: &TapeRecorder) -> Result<()> {
        let tape_id = TapeId::parse(&args.tape_id)?;
        
        // Load tape info for confirmation
        let tape_result = recorder.load_tape(&tape_id).await;
        let tape_name = match &tape_result {
            Ok(tape) => tape.metadata.name.clone(),
            Err(_) if args.force => format!("corrupted-{}", tape_id),
            Err(e) => return Err(e.clone().into()),
        };
        
        if !args.confirm {
            println!("⚠️  About to delete tape:");
            println!("   ID: {}", tape_id);
            println!("   Name: {}", tape_name);
            
            print!("Are you sure? [y/N]: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                println!("❌ Deletion cancelled");
                return Ok(());
            }
        }
        
        recorder.delete_tape(&tape_id).await?;
        println!("✅ Deleted tape: {}", tape_name);
        Ok(())
    }
}

// 6. Implement export command
#[derive(Args)]
pub struct ExportArgs {
    /// Tape ID to export
    tape_id: String,
    
    /// Export format
    #[arg(short, long)]
    format: ExportFormat,
    
    /// Output file path
    #[arg(short, long)]
    output: Option<String>,
    
    /// Pretty-print output
    #[arg(long)]
    pretty: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ExportFormat {
    Har,      // HTTP Archive format
    Postman,  // Postman collection
    Curl,     // cURL commands
    Json,     // Raw JSON
}

impl TapeCommand {
    async fn export_tape(&self, args: &ExportArgs, recorder: &TapeRecorder) -> Result<()> {
        let tape_id = TapeId::parse(&args.tape_id)?;
        let tape = recorder.load_tape(&tape_id).await?;
        
        println!("📤 Exporting tape: {}", tape.metadata.name);
        
        let exported_data = match args.format {
            ExportFormat::Har => export_to_har(&tape)?,
            ExportFormat::Postman => export_to_postman(&tape)?,
            ExportFormat::Curl => export_to_curl(&tape)?,
            ExportFormat::Json => export_to_json(&tape, args.pretty)?,
        };
        
        match &args.output {
            Some(path) => {
                fs::write(path, &exported_data).await?;
                println!("✅ Exported to: {}", path);
            }
            None => {
                println!("{}", exported_data);
            }
        }
        
        Ok(())
    }
}
```

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::recorder::TapeRecorder;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_list_command_table_format() {
        let temp_dir = TempDir::new().unwrap();
        let recorder = TapeRecorder::new(temp_dir.path());
        
        // Create test tapes
        create_test_tapes(&recorder).await;
        
        let args = ListArgs {
            format: OutputFormat::Table,
            filter: None,
            sort: SortField::Created,
            reverse: false,
            limit: None,
        };
        
        let cmd = TapeCommand { command: TapeSubcommand::List(args) };
        let result = cmd.execute(&recorder).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_command_with_frames() {
        let temp_dir = TempDir::new().unwrap();
        let recorder = TapeRecorder::new(temp_dir.path());
        
        let tape_id = create_test_tape(&recorder).await;
        
        let args = ShowArgs {
            tape_id: tape_id.to_string(),
            frames: true,
            stats: true,
            format: OutputFormat::Table,
            frame_limit: 10,
        };
        
        let cmd = TapeCommand { command: TapeSubcommand::Show(args) };
        let result = cmd.execute(&recorder).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_command_with_confirmation() {
        let temp_dir = TempDir::new().unwrap();
        let recorder = TapeRecorder::new(temp_dir.path());
        
        let tape_id = create_test_tape(&recorder).await;
        
        let args = DeleteArgs {
            tape_id: tape_id.to_string(),
            confirm: true,  // Skip interactive confirmation
            force: false,
        };
        
        let cmd = TapeCommand { command: TapeSubcommand::Delete(args) };
        let result = cmd.execute(&recorder).await;
        
        assert!(result.is_ok());
        
        // Verify tape is deleted
        let load_result = recorder.load_tape(&tape_id).await;
        assert!(load_result.is_err());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_cli_workflow_integration() {
    let temp_dir = TempDir::new().unwrap();
    let recorder = TapeRecorder::new(temp_dir.path());
    
    // 1. Create some test tapes
    let tape_ids = create_multiple_test_tapes(&recorder).await;
    
    // 2. List tapes
    let list_cmd = TapeCommand { 
        command: TapeSubcommand::List(ListArgs::default()) 
    };
    list_cmd.execute(&recorder).await.unwrap();
    
    // 3. Show detailed info
    let show_cmd = TapeCommand {
        command: TapeSubcommand::Show(ShowArgs {
            tape_id: tape_ids[0].to_string(),
            frames: true,
            stats: true,
            format: OutputFormat::Table,
            frame_limit: 5,
        })
    };
    show_cmd.execute(&recorder).await.unwrap();
    
    // 4. Export tape
    let export_cmd = TapeCommand {
        command: TapeSubcommand::Export(ExportArgs {
            tape_id: tape_ids[0].to_string(),
            format: ExportFormat::Json,
            output: None,
            pretty: true,
        })
    };
    export_cmd.execute(&recorder).await.unwrap();
    
    // 5. Delete tape
    let delete_cmd = TapeCommand {
        command: TapeSubcommand::Delete(DeleteArgs {
            tape_id: tape_ids[1].to_string(),
            confirm: true,
            force: false,
        })
    };
    delete_cmd.execute(&recorder).await.unwrap();
}
```

---

## User Experience Features

### Rich Output Formatting
```rust
// Utility functions for better UX
fn format_duration(duration_ms: Option<u64>) -> String {
    match duration_ms {
        Some(ms) => {
            let seconds = ms / 1000;
            let minutes = seconds / 60;
            let hours = minutes / 60;
            
            if hours > 0 {
                format!("{}h{}m{}s", hours, minutes % 60, seconds % 60)
            } else if minutes > 0 {
                format!("{}m{}s", minutes, seconds % 60)
            } else {
                format!("{}s", seconds)
            }
        }
        None => "unknown".to_string(),
    }
}

fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

fn format_timestamp(timestamp: u64) -> String {
    // Convert timestamp to human-readable format
    chrono::DateTime::from_timestamp(timestamp as i64 / 1000, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
```

### Error Handling with Context
```rust
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid tape ID format: {0}")]
    InvalidTapeId(String),
    
    #[error("Invalid position format: {0}. Expected frame number, timestamp (1500ms), or percentage (50%)")]
    InvalidPosition(String),
    
    #[error("Tape not found: {0}")]
    TapeNotFound(String),
    
    #[error("Export format not supported for transport type {transport:?}: {format:?}")]
    UnsupportedExport { transport: TransportType, format: ExportFormat },
    
    #[error("Interactive operation cancelled by user")]
    UserCancelled,
}
```

---

## Success Criteria

### Functional Requirements
- [x] All planned commands are implemented and functional
- [x] Rich table formatting provides clear, readable output
- [x] Interactive confirmations prevent accidental data loss
- [x] Export functionality works for multiple formats
- [x] Step-by-step replay provides useful debugging experience

### User Experience Requirements
- [x] Commands follow Unix conventions and feel familiar
- [x] Error messages are clear and actionable
- [x] Help text is comprehensive and includes examples
- [x] Progress indicators provide feedback for long operations
- [x] Output formatting is consistent and professional

### Performance Requirements
- [x] List command completes within 500ms for 100 tapes
- [x] Show command displays details within 200ms
- [x] Large tape operations show progress feedback
- [x] Memory usage stays reasonable during operations

This task creates a powerful and user-friendly CLI interface that makes Shadowcat's tape management capabilities accessible and intuitive for developers.