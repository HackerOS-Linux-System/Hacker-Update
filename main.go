package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/charmbracelet/bubbles/progress"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/table"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// Definicje stylów z paletą neonowych kolorów i efektami cyberpunkowymi
var (
	// Styl nagłówka z neonowym cyan i magenta border
	headerStyle = lipgloss.NewStyle().
	Bold(true).
	Foreground(lipgloss.AdaptiveColor{Light: "#00FFFF", Dark: "#00B7EB"}).
	Background(lipgloss.Color("#0A0A23")).
	Padding(2, 4).
	Align(lipgloss.Center).
	Border(lipgloss.DoubleBorder(), true).
	BorderForeground(lipgloss.AdaptiveColor{Light: "#FF00FF", Dark: "#FF55FF"})

	// Styl paneli z neon green border i dark background
	panelStyle = lipgloss.NewStyle().
	Border(lipgloss.RoundedBorder(), true).
	BorderForeground(lipgloss.Color("#00FF7F")).
	Padding(1, 3).
	Background(lipgloss.Color("#0F0F23")).
	Width(120).
	Margin(1, 1)

	// Styl błędów z pulsującym neon red
	errorStyle = lipgloss.NewStyle().
	Border(lipgloss.ThickBorder(), true).
	BorderForeground(lipgloss.Color("#FF5555")).
	Padding(1, 2).
	Background(lipgloss.Color("#2A1A1A")).
	Foreground(lipgloss.Color("#FF5555"))

	// Styl sukcesu z neon green glow
	successStyle = lipgloss.NewStyle().
	Border(lipgloss.RoundedBorder(), true).
	BorderForeground(lipgloss.Color("#55FF55")).
	Padding(1, 2).
	Foreground(lipgloss.Color("#55FF55"))

	// Dodatkowe style neonowe
	warningStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFFF55"))
	infoStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("#55FFFF"))

	// Style dla sekcji z neonowymi kolorami
	dnfStyle     = lipgloss.NewStyle().Foreground(lipgloss.Color("#FF55FF"))
	flatpakStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFD700"))
	snapStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("#00FFFF"))
	fwStyle      = lipgloss.NewStyle().Foreground(lipgloss.Color("#32CD32"))

	// Styl menu z neon gold border
	menuStyle = lipgloss.NewStyle().
	Border(lipgloss.DoubleBorder(), true).
	BorderForeground(lipgloss.Color("#FFD700")).
	Padding(2, 3).
	Background(lipgloss.Color("#1A1A2E")).
	Width(100).
	Align(lipgloss.Center)

	// Styl dla glitch effect
	glitchStyle = lipgloss.NewStyle().
	Bold(true).
	Italic(true).
	Foreground(lipgloss.Color("#FF00FF")).
	Background(lipgloss.Color("#000000"))

	// Styl dla separatorów z neon linami Unicode
	separatorStyle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("#00FFFF")).
	Render(strings.Repeat("━", 100))

	// Unicode progress bar style
	progressBarStyle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("#55FF55")).
	Background(lipgloss.Color("#1A1A2E"))

	// Unicode blocks dla postępu
	unicodeBlocks = []string{"░", "▒", "▓", "█"}
)

// Struktura poleceń
type Command struct {
	Name  string
	Cmd   string
	Color lipgloss.Style
}

type Section struct {
	Name     string
	Commands []Command
}

var sections = []Section{
	{
		Name: "DNF System Update",
		Commands: []Command{
			{"DNF Update", "sudo dnf update", dnfStyle},
			{"DNF Upgrade", "sudo dnf upgrade -y", dnfStyle},
			{"DNF Autoremove", "sudo dnf autoremove -y", dnfStyle},
		},
	},
	{
		Name: "Flatpak Update",
		Commands: []Command{
			{"Flatpak Update", "flatpak update -y", flatpakStyle},
		},
	},
	{
		Name: "Snap Update",
		Commands: []Command{
			{"Snap Refresh", "sudo snap refresh", snapStyle},
		},
	},
	{
		Name: "Firmware Update",
		Commands: []Command{
			{"Firmware Refresh", "sudo fwupdmgr refresh", fwStyle},
			{"Firmware Update", "sudo fwupdmgr update", fwStyle},
		},
	},
}

