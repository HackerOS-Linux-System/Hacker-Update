#!/bin/bash

# Log file for updates in /tmp
LOGFILE="/tmp/update_log_$(date +%Y%m%d_%H%M%S).txt"

# Expanded color palette for vibrant output
RED='\033[1;31m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
CYAN='\033[1;36m'
BLUE='\033[1;34m'
MAGENTA='\033[1;35m'
WHITE='\033[1;37m'
ORANGE='\033[1;38;5;208m'
PURPLE='\033[1;38;5;135m'
PINK='\033[1;38;5;201m'
LIME='\033[1;38;5;154m'
TEAL='\033[1;38;5;51m'
VIOLET='\033[1;38;5;141m'
GOLD='\033[1;38;5;220m'
CORAL='\033[1;38;5;203m'
AQUA='\033[1;38;5;45m'
EMERALD='\033[1;38;5;48m'
FUCHSIA='\033[1;38;5;198m'
NC='\033[0m' # No Color

# Check if running in Alacritty
if [ -z "$ALACRITTY_WINDOW_ID" ]; then
    if command -v alacritty >/dev/null 2>&1; then
        exec alacritty -e bash -c "$0"
    else
        echo -e "${RED}Alacritty not found. Please install Alacritty to run this script.${NC}"
        exit 1
    fi
fi

# Function to log messages
log_message() {
    local message="$1"
    echo -e "$message" | tee -a "$LOGFILE"
}

