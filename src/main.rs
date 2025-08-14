use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Constants for styling
const HEADER_WIDTH: usize = 60;
const SPINNER_TICK_CHARS: &str = "⣾⣷⣯⣟⡿⢿⣻⣽";
const PROGRESS_BAR_CHARS: &str = "█▉▊▋▌▍▎▏ ";

// Structure to hold command details
struct CommandInfo {
    name: &'static str,
    cmd: &'static str,
    color: Color,
}

// Structure to manage update sections
struct UpdateSection {
    name: &'static str,
    commands: Vec<CommandInfo>,
}

fn main() -> io::Result<()> {
    // Initialize terminal
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    // Display stylized header
    print_header();

    // Define update sections
    let update_sections = vec![
        UpdateSection {
            name: "APT System Update",
            commands: vec![
                CommandInfo {
                    name: "APT Update",
                    cmd: "sudo /usr/lib/HackerOS/apt update",
                    color: Color::BrightMagenta,
                },
                CommandInfo {
                    name: "APT Upgrade",
                    cmd: "sudo /usr/lib/HackerOS/apt upgrade -y",
                    color: Color::BrightMagenta,
                },
                CommandInfo {
                    name: "APT Autoremove",
                    cmd: "sudo /usr/lib/HackerOS/apt autoremove -y",
                    color: Color::BrightMagenta,
                },
                CommandInfo {
                    name: "APT Autoclean",
                    cmd: "sudo /usr/lib/HackerOS/apt autoclean",
                    color: Color::BrightMagenta,
                },
            ],
        },
        UpdateSection {
            name: "Flatpak Update",
            commands: vec![
                CommandInfo {
                    name: "Flatpak Update",
                    cmd: "flatpak update -y",
                    color: Color::BrightYellow,
                },
            ],
        },
        UpdateSection {
            name: "Snap Update",
            commands: vec![
                CommandInfo {
                    name: "Snap Refresh",
                    cmd: "sudo snap refresh",
                    color: Color::BrightBlue,
                },
            ],
        },
        UpdateSection {
            name: "Firmware Update",
            commands: vec![
                CommandInfo {
                    name: "Firmware Refresh",
                    cmd: "sudo fwupdmgr refresh",
                    color: Color::BrightGreen,
                },
                CommandInfo {
                    name: "Firmware Update",
                    cmd: "sudo fwupdmgr update",
                    color: Color::BrightGreen,
                },
            ],
        },
        UpdateSection {
            name: "HackerOS Update",
            commands: vec![
                CommandInfo {
                    name: "HackerOS Script",
                    cmd: "/usr/share/HackerOS/Scripts/Bin/Update-usrshare.sh",
                    color: Color::Magenta,
                },
            ],
        },
    ];

    let mut logs: Vec<(String, String, bool)> = Vec::new();
    let multi_pb = MultiProgress::new();

    // Process each update section
    for section in update_sections {
        print_section_header(&section.name);
        let total_steps = section.commands.len() as u64 * 100;
        let pb = multi_pb.add(ProgressBar::new(total_steps));
        pb.set_style(
            ProgressStyle::with_template(
                "{prefix:.bold.dim} {spinner:.cyan/blue} [{bar:40.cyan/blue}] {percent}% | {msg} | ETA: {eta_precise}"
            )
            .unwrap()
            .progress_chars(PROGRESS_BAR_CHARS)
            .tick_chars(SPINNER_TICK_CHARS),
        );
        pb.set_prefix(format!("{} ", section.name));
        pb.enable_steady_tick(Duration::from_millis(50));

        for cmd_info in section.commands {
            pb.set_message(format!("{}", cmd_info.name.bright_white().bold()));
            // Simulate progress
            for _ in 0..100 {
                pb.inc(1);
                thread::sleep(Duration::from_millis(20));
            }
            let spinner = multi_pb.add(ProgressBar::new_spinner());
            spinner.set_style(
                ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars(SPINNER_TICK_CHARS),
            );
            spinner.set_message(format!("Executing: {}", cmd_info.name));
            spinner.enable_steady_tick(Duration::from_millis(40));

            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd_info.cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

            spinner.finish_and_clear();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let success = output.status.success();
                    logs.push((cmd_info.name.to_string(), stdout.clone(), true));
                    if !stderr.is_empty() {
                        logs.push((cmd_info.name.to_string(), stderr.clone(), false));
                    }
                    print_command_result(&cmd_info.name, success, cmd_info.color);
                }
                Err(e) => {
                    logs.push((cmd_info.name.to_string(), format!("Failed: {}", e), false));
                    print_command_result(&cmd_info.name, false, cmd_info.color);
                }
            }
        }
        pb.finish_with_message(format!("{} completed", section.name.bright_green().bold()));
        println!();
        thread::sleep(Duration::from_millis(300));
    }

    // Interactive menu
    loop {
        print_menu();
        io::stdout().flush()?;
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    print_action("Exiting Update Utility", Color::BrightBlue);
                    break;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    print_action("Shutting Down System", Color::BrightBlue);
                    let _ = Command::new("sudo").arg("poweroff").output();
                    break;
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    print_action("Rebooting System", Color::BrightBlue);
                    let _ = Command::new("sudo").arg("reboot").output();
                    break;
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    print_action("Logging Out", Color::BrightBlue);
                    let _ = Command::new("pkill").arg("-u").arg(&whoami::username()).output();
                    break;
                }
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    print_action("Restarting Update Process", Color::BrightBlue);
                    let _ = execute!(io::stdout(), LeaveAlternateScreen)?;
                    disable_raw_mode()?;
                    main()?;
                    return Ok(());
                }
                KeyCode::Char('h') | KeyCode::Char('H') => {
                    print_logs(&logs);
                }
                _ => {
                    print_action("Invalid Option", Color::BrightRed);
                }
            }
        }
    }

    // Cleanup
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