// Funkcja nagłówka z neonowym efektem i glitch symulacją
func printHeader() string {
	title := glitchStyle.Render("HACKER UPDATE")
	subtitle := lipgloss.NewStyle().
	Italic(true).
	Foreground(lipgloss.Color("#7FFFD4")).
	Render("Updating HackerOS")
	version := lipgloss.NewStyle().
	Foreground(lipgloss.Color("#FFFF55")).
	Render("Hacker Update - Version 0.9")
	content := lipgloss.JoinVertical(lipgloss.Center, title, "\n", subtitle, "\n", version)
	return headerStyle.Render(content)
}

// Funkcja sekcji z kolorowym tłem i neon border
func printSectionHeader(name string) string {
	icon := getSectionIcon(name)
	header := lipgloss.NewStyle().
	Bold(true).
	Foreground(lipgloss.Color("#E0FFFF")).
	Background(getSectionColor(name)).
	Padding(1, 4).
	Border(lipgloss.DoubleBorder(), true).
	BorderForeground(getSectionColor(name)).
	Render(icon + " " + strings.ToUpper(name))
	return header
}

// Pobieranie ikony Unicode sekcji
func getSectionIcon(name string) string {
	switch name {
		case "DNF System Update":
			return "┃┃"
		case "Flatpak Update":
			return "┣┫"
		case "Snap Update":
			return "┳┳"
		case "Firmware Update":
			return "┻┻"
		default:
			return "══"
	}
}

// Pobieranie koloru sekcji
func getSectionColor(name string) lipgloss.Color {
	switch name {
		case "DNF System Update":
			return lipgloss.Color("#FF55FF")
		case "Flatpak Update":
			return lipgloss.Color("#FFD700")
		case "Snap Update":
			return lipgloss.Color("#00FFFF")
		case "Firmware Update":
			return lipgloss.Color("#32CD32")
		default:
			return lipgloss.Color("#FFFFFF")
	}
}

// Funkcja wyświetlania tabeli statusów z ikonami i kolorami, Unicode icons
func showStatusTable(logs []string) string {
	columns := []table.Column{
		{Title: "▌", Width: 3},
		{Title: "Section", Width: 20},
		{Title: "Command", Width: 20},
		{Title: "Status", Width: 15},
		{Title: "Output", Width: 55},
	}
	rows := []table.Row{}
	for _, log := range logs {
		parts := strings.SplitN(log, "|", 4)
		if len(parts) == 4 {
			icon := "✗"
			style := errorStyle
			if parts[2] == "Success" {
				icon = "✓"
				style = successStyle
			}
			rows = append(rows, table.Row{icon, parts[0], parts[1], parts[2], style.Render(parts[3])})
		}
	}
	t := table.New(
		table.WithColumns(columns),
		       table.WithRows(rows),
		       table.WithFocused(true),
		       table.WithHeight(len(rows)),
	)
	s := table.DefaultStyles()
	s.Header = s.Header.
	BorderStyle(lipgloss.NormalBorder()).
	BorderForeground(lipgloss.Color("#00FFFF")).
	Background(lipgloss.Color("#2A2A4E")).
	Bold(true).
	Foreground(lipgloss.Color("#FFD700"))
	s.Selected = s.Selected.
	Foreground(lipgloss.Color("#FF69B4")).
	Background(lipgloss.Color("#1A1A2E")).
	Bold(true)
	t.SetStyles(s)
	return panelStyle.Render(t.View())
}

// Funkcja uruchamiania komendy z sudo i obsługą przerwania
func runCommand(ctx context.Context, cmd string, color lipgloss.Style) (string, string, bool) {
	args := strings.Fields(cmd)
	command := exec.CommandContext(ctx, args[0], args[1:]...)
	stdout, stderr := &strings.Builder{}, &strings.Builder{}
	command.Stdout = stdout
	command.Stderr = stderr
	err := command.Run()
	output := color.Render(stdout.String())
	snippet := strings.TrimSpace(output)
	if len(snippet) > 50 {
		snippet = snippet[:50] + "..."
	}
	if err != nil {
		if ctx.Err() == context.Canceled {
			return "", errorStyle.Render("Command interrupted by user"), false
		}
		return snippet, errorStyle.Render(stderr.String()), false
	}
	return snippet, "", true
}

