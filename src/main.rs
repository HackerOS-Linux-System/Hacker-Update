use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Constants for styling
const HEADER_WIDTH: usize = 80;
const SPINNER_TICK_CHARS: &str = "⠁⠂⠄⠈⠐⠠⢀⡀";
const PROGRESS_BAR_CHARS: &str = "█▓▒░ ";

// Structure to hold command details
struct CommandInfo {
    name: &'static str,
    cmd: &'static str,
    color: Color,
    list_cmd: Option<&'static str>,
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
                    cmd: "sudo apt update",
                    color: Color::BrightMagenta,
                    list_cmd: None,
                },
                CommandInfo {
                    name: "APT Upgrade",
                    cmd: "sudo apt upgrade -y",
                    color: Color::BrightMagenta,
                    list_cmd: Some("apt list --upgradable"),
                },
                CommandInfo {
                    name: "APT Autoremove",
                    cmd: "sudo apt autoremove -y",
                    color: Color::BrightMagenta,
                    list_cmd: None,
                },
                CommandInfo {
                    name: "APT Autoclean",
                    cmd: "sudo apt autoclean",
                    color: Color::BrightMagenta,
                    list_cmd: None,
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
                    list_cmd: Some("flatpak remote-ls --updates flathub"),
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
                    list_cmd: Some("snap refresh --list"),
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
                    list_cmd: None,
                },
                CommandInfo {
                    name: "Firmware Update",
                    cmd: "sudo fwupdmgr update",
                    color: Color::BrightGreen,
                    list_cmd: Some("fwupdmgr get-updates"),
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
                    list_cmd: None,
                },
            ],
        },
    ];

    let mut logs: Vec<(String, String, bool)> = Vec::new();
    let multi_pb = MultiProgress::new();

    // Process each update section
    for section in update_sections {
        print_section_header(&section.name);
        let total_steps = section.commands.len() as u64;
        let pb = multi_pb.add(ProgressBar::new(total_steps));
        pb.set_style(
            ProgressStyle::with_template(
                "{prefix:.bold.dim} {spinner:.cyan/blue} [{wide_bar:.cyan/blue}] {pos}/{len} | {msg:.white.bold} | ETA: {eta}"
            )
            .unwrap()
            .progress_chars(PROGRESS_BAR_CHARS)
            .tick_chars(SPINNER_TICK_CHARS),
        );
        pb.set_prefix(format!("{:<30}", section.name.bright_cyan()));
        pb.enable_steady_tick(Duration::from_millis(50));

        for cmd_info in section.commands {
            pb.set_message(format!("{}", cmd_info.name));
            if let Some(list_cmd) = cmd_info.list_cmd {
                let list_spinner = multi_pb.add(ProgressBar::new_spinner());
                list_spinner.set_style(
                    ProgressStyle::with_template("{spinner:.green} {msg:.white}")
                    .unwrap()
                    .tick_chars(SPINNER_TICK_CHARS),
                );
                list_spinner.set_message(format!("Listing updates for {}...", cmd_info.name));
                list_spinner.enable_steady_tick(Duration::from_millis(40));

                let list_output = Command::new("sh")
                .arg("-c")
                .arg(list_cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

                list_spinner.finish_and_clear();
                match list_output {
                    Ok(o) => {
                        let list_stdout = String::from_utf8_lossy(&o.stdout).to_string();
                        let list_stderr = String::from_utf8_lossy(&o.stderr).to_string();
                        if !list_stdout.trim().is_empty() && o.status.success() {
                            println!(
                                "{}",
                                format!("Updates available for {}:", cmd_info.name).bold().color(cmd_info.color)
                            );
                            let mut count = 0;
                            for line in list_stdout.lines().filter(|l| !l.trim().is_empty() && !l.contains("Listing...") && !l.contains("All snaps up to date.")) {
                                println!("  {:2}. {}", count + 1, line.bright_white());
                                count += 1;
                            }
                            if count == 0 {
                                println!("{}", "  None".bright_white());
                            }
                        } else if !list_stderr.is_empty() {
                            println!("{}", format!("Error listing updates: {}", list_stderr).bright_red());
                        } else {
                            println!(
                                "{}",
                                format!("No updates available for {}.", cmd_info.name).bright_green()
                            );
                        }
                    }
                    Err(e) => {
                        println!("{}", format!("Failed to list updates: {}", e).bright_red());
                    }
                }
                println!();
            }

            let spinner = multi_pb.add(ProgressBar::new_spinner());
            spinner.set_style(
                ProgressStyle::with_template("{spinner:.green} {msg:.white}")
                .unwrap()
                .tick_chars(SPINNER_TICK_CHARS),
            );
            spinner.set_message(format!("Executing: {}", cmd_info.name));
            spinner.enable_steady_tick(Duration::from_millis(40));

            // Spawn the command
            let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd_info.cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

            let stdout_reader = BufReader::new(child.stdout.take().unwrap());
            let stderr_reader = BufReader::new(child.stderr.take().unwrap());

            let stdout_lines = Arc::new(Mutex::new(Vec::<String>::new()));
            let stderr_lines = Arc::new(Mutex::new(Vec::<String>::new()));

            let stdout_lines_clone = Arc::clone(&stdout_lines);
            let stderr_lines_clone = Arc::clone(&stderr_lines);
            let cmd_color = cmd_info.color;

            // Threads to read stdout and stderr
            let stdout_handle: JoinHandle<()> = thread::spawn(move || {
                for line in stdout_reader.lines().flatten() {
                    println!("{}", line.color(cmd_color));
                    stdout_lines_clone.lock().unwrap().push(line);
                }
            });

            let stderr_handle: JoinHandle<()> = thread::spawn(move || {
                for line in stderr_reader.lines().flatten() {
                    println!("{}", line.bright_red());
                    stderr_lines_clone.lock().unwrap().push(line);
                }
            });

            // Wait for threads to finish
            stdout_handle.join().unwrap();
            stderr_handle.join().unwrap();

            // Wait for child and get status
            let status = child.wait()?;
            let success = status.success();

            let stdout = stdout_lines.lock().unwrap().join("\n");
            let stderr = stderr_lines.lock().unwrap().join("\n");

            spinner.finish_with_message(format!(
                "{}: {}",
                cmd_info.name,
                if success { "Completed".bright_green().bold() } else { "Failed".bright_red().bold() }
            ));
            println!();

            logs.push((cmd_info.name.to_string(), stdout.clone(), true));
            if !stderr.is_empty() {
                logs.push((cmd_info.name.to_string(), stderr.clone(), false));
            }
            pb.inc(1);
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
    let title = "HackerOS Update Utility v0.0.1";
    println!("{}", "┏".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┓".bright_green().bold());
    println!("{}", format!("┃{:^width$}┃", title.bright_cyan().bold(), width = HEADER_WIDTH - 2).on_bright_black());
    println!("{}", "┗".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┛".bright_green().bold());
    println!("{}", format!("{:^width$}", "Initializing updates...".bright_blue().italic().bold(), width = HEADER_WIDTH));
    println!();
}

fn print_section_header(name: &str) {
    let padding = (HEADER_WIDTH - name.len() - 4) / 2;
    println!(
        "{}",
        format!("┣{} {} {}{}┫", "━".repeat(padding), name.bright_white().bold(), "━".repeat(padding), if (name.len() + 4) % 2 == 1 { "━" } else { "" })
            .on_color(get_section_color(name))
    );
    println!();
}

fn print_menu() {
    println!("{}", format!("{}", "┏".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┓".bright_cyan().bold()));
    println!("{}", format!("┃{:^width$}┃", "Update Process Completed".bright_green().bold(), width = HEADER_WIDTH - 2).on_bright_black());
    println!("{}", format!("{}", "┣".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┫".bright_cyan().bold()));
    println!("{}", format!("┃{:^width$}┃", "Choose an action:", width = HEADER_WIDTH - 2).bright_white().bold());
    println!("{}", format!("┃{:^width$}┃", "(E)xit  (S)hutdown  (R)eboot", width = HEADER_WIDTH - 2).bright_yellow());
    println!("{}", format!("┃{:^width$}┃", "(L)og Out  (T)ry Again  (H) Show Logs", width = HEADER_WIDTH - 2).bright_yellow());
    println!("{}", format!("{}", "┗".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┛".bright_cyan().bold()));
    println!("{}", format!("{:^width$}", "Select an option:".white().italic().bold(), width = HEADER_WIDTH));
}

fn print_logs(logs: &[(String, String, bool)]) {
    println!("{}", format!("{}", "┏".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┓".bright_cyan().bold()));
    println!("{}", format!("┃{:^width$}┃", "Update Logs", width = HEADER_WIDTH - 2).white().bold().on_bright_cyan());
    println!("{}", format!("{}", "┣".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┫".bright_cyan().bold()));
    for (name, log, is_stdout) in logs {
        let log_type = if *is_stdout { "Output" } else { "Error" };
        let log_color = if *is_stdout { Color::BrightWhite } else { Color::BrightRed };
        if !log.trim().is_empty() {
            println!("{}", format!("┃ {} for {}:", log_type, name).bold().color(log_color));
            for line in log.lines() {
                println!("{}", format!("┃   {}", line).color(log_color));
            }
            println!("{}", format!("{}", "┣".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┫".bright_black()));
        }
    }
    println!("{}", format!("{}", "┗".to_string() + &"━".repeat(HEADER_WIDTH - 2) + &"┛".bright_cyan().bold()));
    println!();
}

fn print_action(message: &str, color: Color) {
    println!(
        "{}",
        format!("┣━ {} ━┫", message).white().bold().on_color(color)
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
