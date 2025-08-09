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
use std::fs;

fn main() -> io::Result<()> {
    // Enter alternate screen and enable raw mode for key input
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    // Print stylized header with gradient-like effect
    println!("{}", "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓".bright_green().bold());
    println!("{}", "┃ Hacker-Update ┃".bright_cyan().bold().on_bright_black());
    println!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛".bright_green().bold());
    println!("{}", " Initializing system updates...".bright_blue().italic());
    println!();
    // Define update commands with associated colors for visual distinction
    let update_commands = vec![
        ("DNF System Update", vec![
            ("sudo /usr/lib/HackerOS/dnf update -y", Color::BrightMagenta),
         ("sudo /usr/lib/HackerOS/dnf upgrade -y", Color::BrightMagenta),
         ("sudo /usr/lib/HackerOS/dnf autoremove -y", Color::BrightMagenta),
        ]),
        ("Flatpak Update", vec![("flatpak update -y", Color::BrightYellow)]),
        ("Snap Update", vec![("sudo snap refresh", Color::BrightBlue)]),
        ("Firmware Update", vec![
            ("sudo fwupdmgr refresh", Color::BrightGreen),
         ("sudo fwupdmgr update", Color::BrightGreen),
        ]),
    ];
    // Store logs
    let mut logs: Vec<String> = Vec::new();
    // Initialize MultiProgress for concurrent progress bars
    let multi_pb = MultiProgress::new();
    // Run each update command with enhanced progress bar
    for (update_name, commands) in update_commands {
        println!("{}", format!(" Starting {} ", update_name).white().bold().on_color(update_name_color(update_name)));
        let pb = multi_pb.add(ProgressBar::new(100));
        pb.set_style(
            ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{elapsed_precise}] {bar:50.green/blue} {msg} {percent:>3}%")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ ")
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
        );
        for (_i, (cmd, color)) in commands.iter().enumerate() {
            pb.set_message(format!("{}", cmd.color(*color).bold()).to_string());
            let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    logs.push(format!("{} Output:\n{}", cmd, stdout));
                    if !stderr.is_empty() {
                        logs.push(format!("{} Errors:\n{}", cmd, stderr));
                    }
                    if !output.status.success() {
                        println!("{}", format!(" Error running {} ", cmd).white().bold().on_bright_red());
                    } else {
                        println!("{}", format!(" Completed {} ", cmd).white().bold().on_bright_green());
                    }
                }
                Err(e) => {
                    logs.push(format!("Failed to execute {}: {}", cmd, e));
                    println!("{}", format!(" Failed to execute {}: {} ", cmd, e).white().bold().on_bright_red());
                }
            }
            pb.inc(100 / commands.len() as u64);
            thread::sleep(Duration::from_millis(200));
        }
        pb.finish_with_message(format!(" {} completed ", update_name).white().bold().to_string());
        println!();
    }
    // Run the custom script
    let script_path = "/usr/share/HackerOS/Scripts/Bin/Update-usrshare.sh";
    if fs::metadata(script_path).is_ok() {
        println!("{}", " Starting custom update script ".white().bold().on_bright_purple());
        let pb = multi_pb.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
            .template("{spinner:.purple} {msg} [{elapsed_precise}]")
            .unwrap()
            .tick_chars("⣾⣷⣯⣟⡿⢿⣻⣽"),
        );
        pb.set_message("Executing Update-usrshare.sh".to_string());
        let output = Command::new("sh")
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                logs.push(format!("Update-usrshare.sh Output:\n{}", stdout));
                if !stderr.is_empty() {
                    logs.push(format!("Update-usrshare.sh Errors:\n{}", stderr));
                }
                if !output.status.success() {
                    println!("{}", " Error running custom script ".white().bold().on_bright_red());
                } else {
                    println!("{}", " Completed custom script ".white().bold().on_bright_green());
                }
            }
            Err(e) => {
                logs.push(format!("Failed to execute script: {}", e));
                println!("{}", format!(" Failed to execute script: {} ", e).white().bold().on_bright_red());
            }
        }
        pb.finish_with_message(" Script execution completed ".to_string());
    } else {
        println!("{}", " Custom script not found ".white().bold().on_bright_red());
        logs.push("Custom script not found!".to_string());
    }
    // Display stylized menu with gradient borders
    loop {
        println!();
        println!("{}", "╔════════════════════════════════════════════════╗".bright_cyan().bold());
        println!("{}", "║ Update Completed! ║".white().bold().on_bright_black());
        println!("{}", "╠════════════════════════════════════════════════╣".bright_cyan().bold());
        println!("{}", "║ (E)xit (S)hutdown (R)eboot ║".bright_yellow().bold());
        println!("{}", "║ (L)og Out (T)ry again (S)ow logs ║".bright_yellow().bold());
        println!("{}", "╚════════════════════════════════════════════════╝".bright_cyan().bold());
        println!("{}", " Select an option: ".white().italic());
        io::stdout().flush()?;
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    println!("{}", " Exiting ".white().bold().on_bright_blue());
                    break;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    println!("{}", " Shutting down ".white().bold().on_bright_blue());
                    let _ = Command::new("sudo").arg("poweroff").output();
                    break;
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    println!("{}", " Rebooting ".white().bold().on_bright_blue());
                    let _ = Command::new("sudo").arg("reboot").output();
                    break;
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    println!("{}", " Logging out ".white().bold().on_bright_blue());
                    let _ = Command::new("pkill").arg("-u").arg(&whoami::username()).output();
                    break;
                }
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    println!("{}", " Restarting updates ".white().bold().on_bright_blue());
                    let _ = execute!(io::stdout(), LeaveAlternateScreen)?;
                    disable_raw_mode()?;
                    main()?;
                    return Ok(());
                }
                KeyCode::Char('h') | KeyCode::Char('H') => {
                    println!("{}", " Update Logs ".white().bold().on_bright_cyan());
                    println!("{}", "╔════════════════════════════════════════════════╗".bright_cyan().bold());
                    for log in &logs {
                        println!("{}", format!("║ {} ", log).white().on_bright_black());
                    }
                    println!("{}", "╚════════════════════════════════════════════════╝".bright_cyan().bold());
                }
                _ => {
                    println!("{}", " Invalid option, try again. ".white().bold().on_bright_red());
                }
            }
        }
    }
    // Cleanup
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

// Helper function to assign background colors based on update type
fn update_name_color(update_name: &str) -> Color {
    match update_name {
        "DNF System Update" => Color::BrightMagenta,
        "Flatpak Update" => Color::BrightYellow,
        "Snap Update" => Color::BrightBlue,
        "Firmware Update" => Color::BrightGreen,
        _ => Color::BrightBlack,
    }
}