// Ulepszony Model Bubble Tea dla paska postępu z Unicode i animacją
type progressModel struct {
	progress  progress.Model
	spinner   spinner.Model
	section   string
	cmd       string
	percent   float64
	done      bool
	blockIdx  int
	ctx       context.Context
}

func newProgressModel(ctx context.Context, section, cmd string) progressModel {
	return progressModel{
		progress: progress.New(
			progress.WithScaledGradient("#FF00FF", "#00FFFF"),
				       progress.WithWidth(60),
				       progress.WithSolidFill(unicodeBlocks[3]),
		),
		spinner: spinner.New(
			spinner.WithSpinner(spinner.MiniDot),
				     spinner.WithStyle(lipgloss.NewStyle().Foreground(lipgloss.Color("#FF4500"))),
		),
		section:  section,
		cmd:      cmd,
		blockIdx: 0,
		ctx:      ctx,
	}
}

func (m progressModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, tea.Tick(50*time.Millisecond, func(t time.Time) tea.Msg {
		return tickMsg{}
	}))
}

type tickMsg struct{}

func (m progressModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg.(type) {
		case tea.KeyMsg:
			if msg.(tea.KeyMsg).String() == "ctrl+c" {
				return m, tea.Quit
			}
		case tickMsg:
			if m.ctx.Err() == context.Canceled {
				return m, tea.Quit
			}
			if m.percent < 1.0 {
				m.percent += 0.02
				m.blockIdx = int(m.percent*100/25) % len(unicodeBlocks)
				return m, tea.Tick(50*time.Millisecond, func(t time.Time) tea.Msg {
					return tickMsg{}
				})
			}
			m.done = true
			return m, tea.Quit
		case spinner.TickMsg:
			var cmd tea.Cmd
			m.spinner, cmd = m.spinner.Update(msg)
			return m, cmd
	}
	return m, nil
}

func (m progressModel) View() string {
	bar := m.progress.ViewAs(m.percent)
	percent := fmt.Sprintf(" %3.0f%% ", m.percent*100)
	return lipgloss.JoinVertical(lipgloss.Center,
				     lipgloss.NewStyle().Foreground(lipgloss.Color("#00FFFF")).Render("▌ "+m.section+": Executing "+m.cmd+" ▐"),
				     progressBarStyle.Render("["+bar+"]"),
				     warningStyle.Render(percent),
	)
}

// Model Bubble Tea dla menu z animacjami i Unicode
type menuModel struct {
	choices   []string
	cursor    int
	selected  string
	spinner   spinner.Model
	animating bool
}

func initialMenuModel() menuModel {
	return menuModel{
		choices: []string{"Exit", "Shutdown", "Reboot", "Show Logs"},
		spinner: spinner.New(
			spinner.WithSpinner(spinner.MiniDot),
				     spinner.WithStyle(lipgloss.NewStyle().Foreground(lipgloss.Color("#00FF7F"))),
		),
		animating: true,
	}
}

func (m menuModel) Init() tea.Cmd {
	return m.spinner.Tick
}

func (m menuModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
		case tea.KeyMsg:
			switch msg.String() {
				case "ctrl+c", "q":
					return m, tea.Quit
				case "up":
					if m.cursor > 0 {
						m.cursor--
					}
				case "down":
					if m.cursor < len(m.choices)-1 {
						m.cursor++
					}
				case "enter":
					m.selected = m.choices[m.cursor]
					m.animating = false
					return m, tea.Quit
			}
				case spinner.TickMsg:
					var cmd tea.Cmd
					m.spinner, cmd = m.spinner.Update(msg)
					return m, cmd
	}
	return m, nil
}

func (m menuModel) View() string {
	s := strings.Builder{}
	s.WriteString(menuStyle.Render(
		lipgloss.JoinVertical(lipgloss.Center,
				      successStyle.Render("Update Protocol Terminated Successfully"),
				      lipgloss.NewStyle().Foreground(lipgloss.Color("#E0FFFF")).Render("Initiate Next Directive:"),
				      lipgloss.NewStyle().Foreground(lipgloss.Color("#FFFF55")).Render("↑↓ Navigate | ↵ Execute"),
		),
	))
	s.WriteString("\n")
	if m.animating {
		s.WriteString(m.spinner.View() + " Scanning Neural Interface...\n\n")
	}
	for i, choice := range m.choices {
		cursor := " "
		if m.cursor == i {
			cursor = "▶▶▶ "
		}
		color := lipgloss.Color("#FFFFFF")
		if m.cursor == i {
			color = lipgloss.Color("#55FF55")
		}
		s.WriteString(lipgloss.NewStyle().Foreground(color).Padding(0, 2).Render(cursor+choice+"\n"))
	}
	return s.String()
}

