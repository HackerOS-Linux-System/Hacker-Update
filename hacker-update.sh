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
NC='\033[0m' # No Color

# Function to log messages
log_message() {
    local message="$1"
    echo -e "$message" | tee -a "$LOGFILE"
}

# Function to display a spinner with enhanced Unicode
spinner() {
    local pid=$1
    local delay=0.08
    local spinstr='⡿⣟⣯⣷⣾⣽⣻⢿⡷⣧⣗⣮⢾⡽⣫⣹⢷⣯⣟' # Longer Unicode spinner
    local message="$2"
    local max_width=60
    local trunc_message="${message:0:$((max_width-4))}"
    tput civis
    while ps -p "$pid" > /dev/null; do
        for ((i=0; i<${#spinstr}; i++)); do
            printf "\r${PURPLE}%s${NC} %-${max_width}s" "${spinstr:$i:1}" "$trunc_message"
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
    log_message "${ORANGE}┌$(printf '─%.0s' $(seq 1 $width))┐${NC}"
    log_message "${ORANGE}│$(printf '%*s' "$left_pad" '') ${PINK}${message}${NC} $(printf '%*s' "$right_pad" '')│${NC}"
    log_message "${ORANGE}└$(printf '─%.0s' $(seq 1 $width))┘${NC}"
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

    # Update Pacman (Arch Linux package manager)
    if check_command pacman; then
        print_header "Pacman Package Updates"
        sudo pacman -Syu --noconfirm 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Updating and upgrading packages"
        if [ $? -ne 0 ]; then
            log_message "${RED}Pacman update failed. Check log for details.${NC}"
            return 1
        fi
        sudo pacman -Sc --noconfirm 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Cleaning package cache"
        log_message "${LIME}Pacman updates completed successfully.${NC}"
    else
        log_message "${RED}Pacman not found. Skipping Pacman updates.${NC}"
    fi

    # Update Flatpak (if installed)
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

    # Update Plymouth
    print_header "Plymouth Updates"
    local plymouth_updated=false
    local source_file="/usr/share/HackerOS/ICONS/Plymouth-Icons/watermark.png"
    local dest_dir="/usr/share/plymouth/themes/spinner"
    local dest_file="$dest_dir/watermark.png"

    if [ -f "$source_file" ]; then
        sudo mkdir -p "$dest_dir" 2>&1 | tee -a "$LOGFILE"
        sudo cp -f "$source_file" "$dest_file" 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Copying watermark.png"
        plymouth_updated=true
    else
        log_message "${RED}File watermark.png not found in /usr/share/HackerOS/ICONS/Plymouth-Icons.${NC}"
    fi
    $plymouth_updated && log_message "${LIME}Plymouth updates completed successfully.${NC}"
}

# Function to check disk space
check_disk_space() {
    print_header "Disk Space Check"
    log_message "${CYAN}Checking disk space...${NC}"
    df -h 2>&1 | tee -a "$LOGFILE" &
    spinner $! "Gathering disk usage information"
    log_message "${LIME}Disk space check completed.${NC}"
}

# Function to clear package cache
clear_package_cache() {
    print_header "Clear Package Cache"
    if check_command pacman; then
        sudo pacman -Sc --noconfirm 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Clearing Pacman package cache"
        log_message "${LIME}Pacman cache cleared successfully.${NC}"
    else
        log_message "${RED}Pacman not found. Skipping cache clear.${NC}"
    fi
}

# Function to sync system time
sync_system_time() {
    print_header "System Time Synchronization"
    if check_command timedatectl; then
        sudo timedatectl set-ntp true 2>&1 | tee -a "$LOGFILE" &
        spinner $! "Synchronizing system time"
        log_message "${LIME}System time synchronized successfully.${NC}"
    else
        log_message "${RED}timedatectl not found. Skipping time sync.${NC}"
    fi
}

# Function to display menu with more options
show_menu() {
    while true; do
        print_header "Update Options"
        log_message "${CYAN}Available actions:${NC}"
        log_message "${WHITE}  e) Exit          Close the terminal${NC}"
        log_message "${WHITE}  r) Reboot        Reboot the system${NC}"
        log_message "${WHITE}  s) Shutdown      Shut down the system${NC}"
        log_message "${WHITE}  l) Log out       Log out of the current session${NC}"
        log_message "${WHITE}  t) Try again     Rerun the update process${NC}"
        log_message "${WHITE}  v) View log      View the update log${NC}"
        log_message "${WHITE}  d) Disk space    Check disk space usage${NC}"
        log_message "${WHITE}  c) Clear cache   Clear package cache${NC}"
        log_message "${WHITE}  n) Sync time     Synchronize system time${NC}"
        printf "${ORANGE}⤷ Select an option [e/r/s/l/t/v/d/c/n]: ${NC}"
        read -n 1 choice
        echo

        case "${choice,,}" in
            e)
                log_message "${GREEN}Exiting for update mode...${NC}"
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
            v)
                log_message "${CYAN}Viewing update log...${NC}"
                if [ -f "$LOGFILE" ]; then
                    less "$LOGFILE"
                else
                    log_message "${RED}Log file not found.${NC}"
                fi
                ;;
            d)
                log_message "${YELLOW}Checking disk space...${NC}"
                check_disk_space
                ;;
            c)
                log_message "${YELLOW}Clearing package cache...${NC}"
                authenticate_sudo
                clear_package_cache
                ;;
            n)
                log_message "${YELLOW}Synchronizing system time...${NC}"
                authenticate_sudo
                sync_system_time
                ;;
            *)
                log_message "${RED}Invalid option. Please use e, r, s, l, t, v, d, c, or n.${NC}"
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
