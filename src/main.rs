use std::process::{Command, ExitStatus};
use std::env;
use std::os::unix::fs::PermissionsExt;

const DNF_PATH: &str = "/usr/lib/HackerOS/dnf";

fn print_help() {
    println!("hacker: A simple package management tool");
    println!("Usage: hacker <command> [arguments]");
    println!("\nAvailable commands:");
    println!(" autoremove        Remove unneeded packages");
    println!(" install <packages> Install one or more packages");
    println!(" remove <packages>  Remove one or more packages");
    println!(" list              List installed packages");
    println!(" search <term>     Search for packages");
    println!(" clean             Clean package cache");
    println!(" info <package>    Show package information");
    println!(" repolist          List enabled repositories");
    println!(" copr-enable <repo> Enable a COPR repository");
    println!(" copr-disable <repo> Disable a COPR repository");
    println!(" ?                 Show this help message");
    println!("\nNote: Use 'hacker-update' for system updates and upgrades.");
}

fn execute_dnf(args: Vec<&str>, use_sudo: bool) -> Result<ExitStatus, String> {
    let mut command = if use_sudo {
        let mut cmd = Command::new("sudo");
        cmd.arg(DNF_PATH);
        cmd
    } else {
        Command::new(DNF_PATH)
    };

    let output = command
    .args(&args)
    .status()
    .map_err(|e| format!("Failed to execute dnf: {}", e))?;
    Ok(output)
}

fn can_run_without_sudo() -> bool {
    // Check if user has write permissions to /usr/lib/HackerOS/dnf
    if let Ok(metadata) = std::fs::metadata(DNF_PATH) {
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        // Check if executable and writable by user or group
        (mode & 0o111) != 0 && (mode & 0o600) != 0
    } else {
        false
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Error: No command provided");
        print_help();
        std::process::exit(1);
    }

    let command = &args[1];
    let use_sudo = !can_run_without_sudo();

    match command.as_str() {
        "autoremove" => {
            match execute_dnf(vec!["autoremove", "-y"], use_sudo) {
                Ok(status) if status.success() => println!("Autoremove completed successfully"),
                Ok(_) => println!("Autoremove failed"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "install" => {
            if args.len() < 3 {
                println!("Error: At least one package name required for install");
                std::process::exit(1);
            }
            let packages = &args[2..];
            let mut dnf_args = vec!["install", "-y"];
            dnf_args.extend(packages.iter().map(|s| s.as_str()));
            match execute_dnf(dnf_args, use_sudo) {
                Ok(status) if status.success() => println!("Package(s) {} installed successfully", packages.join(" ")),
                Ok(_) => println!("Failed to install package(s) {}", packages.join(" ")),
                Err(e) => println!("Error: {}", e),
            }
        }
        "remove" => {
            if args.len() < 3 {
                println!("Error: At least one package name required for remove");
                std::process::exit(1);
            }
            let packages = &args[2..];
            let mut dnf_args = vec!["remove", "-y"];
            dnf_args.extend(packages.iter().map(|s| s.as_str()));
            match execute_dnf(dnf_args, use_sudo) {
                Ok(status) if status.success() => println!("Package(s) {} removed successfully", packages.join(" ")),
                Ok(_) => println!("Failed to remove package(s) {}", packages.join(" ")),
                Err(e) => println!("Error: {}", e),
            }
        }
        "list" => {
            match execute_dnf(vec!["list", "installed"], use_sudo) {
                Ok(status) if status.success() => println!("Listed installed packages"),
                Ok(_) => println!("Failed to list packages"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "search" => {
            if args.len() < 3 {
                println!("Error: Search term required");
                std::process::exit(1);
            }
            let term = &args[2];
            match execute_dnf(vec!["search", term], use_sudo) {
                Ok(status) if status.success() => println!("Search completed"),
                Ok(_) => println!("Search failed"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "clean" => {
            match execute_dnf(vec!["clean", "all"], use_sudo) {
                Ok(status) if status.success() => println!("Package cache cleaned successfully"),
                Ok(_) => println!("Failed to clean package cache"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "info" => {
            if args.len() < 3 {
                println!("Error: Package name required for info");
                std::process::exit(1);
            }
            let package = &args[2];
            match execute_dnf(vec!["info", package], use_sudo) {
                Ok(status) if status.success() => println!("Package information displayed"),
                Ok(_) => println!("Failed to display package information"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "repolist" => {
            match execute_dnf(vec!["repolist"], use_sudo) {
                Ok(status) if status.success() => println!("Repository list displayed"),
                Ok(_) => println!("Failed to display repository list"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "copr-enable" => {
            if args.len() < 3 {
                println!("Error: COPR repository name required");
                std::process::exit(1);
            }
            let repo = &args[2];
            match execute_dnf(vec!["copr", "enable", repo], use_sudo) {
                Ok(status) if status.success() => println!("COPR repository {} enabled", repo),
                Ok(_) => println!("Failed to enable COPR repository {}", repo),
                Err(e) => println!("Error: {}", e),
            }
        }
        "copr-disable" => {
            if args.len() < 3 {
                println!("Error: COPR repository name required");
                std::process::exit(1);
            }
            let repo = &args[2];
            match execute_dnf(vec!["copr", "disable", repo], use_sudo) {
                Ok(status) if status.success() => println!("COPR repository {} disabled", repo),
                Ok(_) => println!("Failed to disable COPR repository {}", repo),
                Err(e) => println!("Error: {}", e),
            }
        }
        "update" | "upgrade" => {
            println!("Error: Use 'hacker-update' for system updates and upgrades.");
            std::process::exit(1);
        }
        "?" => {
            print_help();
        }
        _ => {
            println!("Error: Unknown command '{}'", command);
            print_help();
            std::process::exit(1);
        }
    }
}
