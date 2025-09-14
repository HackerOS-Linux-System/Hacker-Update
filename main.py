import os
import sys
import subprocess
import threading
import queue
import time
import logging
from rich.console import Console
from rich.panel import Panel
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn, TimeRemainingColumn
from rich.text import Text
from rich.style import Style
from rich.live import Live
from rich import box

# Setup logging
logging.basicConfig(
    filename='/tmp/hacker-update.log',
    level=logging.DEBUG,
    format='%(asctime)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)

console = Console()

# Constants for styling
HEADER_WIDTH = 80
COLOR_MAP = {
    "APT System Update": "bright_magenta",
    "Flatpak Update": "bright_yellow",
    "Snap Update": "bright_blue",
    "Firmware Update": "bright_green",
}

# Structure for commands
commands = [
    {
        "name": "APT System Update",
        "commands": [
            {"name": "APT Update", "cmd": "sudo apt update", "color": "bright_magenta", "list_cmd": None},
            {"name": "APT Upgrade", "cmd": "sudo apt upgrade -y", "color": "bright_magenta", "list_cmd": "apt list --upgradable"},
            {"name": "APT Autoremove", "cmd": "sudo apt autoremove -y", "color": "bright_magenta", "list_cmd": None},
            {"name": "APT Autoclean", "cmd": "sudo apt autoclean", "color": "bright_magenta", "list_cmd": None},
        ]
    },
    {
        "name": "Flatpak Update",
        "commands": [
            {"name": "Flatpak Update", "cmd": "flatpak update -y", "color": "bright_yellow", "list_cmd": "flatpak remote-ls --updates flathub"},
        ]
    },
    {
        "name": "Snap Update",
        "commands": [
            {"name": "Snap Refresh", "cmd": "sudo snap refresh", "color": "bright_blue", "list_cmd": "snap refresh --list"},
        ]
    },
    {
        "name": "Firmware Update",
        "commands": [
            {"name": "Firmware Refresh", "cmd": "sudo fwupdmgr refresh", "color": "bright_green", "list_cmd": None},
            {"name": "Firmware Update", "cmd": "sudo fwupdmgr update", "color": "bright_green", "list_cmd": "fwupdmgr get-updates"},
        ]
    },
]

def print_header():
    title = Text("HackerOS Update Utility v0.0.1", style="bold bright_cyan")
    panel = Panel(title, width=HEADER_WIDTH, box=box.HEAVY, expand=True, style="on black")
    console.print(panel)
    console.print(Text("Initializing updates...", style="italic bright_blue", justify="center"))
    console.print()

def print_section_header(name):
    color = COLOR_MAP.get(name, "bright_white")
    header_text = Text(name, style=f"bold white on {color}")
    panel = Panel(header_text, width=HEADER_WIDTH, box=box.DOUBLE_EDGE, expand=True)
    console.print(panel)
    console.print()

def run_command(cmd, color, is_list=False):
    process = subprocess.Popen(
        ["sh", "-c", cmd],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    stdout_queue = queue.Queue()
    stderr_queue = queue.Queue()

    def read_stdout():
        for line in iter(process.stdout.readline, ''):
            stdout_queue.put(line.strip())
        process.stdout.close()

    def read_stderr():
        for line in iter(process.stderr.readline, ''):
            stderr_queue.put(line.strip())
        process.stderr.close()

    stdout_thread = threading.Thread(target=read_stdout)
    stderr_thread = threading.Thread(target=read_stderr)
    stdout_thread.start()
    stderr_thread.start()

    stdout_lines = []
    stderr_lines = []

    while process.poll() is None or not stdout_queue.empty() or not stderr_queue.empty():
        try:
            line = stdout_queue.get_nowait()
            if not is_list:
                console.print(Text(line, style=color))
            stdout_lines.append(line)
            logging.info(f"STDOUT: {line}")
        except queue.Empty:
            pass
        try:
            line = stderr_queue.get_nowait()
            if not is_list:
                console.print(Text(line, style="bright_red"))
            stderr_lines.append(line)
            logging.error(f"STDERR: {line}")
        except queue.Empty:
            pass
        time.sleep(0.01)

    stdout_thread.join()
    stderr_thread.join()
    return_code = process.wait()

    return '\n'.join(stdout_lines), '\n'.join(stderr_lines), return_code == 0

def list_updates(list_cmd, name, color):
    with console.status(f"[green]Listing updates for {name}...[/]", spinner="dots"):
        stdout, stderr, success = run_command(list_cmd, color, is_list=True)
    if success and stdout.strip():
        console.print(Text(f"Updates available for {name}:", style=f"bold {color}"))
        lines = [line for line in stdout.splitlines() if line.strip() and "Listing..." not in line and "All snaps up to date." not in line]
        if lines:
            for i, line in enumerate(lines, 1):
                console.print(Text(f" {i:2}. {line}", style="bright_white"))
        else:
            console.print(Text(" None", style="bright_white"))
    elif stderr:
        console.print(Text(f"Error listing updates: {stderr}", style="bright_red"))
    else:
        console.print(Text(f"No updates available for {name}.", style="bright_green"))
    console.print()

def print_menu():
    menu_text = Text.assemble(
        Text("Update Process Completed\n", style="bold bright_green", justify="center"),
        Text("Choose an action:\n", style="bold white", justify="center"),
        Text("(E)xit (S)hutdown (R)eboot\n", style="bright_yellow", justify="center"),
        Text("(L)og Out (T)ry Again (H) Show Logs", style="bright_yellow", justify="center")
    )
    panel = Panel(menu_text, width=HEADER_WIDTH, box=box.ROUNDED, expand=True, style="bright_cyan")
    console.print(panel)
    console.print(Text("Select an option:", style="italic white", justify="center"))

def print_logs(logs):
    log_content = Text()
    for name, log, is_stdout in logs:
        if log.strip():
            log_type = "Output" if is_stdout else "Error"
            log_color = "bright_white" if is_stdout else "bright_red"
            log_content.append(f"{log_type} for {name}:\n", style=f"bold {log_color}")
            for line in log.splitlines():
                log_content.append(f" {line}\n", style=log_color)
            log_content.append("\n")
    panel = Panel(log_content, title="Update Logs", width=HEADER_WIDTH, box=box.SQUARE, style="on bright_cyan", expand=True)
    console.print(panel)
    console.print()

def print_action(message, color):
    action_text = Text(message, style=f"white on {color}")
    panel = Panel(action_text, width=HEADER_WIDTH // 2, box=box.MINIMAL, expand=False, style="bold")
    console.print(panel, justify="center")
    time.sleep(0.2)

def get_single_key():
    import termios, tty
    fd = sys.stdin.fileno()
    old_attr = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        return sys.stdin.read(1).lower()
    finally:
        termios.tcsetattr(fd, termios.TCDRAIN, old_attr)

def main():
    print_header()
    logs = []

    for section in commands:
        print_section_header(section["name"])
        total_steps = len(section["commands"])
        progress = Progress(
            SpinnerColumn(),
            BarColumn(bar_width=None),
            TextColumn("[progress.description]{task.description}"),
            TextColumn("{task.completed}/{task.total}"),
            "|",
            TextColumn("{task.fields[msg]}"),
            "| ETA:",
            TimeRemainingColumn(),
            console=console,
            expand=True
        )
        task_id = progress.add_task(f"[bright_cyan]{section['name']:<30}[/]", total=total_steps, msg="")

        with Live(progress, refresh_per_second=20):
            for cmd_info in section["commands"]:
                progress.update(task_id, description=f"[bright_cyan]{section['name']:<30}[/]", msg=f"{cmd_info['name']}")
                if cmd_info["list_cmd"]:
                    list_updates(cmd_info["list_cmd"], cmd_info["name"], cmd_info["color"])

                with console.status(f"[green]Executing: {cmd_info['name']}[/]", spinner="dots"):
                    stdout, stderr, success = run_command(cmd_info["cmd"], cmd_info["color"])

                logs.append((cmd_info["name"], stdout, True))
                if stderr:
                    logs.append((cmd_info["name"], stderr, False))

                status_msg = Text(cmd_info["name"] + ": " + ("Completed" if success else "Failed"), style="bright_green bold" if success else "bright_red bold")
                console.print(status_msg)
                console.print()

                progress.advance(task_id)

            progress.update(task_id, description=f"[bright_cyan]{section['name']:<30}[/]", msg=f"{section['name']} completed", completed=total_steps)
        console.print(Text(f"{section['name']} completed", style="bold bright_green"))
        console.print()
        time.sleep(0.3)

    while True:
        print_menu()
        choice = get_single_key()
        if choice == 'e':
            print_action("Exiting Update Utility", "bright_blue")
            break
        elif choice == 's':
            print_action("Shutting Down System", "bright_blue")
            subprocess.run(["sudo", "poweroff"])
            break
        elif choice == 'r':
            print_action("Rebooting System", "bright_blue")
            subprocess.run(["sudo", "reboot"])
            break
        elif choice == 'l':
            print_action("Logging Out", "bright_blue")
            username = os.getlogin()
            subprocess.run(["pkill", "-u", username])
            break
        elif choice == 't':
            print_action("Restarting Update Process", "bright_blue")
            main()
            return
        elif choice == 'h':
            print_logs(logs)
        else:
            print_action("Invalid Option", "bright_red")

if __name__ == "__main__":
    main()
