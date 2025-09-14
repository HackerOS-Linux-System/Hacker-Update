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
from rich.live import Live
from rich.table import Table
from rich.theme import Theme
from rich import box

# Custom theme for a vibrant, hacker-themed palette
custom_theme = Theme({
    "header": "bold bright_cyan on black",
    "section": "bold white",
    "success": "bold green",
    "error": "bold red",
    "info": "bright_white",
    "apt": "bright_magenta",
    "flatpak": "yellow",
    "snap": "bright_blue",
    "firmware": "bright_green",
    "highlight": "cyan",
    "accent": "purple",
    "warning": "bright_yellow",
    "border": "bright_cyan",
    "secondary": "blue",
    "contrast": "bright_black",
    "neon": "bright_white",
    "shadow": "dim blue",
    "glow": "cyan"
})

console = Console(theme=custom_theme, record=True)

# Setup detailed logging
logging.basicConfig(
    filename='/tmp/hacker-update.log',
    level=logging.DEBUG,
    format='%(asctime)s - %(levelname)s - [%(name)s] - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger("HackerOS_Update")

# Constants for styling
HEADER_WIDTH = 120
COLOR_MAP = {
    "APT System Update": {"style": "apt", "color": "bright_magenta"},
    "Flatpak Update": {"style": "flatpak", "color": "yellow"},
    "Snap Update": {"style": "snap", "color": "bright_blue"},
    "Firmware Update": {"style": "firmware", "color": "bright_green"},
}

# Structure for commands
commands = [
    {
        "name": "APT System Update",
        "commands": [
            {"name": "APT Update", "cmd": "sudo apt update", "color": "apt", "list_cmd": None},
            {"name": "APT Upgrade", "cmd": "sudo apt upgrade -y", "color": "apt", "list_cmd": "apt list --upgradable"},
            {"name": "APT Autoremove", "cmd": "sudo apt autoremove -y", "color": "apt", "list_cmd": None},
            {"name": "APT Autoclean", "cmd": "sudo apt autoclean", "color": "apt", "list_cmd": None},
        ]
    },
    {
        "name": "Flatpak Update",
        "commands": [
            {"name": "Flatpak Update", "cmd": "flatpak update -y", "color": "flatpak", "list_cmd": "flatpak remote-ls --updates flathub"},
        ]
    },
    {
        "name": "Snap Update",
        "commands": [
            {"name": "Snap Refresh", "cmd": "sudo snap refresh", "color": "snap", "list_cmd": "snap refresh --list"},
        ]
    },
    {
        "name": "Firmware Update",
        "commands": [
            {"name": "Firmware Refresh", "cmd": "sudo fwupdmgr refresh", "color": "firmware", "list_cmd": None},
            {"name": "Firmware Update", "cmd": "sudo fwupdmgr update", "color": "firmware", "list_cmd": "fwupdmgr get-updates"},
        ]
    },
]

def print_header():
    title = Text("HackerOS Update Utility v0.0.1", style="header", justify="center")
    subtitle = Text("Initializing System Update Sequence...", style="italic shadow", justify="center")
    panel = Panel(
        Text.assemble(title, "\n", subtitle),
        width=HEADER_WIDTH,
        box=box.HEAVY_HEAD,
        border_style="border",
        padding=(2, 4),
        style="on black",
        title="[accent]System Update Terminal[/]",
        title_align="center"
    )
    console.print(panel)
    console.print()
    logger.info("Initialized HackerOS Update Utility")

def print_section_header(name):
    color_info = COLOR_MAP.get(name, {"style": "section", "color": "white"})
    header_text = Text(name.upper(), style=f"section on {color_info['color']}")
    panel = Panel(
        header_text,
        width=HEADER_WIDTH,
        box=box.DOUBLE_EDGE,
        border_style=color_info['color'],
        padding=(1, 4),
        title=f"[glow]{name}[/]",
        title_align="center"
    )
    console.print(panel)
    console.print()
    logger.info(f"Starting section: {name}")

def run_command(cmd, color):
    logger.debug(f"Executing command: {cmd}")
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

    try:
        with console.status(f"[cyan]Executing {cmd.split()[-1]}...[/]", spinner="dots12"):
            while process.poll() is None or not stdout_queue.empty() or not stderr_queue.empty():
                try:
                    line = stdout_queue.get_nowait()
                    console.print(Text(f"  {line}", style=color))
                    stdout_lines.append(line)
                    logger.info(f"STDOUT: {line}")
                except queue.Empty:
                    pass
                try:
                    line = stderr_queue.get_nowait()
                    console.print(Text(f"  {line}", style="error"))
                    stderr_lines.append(line)
                    logger.error(f"STDERR: {line}")
                except queue.Empty:
                    pass
                time.sleep(0.01)
    except KeyboardInterrupt:
        logger.warning("KeyboardInterrupt received, terminating command: {cmd}")
        process.terminate()
        try:
            process.wait(timeout=1)  # Give the process a moment to terminate
        except subprocess.TimeoutExpired:
            process.kill()  # Force kill if it doesn't terminate
        console.print(Panel(Text("Process interrupted by user.", style="error"), box=box.ROUNDED_DOUBLE, border_style="error", padding=(1, 2)))
        stdout_thread.join()
        stderr_thread.join()
        logger.info("Command execution interrupted and cleaned up")
        return '\n'.join(stdout_lines), '\n'.join(stderr_lines), False

    stdout_thread.join()
    stderr_thread.join()
    return_code = process.wait()
    logger.debug(f"Command completed with return code: {return_code}")

    return '\n'.join(stdout_lines), '\n'.join(stderr_lines), return_code == 0

def list_updates(list_cmd, name, color):
    logger.debug(f"Listing updates for: {name}")
    color_info = COLOR_MAP.get(name, {"style": "section", "color": "white"})
    with console.status(f"[glow]Scanning updates for {name}...[/]", spinner="dots12"):
        stdout, stderr, success = run_command(list_cmd, color)
        stdout = stdout.strip()
        stderr = stderr.strip()
    if success and stdout:
        table = Table(
            title=f"[glow]Available Updates for {name}[/]",
            box=box.SQUARE_DOUBLE_HEAD,
            style=color_info['color'],
            title_style="bold glow",
            header_style="bold accent"
        )
        table.add_column("No.", style="neon", justify="right", width=5)
        table.add_column("Update", style="neon")
        lines = [line for line in stdout.splitlines() if line.strip() and "Listing..." not in line and "All snaps up to date." not in line]
        if lines:
            for i, line in enumerate(lines, 1):
                table.add_row(str(i), line, style="neon")
            console.print(table)
        else:
            console.print(Panel(Text("No updates available.", style="success"), box=box.ROUNDED_DOUBLE, border_style="success", padding=(1, 2)))
    elif stderr:
        console.print(Panel(Text(f"Error: {stderr}", style="error"), box=box.ROUNDED_DOUBLE, border_style="error", padding=(1, 2)))
    else:
        console.print(Panel(Text(f"No updates available for {name}.", style="success"), box=box.ROUNDED_DOUBLE, border_style="success", padding=(1, 2)))
    console.print()
    logger.info(f"Completed listing updates for: {name}")

def print_menu():
    menu_text = Text.assemble(
        Text("Update Process Completed", style="success", justify="center"),
        Text("\nSelect an Action:", style="section", justify="center"),
        Text("\n(E)xit | (S)hutdown | (R)eboot", style="warning", justify="center"),
        Text("\n(L)og Out | (T)ry Again | (H) Show Logs", style="warning", justify="center")
    )
    panel = Panel(
        menu_text,
        width=HEADER_WIDTH,
        box=box.ASCII_DOUBLE_HEAD,
        border_style="border",
        padding=(2, 4),
        style="on black",
        title="[accent]Control Panel[/]",
        title_align="center"
    )
    console.print(panel)
    console.print(Text("Enter your choice:", style="italic cyan", justify="center"))
    logger.info("Displayed action menu")

def print_logs(logs):
    log_content = Text()
    for name, log, is_stdout in logs:
        if log.strip():
            log_type = "Output" if is_stdout else "Error"
            log_color = "neon" if is_stdout else "error"
            log_content.append(f"{log_type} for {name}:\n", style=f"bold {log_color}")
            for line in log.splitlines():
                log_content.append(f"  {line}\n", style=log_color)
            log_content.append("\n")
    panel = Panel(
        log_content,
        title="[glow]System Update Logs[/]",
        width=HEADER_WIDTH,
        box=box.DOUBLE,
        border_style="border",
        padding=(2, 4),
        style="on black"
    )
    console.print(panel)
    console.print()
    logger.info("Displayed update logs")

def print_action(message, color):
    action_text = Text(message, style=f"section on {color}")
    panel = Panel(
        action_text,
        width=HEADER_WIDTH // 2,
        box=box.HEAVY_OUTER,
        border_style=f"bold {color}",
        padding=(1, 4),
        style="on contrast"
    )
    console.print(panel, justify="center")
    time.sleep(0.4)
    logger.info(f"Action displayed: {message}")

def get_single_key():
    try:
        import termios, tty
        fd = sys.stdin.fileno()
        old_attr = termios.tcgetattr(fd)
        try:
            tty.setraw(fd)
            return sys.stdin.read(1).lower()
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_attr)
    except KeyboardInterrupt:
        logger.warning("KeyboardInterrupt received during menu input")
        console.print(Panel(Text("Input interrupted by user.", style="error"), box=box.ROUNDED_DOUBLE, border_style="error", padding=(1, 2)))
        return 'e'  # Default to exit on interrupt