# Function to display a spinner with enhanced Unicode
spinner() {
    local pid=$1
    local delay=0.08
    local spinstr='⡿⣟⣯⣷⣾⣽⣻⢿' # Enhanced Unicode spinner for smoother animation
    local message="$2"
    local max_width=60
    local trunc_message="${message:0:$((max_width-4))}"
    tput civis
    while ps -p "$pid" > /dev/null; do
        for ((i=0; i<${#spinstr}; i++)); do
            printf "\r${AQUA}%s${NC} %-${max_width}s" "${spinstr:$i:1}" "$trunc_message"
            sleep "$delay"
        done
    done
    wait "$pid"
    local exit_status=$?
    printf "\r%*s\r" "$((max_width+4))" ""
    tput cnorm
    return $exit_status
}

# Function to check command existence
check_command() {
    command -v "$1" &>/dev/null
}

# Function to print section header
print_header() {
    local message="$1"
    local width=60
    local title_width=$(( ${#message} + 2 ))
    local left_pad=$(( (width - title_width) / 2 ))
    local right_pad=$(( width - title_width - left_pad ))
    log_message "${GOLD}┌$(printf '─%.0s' $(seq 1 $width))┐${NC}"
    log_message "${GOLD}│$(printf '%*s' "$left_pad" '') ${FUCHSIA}${message}${NC} $(printf '%*s' "$right_pad" '')│${NC}"
    log_message "${GOLD}└$(printf '─%.0s' $(seq 1 $width))┘${NC}"
}

# Function to authenticate sudo upfront
authenticate_sudo() {
    log_message "${YELLOW}Authenticating sudo credentials...${NC}"
    sudo -v
    if [ $? -ne 0 ]; then
        log_message "${RED}Sudo authentication failed. Exiting.${NC}"
        exit 1
    fi
}

# Function to perform updates
perform_updates() {
    print_header "System Update Process"
    log_message "${CYAN}Starting system updates...${NC}"

    # Update APT
    if check_command apt-get; then
        print_header "APT Package Updates"
        sudo apt-get update -y 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Updating package lists"
        if [ $? -ne 0 ]; then
            log_message "${RED}APT update failed. Check log for details.${NC}"
            return 1
        fi
        sudo apt-get upgrade -y 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Installing package upgrades"
        if [ $? -ne 0 ]; then
            log_message "${RED}APT upgrade failed. Check log for details.${NC}"
            return 1
        fi
        sudo apt-get autoremove -y 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Removing unused packages"
        sudo apt-get autoclean -y 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Cleaning package cache"
        log_message "${LIME}APT updates completed successfully.${NC}"
    else
        log_message "${RED}APT not found. Skipping APT updates.${NC}"
    fi

    # Update Snap
    if check_command snap; then
        print_header "Snap Package Updates"
        sudo snap refresh 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Refreshing Snap packages"
        if [ $? -ne 0 ]; then
            log_message "${RED}Snap refresh failed. Check log for details.${NC}"
        else
            log_message "${LIME}Snap updates completed successfully.${NC}"
        fi
    else
        log_message "${RED}Snap not installed. Skipping Snap updates.${NC}"
    fi

    # Update Flatpak
    if check_command flatpak; then
        print_header "Flatpak Package Updates"
        flatpak update -y 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Updating Flatpak packages"
        if [ $? -ne 0 ]; then
            log_message "${RED}Flatpak update failed. Check log for details.${NC}"
        else
            log_message "${LIME}Flatpak updates completed successfully.${NC}"
        fi
    else
        log_message "${RED}Flatpak not installed. Skipping Flatpak updates.${NC}"
    fi

    # Update firmware
    if check_command fwupdmgr; then
        print_header "Firmware Updates"
        sudo fwupdmgr refresh 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Refreshing firmware metadata"
        sudo fwupdmgr update 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Applying firmware updates"
        if [ $? -ne 0 ]; then
            log_message "${RED}Firmware update failed. Check log for details.${NC}"
        else
            log_message "${LIME}Firmware updates completed successfully.${NC}"
        fi
    else
        log_message "${RED}fwupdmgr not installed. Skipping firmware updates.${NC}"
    fi

    # Update Rust
    if check_command rustup; then
        print_header "Rust Updates"
        rustup update 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Updating Rust toolchain"
        if [ $? -ne 0 ]; then
            log_message "${RED}Rust update failed. Check log for details.${NC}"
        else
            log_message "${LIME}Rust updates completed successfully.${NC}"
        fi
    else
        log_message "${RED}rustup not installed. Skipping Rust updates.${NC}"
    fi

    # Update Python (pip)
    if check_command pip; then
        print_header "Python Package Updates"
        pip install --upgrade pip 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Upgrading pip"
        pip list --outdated --format=json | jq -r '.[] | "\(.name)==\(.latest_version)"' | xargs -n1 pip install --upgrade 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Updating Python packages"
        if [ $? -ne 0 ]; then
            log_message "${RED}Python package update failed. Check log for details.${NC}"
        else
            log_message "${LIME}Python package updates completed successfully.${NC}"
        fi
    else
        log_message "${RED}pip not installed. Skipping Python updates.${NC}"
    fi

    # Update Node.js (nvm)
    if check_command nvm; then
        print_header "Node.js Updates"
        # Source nvm script to ensure it's available
        [ -s "$HOME/.nvm/nvm.sh" ] && \. "$HOME/.nvm/nvm.sh"
        nvm install node --reinstall-packages-from=node 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Updating Node.js and npm packages"
        if [ $? -ne 0 ]; then
            log_message "${RED}Node.js update failed. Check log for details.${NC}"
        else
            log_message "${LIME}Node.js updates completed successfully.${NC}"
        fi
    else
        log_message "${RED}nvm not installed. Skipping Node.js updates.${NC}"
    fi

    # Update Plymouth
    print_header "Plymouth Updates"
    local plymouth_updated=false
    local source_dir="/usr/share/HackerOS/ICONS/Plymouth-Icons"
    local config_dir="/usr/share/HackerOS/Config-Files"

    for file in \
        "$source_dir/ubuntu-logo.png:/usr/share/plymouth" \
        "$source_dir/watermark.png:/usr/share/plymouth/themes/spinner" \
        "$config_dir/org.gnome.Software.desktop:/usr/share/applications" \
        "$source_dir/bgrt-fallback.png:/usr/share/plymouth/themes/spinner"; do
        src=${file%%:*}
        dest=${file##*:}
        if [ -f "$src" ]; then
            sudo mkdir -p "$dest" 2>&1 | tee -a "$LOGFILE"
            sudo cp -f "$src" "$dest" 2>&1 | tee -a "$LOGFILE" &
            spinner $! "Copying $(basename "$src")"
            plymouth_updated=true
        else
            log_message "${RED}File $(basename "$src") not found in $source_dir.${NC}"
        fi
    done
    $plymouth_updated && log_message "${LIME}Plymouth updates completed successfully.${NC}"
}

# Function to display menu with reduced options
show_menu() {
    while true; do
        print_header "Update Options"
        log_message "${CYAN}Available actions:${NC}"
        log_message "${WHITE}  e) Exit          Close the terminal${NC}"
        log_message "${WHITE}  r) Reboot        Reboot the system${NC}"
        log_message "${WHITE}  s) Shutdown      Shut down the system${NC}"
        log_message "${WHITE}  l) Log out       Log out of the current session${NC}"
        log_message "${WHITE}  t) Try again     Rerun the update process${NC}"
        printf "${ORANGE}⤷ Select an option [e/r/s/l/t]: ${NC}"
        read -n 1 choice
        echo

        case "${choice,,}" in
            e)
                log_message "${GREEN}Exiting update mode...${NC}"
                exit 0
                ;;
            r)
                log_message "${YELLOW}Initiating system reboot...${NC}"
                sudo reboot
                ;;
            s)
                log_message "${YELLOW}Initiating system shutdown...${NC}"
                sudo shutdown -h now
                ;;
            l)
                log_message "${YELLOW}Logging out of session...${NC}"
                if gnome-session-quit --no-prompt 2>&1 | tee -a "$LOGFILE"; then
                    log_message "${GREEN}Logout successful.${NC}"
                else
                    log_message "${RED}Logout failed. Check log for details.${NC}"
                fi
                ;;
            t)
                log_message "${YELLOW}Restarting update process...${NC}"
                authenticate_sudo
                perform_updates
                ;;
            *)
                log_message "${RED}Invalid option. Please use e, r, s, l, or t.${NC}"
                ;;
        esac
    done
}

# Main execution
{
    print_header "System Update Script"
    log_message "${CYAN}Initializing update process...${NC}"
    authenticate_sudo
    perform_updates
    show_menu
} 2>&1 | tee -a "$LOGFILE"