// Helper functions
fn print_header() {
    let title = "HackerOS Update Utility";
    let _padding = (HEADER_WIDTH - title.len()) / 2; // Unused but kept for potential future use
    println!("{}", "═".repeat(HEADER_WIDTH).bright_green().bold());
    println!("{}", format!("║{:^width$}║", title.bright_cyan().bold(), width = HEADER_WIDTH - 2).on_bright_black());
    println!("{}", "═".repeat(HEADER_WIDTH).bright_green().bold());
    println!("{}", "Initializing updates...".bright_blue().italic().bold());
    println!();
}

fn print_section_header(name: &str) {
    let padding = (HEADER_WIDTH - name.len() - 4) / 2;
    println!(
        "{}",
        format!("╠{} {} {}{}╣", "═".repeat(padding), name, "═".repeat(padding), if (name.len() + 4) % 2 == 1 { "═" } else { "" })
            .white().bold().on_color(get_section_color(name))
    );
    println!();
}

fn print_command_result(name: &str, success: bool, color: Color) {
    let status = if success { "Completed" } else { "Failed" };
    let _status_color = if success { Color::BrightGreen } else { Color::BrightRed }; // Unused but kept for potential future use
    println!(
        "{}",
        format!("╠═ {} {} ═╝", status, name).white().bold().on_color(color)
    );
}

fn print_menu() {
    println!("{}", format!("{}", "╒".to_string() + &"═".repeat(HEADER_WIDTH - 2) + &"╕".bright_cyan().bold()));
    println!("{}", format!("│{:^width$}│", "Update Process Completed", width = HEADER_WIDTH - 2).white().bold().on_bright_black());
    println!("{}", format!("{}", "├".to_string() + &"─".repeat(HEADER_WIDTH - 2) + &"┤".bright_cyan().bold()));
    println!("{}", format!("│{:^width$}│", "(E)xit (S)hutdown (R)eboot", width = HEADER_WIDTH - 2).bright_yellow().bold());
    println!("{}", format!("│{:^width$}│", "(L)og Out (T)ry Again (H) Show Logs", width = HEADER_WIDTH - 2).bright_yellow().bold());
    println!("{}", format!("{}", "╘".to_string() + &"═".repeat(HEADER_WIDTH - 2) + &"╛".bright_cyan().bold()));
    println!("{}", "Select an option:".white().italic().bold());
}

fn print_logs(logs: &[(String, String, bool)]) {
    println!("{}", format!("{}", "╒".to_string() + &"═".repeat(HEADER_WIDTH - 2) + &"╕".bright_cyan().bold()));
    println!("{}", format!("│{:^width$}│", "Update Logs", width = HEADER_WIDTH - 2).white().bold().on_bright_cyan());
    println!("{}", format!("{}", "├".to_string() + &"─".repeat(HEADER_WIDTH - 2) + &"┤".bright_cyan().bold()));
    for (name, log, is_stdout) in logs {
        let log_type = if *is_stdout { "Output" } else { "Error" };
        let log_color = if *is_stdout { Color::White } else { Color::BrightRed };
        let max_len = HEADER_WIDTH - 10;
        let truncated_log = if log.len() > max_len { &log[..max_len] } else { log };
        println!(
            "{}",
            format!("│ {}: {} {}", log_type, name, truncated_log).color(log_color).on_bright_black()
        );
    }
    println!("{}", format!("{}", "╘".to_string() + &"═".repeat(HEADER_WIDTH - 2) + &"╛".bright_cyan().bold()));
    println!();
}

fn print_action(message: &str, color: Color) {
    println!(
        "{}",
        format!("╠═ {} ═╝", message).white().bold().on_color(color)
    );
    thread::sleep(Duration::from_millis(200));
}

fn get_section_color(name: &str) -> Color {
    match name {
        "APT System Update" => Color::BrightMagenta,
        "Flatpak Update" => Color::BrightYellow,
        "Snap Update" => Color::BrightBlue,
        "Firmware Update" => Color::BrightGreen,
        "HackerOS Update" => Color::Magenta,
        _ => Color::BrightBlack,
    }
}