def main():
    print_header()
    logs = []

    try:
        for section in commands:
            print_section_header(section["name"])
            total_steps = len(section["commands"])
            progress = Progress(
                SpinnerColumn(spinner_name="dots12", style="cyan"),
                BarColumn(bar_width=None, style="cyan dim", complete_style="cyan", finished_style="green"),
                TextColumn("[cyan]{task.description}", style="cyan"),
                TextColumn("{task.completed}/{task.total}", style="warning"),
                TextColumn("|", style="glow"),
                TextColumn("{task.fields[msg]}", style="neon"),
                TextColumn("| ETA:", style="secondary"),
                TimeRemainingColumn(),
                console=console,
                expand=True
            )
            task_id = progress.add_task(f"[cyan]{section['name']:<30}[/]", total=total_steps, msg="")

            with Live(progress, refresh_per_second=60):
                for cmd_info in section["commands"]:
                    progress.update(task_id, description=f"[cyan]{section['name']:<30}[/]", msg=f"{cmd_info['name']}")
                    logger.info(f"Starting command: {cmd_info['name']}")
                    if cmd_info["list_cmd"]:
                        list_updates(cmd_info["list_cmd"], cmd_info["name"], cmd_info["color"])

                    stdout, stderr, success = run_command(cmd_info["cmd"], cmd_info["color"])

                    logs.append((cmd_info["name"], stdout, True))
                    if stderr:
                        logs.append((cmd_info["name"], stderr, False))

                    status_msg = Text(
                        f"{cmd_info['name']}: {'Completed' if success else 'Failed'}",
                        style="success" if success else "error"
                    )
                    console.print(Panel(status_msg, box=box.ROUNDED_DOUBLE, border_style="success" if success else "error", padding=(0, 3)))
                    console.print()

                    progress.advance(task_id)

                progress.update(task_id, description=f"[cyan]{section['name']:<30}[/]", msg=f"{section['name']} Completed", completed=total_steps)
            console.print(Panel(Text(f"{section['name']} Completed Successfully", style="success"), box=box.ROUNDED_DOUBLE, border_style="success", padding=(1, 3)))
            console.print()
            time.sleep(0.5)
            logger.info(f"Completed section: {section['name']}")

    except KeyboardInterrupt:
        logger.warning("KeyboardInterrupt received in main loop")
        console.print(Panel(Text("Update process interrupted by user.", style="error"), box=box.ROUNDED_DOUBLE, border_style="error", padding=(1, 2)))
        return

    while True:
        print_menu()
        choice = get_single_key()
        logger.info(f"User selected option: {choice}")
        if choice == 'e':
            print_action("Exiting Update Utility", "glow")
            break
        elif choice == 's':
            print_action("Shutting Down System", "glow")
            subprocess.run(["sudo", "poweroff"])
            break
        elif choice == 'r':
            print_action("Rebooting System", "glow")
            subprocess.run(["sudo", "reboot"])
            break
        elif choice == 'l':
            print_action("Logging Out", "glow")
            username = os.getlogin()
            subprocess.run(["pkill", "-u", username])
            break
        elif choice == 't':
            print_action("Restarting Update Process", "glow")
            main()
            return
        elif choice == 'h':
            print_logs(logs)
        else:
            print_action("Invalid Option", "error")

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        logger.warning("KeyboardInterrupt received at top level")
        console.print(Panel(Text("Program terminated by user.", style="error"), box=box.ROUNDED_DOUBLE, border_style="error", padding=(1, 2)))
        sys.exit(1)