// Główna funkcja z obsługą Ctrl+C
func main() {
	// Utwórz kontekst z możliwością przerwania
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Kanał do przechwytywania sygnałów Ctrl+C
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	// Gorutyna do obsługi sygnału Ctrl+C
	go func() {
		<-sigChan
		fmt.Println()
		fmt.Println(errorStyle.Render("INTERRUPT DETECTED: Terminating Update Protocol"))
		cancel()
		time.Sleep(500 * time.Millisecond) // Krótka pauza dla wyświetlenia wiadomości
		os.Exit(1)
	}()

	fmt.Println(printHeader())
	fmt.Println(separatorStyle)
	fmt.Println()

	var logs []string
	for _, section := range sections {
		if ctx.Err() == context.Canceled {
			return
		}
		fmt.Println(printSectionHeader(section.Name))
		fmt.Println(separatorStyle)
		for _, cmd := range section.Commands {
			if ctx.Err() == context.Canceled {
				return
			}
			pm := newProgressModel(ctx, section.Name, cmd.Name)
			p := tea.NewProgram(pm, tea.WithContext(ctx))

			// Uruchom komendę w gorutynie
			logChan := make(chan struct {
				snippet string
				stderr  string
				success bool
			})
			go func(cmd Command, sectionName string) {
				snippet, stderr, success := runCommand(ctx, cmd.Cmd, cmd.Color)
				logChan <- struct {
					snippet string
					stderr  string
					success bool
				}{snippet, stderr, success}
			}(cmd, section.Name)

			// Uruchom program bubbletea dla paska postępu
			_, err := p.Run()
			if err != nil {
				fmt.Println(errorStyle.Render("SYSTEM ERROR: " + err.Error()))
				os.Exit(1)
			}
			if ctx.Err() == context.Canceled {
				return
			}

			// Odbierz wyniki komendy
			result := <-logChan
			snippet, stderr, success := result.snippet, result.stderr, result.success
			logs = append(logs, fmt.Sprintf("%s|%s|%s|%s", section.Name, cmd.Name, func() string {
				if success {
					return "Success"
				}
				return "Failed"
			}(), snippet+stderr))

			fmt.Println(snippet)
			if stderr != "" {
				fmt.Println(stderr)
			}
			statusPanel := successStyle
			if !success {
				statusPanel = errorStyle
			}
			fmt.Println(statusPanel.Render("┌─ "+cmd.Name+" ─┐\n│ "+func() string {
				if success {
					return "EXECUTED"
				}
				return "ABORTED"
			}()+" │\n└─────────────┘"))
			fmt.Println()
		}
		fmt.Println(successStyle.Render("┌─ "+section.Name+" TERMINATED ─┐"))
		fmt.Println(separatorStyle)
		time.Sleep(300 * time.Millisecond)
	}

	if ctx.Err() == context.Canceled {
		return
	}

	fmt.Println(panelStyle.Render("NEURAL LOG MATRIX"))
	fmt.Println(showStatusTable(logs))
	fmt.Println(separatorStyle)

	p := tea.NewProgram(initialMenuModel(), tea.WithContext(ctx))
	m, err := p.Run()
	if err != nil {
		fmt.Println(errorStyle.Render("INTERFACE ERROR: " + err.Error()))
		os.Exit(1)
	}
	if ctx.Err() == context.Canceled {
		return
	}

	model := m.(menuModel)
	switch model.selected {
		case "Exit":
			fmt.Println(panelStyle.Render("DISCONNECTING FROM GRID"))
		case "Shutdown":
			fmt.Println(panelStyle.Render("INITIATING SYSTEM PURGE"))
			runCommand(ctx, "sudo poweroff", successStyle)
		case "Reboot":
			fmt.Println(panelStyle.Render("REINITIALIZING CORE"))
			runCommand(ctx, "sudo reboot", successStyle)
		case "Show Logs":
			fmt.Println(panelStyle.Render("ACCESSING ARCHIVE"))
			fmt.Println(showStatusTable(logs))
	}
}
