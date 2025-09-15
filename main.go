package main

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/progress"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/table"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// Definicje stylów z rozszerzoną paletą kolorów i gradientami
var (
	// Styl nagłówka z neonowym gradientem
	headerStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.AdaptiveColor{Light: "#00FFFF", Dark: "#00B7EB"}). // Neon cyan gradient start
			Background(lipgloss.Color("#0A0A23")).                           // Dark navy
			Padding(2, 4).
			Align(lipgloss.Center).
			Border(lipgloss.HeavyBorder(), true).
			BorderForeground(lipgloss.AdaptiveColor{Light: "#FF00FF", Dark: "#FF55FF"}) // Neon magenta gradient

	// Styl paneli z cieniem
	panelStyle = lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder(), true).
			BorderForeground(lipgloss.Color("#00FF7F")). // Neon spring green
			Padding(1, 3).
			Background(lipgloss.Color("#0F0F23")).
			Width(120).
			Margin(0, 0, 1, 0) // Dodaj cień poprzez margines

	// Styl błędów z pulsującym czerwonym
	errorStyle = lipgloss.NewStyle().
			Border(lipgloss.ThickBorder(), true).
			BorderForeground(lipgloss.Color("#FF5555")). // Neon red
			Padding(1, 2).
			Background(lipgloss.Color("#2A1A1A")).
			Foreground(lipgloss.Color("#FF5555"))

	// Styl sukcesu z zielonym glow
	successStyle = lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder(), true).
			BorderForeground(lipgloss.Color("#55FF55")). // Neon green
			Padding(1, 2).
			Foreground(lipgloss.Color("#55FF55"))

	// Dodatkowe style dla kolorów
	warningStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFFF55")) // Neon yellow
	infoStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("#55FFFF")) // Neon cyan

	// Style dla sekcji z więcej kolorami
	aptStyle     = lipgloss.NewStyle().Foreground(lipgloss.Color("#FF55FF")) // Neon magenta
	flatpakStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFD700")) // Neon gold yellow
	snapStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("#00FFFF")) // Neon cyan
	fwStyle      = lipgloss.NewStyle().Foreground(lipgloss.Color("#32CD32")) // Neon lime green
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
		Name: "APT System Update",
		Commands: []Command{
			{"APT Update", "sudo apt update", aptStyle},
			{"APT Upgrade", "sudo apt upgrade -y", aptStyle},
			{"APT Autoremove", "sudo apt autoremove -y", aptStyle},
			{"APT Autoclean", "sudo apt autoclean", aptStyle},
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

// Funkcja nagłówka z rozszerzonym gradientem i animowanym efektem (tekstowym)
func printHeader() string {
	gradientTitle := lipgloss.NewStyle().
		Gradient(true).
		Foreground(lipgloss.AdaptiveColor{Light: "#FF00FF", Dark: "#00FFFF"}).
		Bold(true).
		Render("HackerOS Update Utility v1.1.0")
	subtitle := lipgloss.NewStyle().
		Italic(true).
		Foreground(lipgloss.Color("#7FFFD4")). // Neon aquamarine
		Render("Activating Enhanced Cyber Update Protocol...")
	content := lipgloss.JoinVertical(lipgloss.Center, gradientTitle, "\n", subtitle)
	return headerStyle.Render(content)
}

// Funkcja sekcji z gradientowym tłem
func printSectionHeader(name string) string {
	header := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("#E0FFFF")).
		Background(getSectionColor(name)).
		Padding(1, 4).
		Border(lipgloss.DoubleBorder(), true).
		BorderForeground(getSectionColor(name)).
		Render(strings.ToUpper(name))
	return header
}

// Pobieranie koloru sekcji z więcej odcieniami
func getSectionColor(name string) lipgloss.Color {
	switch name {
	case "APT System Update":
		return lipgloss.Color("#FF55FF") // Neon magenta
	case "Flatpak Update":
		return lipgloss.Color("#FFD700") // Neon gold
	case "Snap Update":
		return lipgloss.Color("#00FFFF") // Neon cyan
	case "Firmware Update":
		return lipgloss.Color("#32CD32") // Neon lime
	default:
		return lipgloss.Color("#FFFFFF")
	}
}

// Funkcja wyświetlania tabeli statusów z ikonami i kolorowymi wierszami
func showStatusTable(logs []string) string {
	columns := []table.Column{
		{Title: "Icon", Width: 5},
		{Title: "Section", Width: 20},
		{Title: "Command", Width: 20},
		{Title: "Status", Width: 15},
		{Title: "Output Snippet", Width: 55},
	}
	rows := []table.Row{}
	for _, log := range logs {
		parts := strings.SplitN(log, "|", 4)
		if len(parts) == 4 {
			icon := "✗"
			rowStyle := errorStyle
			if parts[2] == "Success" {
				icon = "✓"
				rowStyle = successStyle
			}
			rows = append(rows, table.Row{icon, parts[0], parts[1], parts[2], parts[3]})
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
		Foreground(lipgloss.Color("#FFD700")) // Gold for headers
	s.Selected = s.Selected.
		Foreground(lipgloss.Color("#FF69B4")). // Hot pink for selected
		Background(lipgloss.Color("#1A1A2E"))
	t.SetStyles(s)
	return panelStyle.Render(t.Render())
}

// Funkcja uruchamiania komendy z sudo
func runCommand(cmd string, color lipgloss.Style) (string, string, bool) {
	args := strings.Fields(cmd)
	command := exec.Command(args[0], args[1:]...)
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
		return snippet, errorStyle.Render(stderr.String()), false
	}
	return snippet, "", true
}

// Model Bubble Tea dla ładniejszego paska postępu z multi-kolor gradientem
type progressModel struct {
	progress progress.Model
	spinner  spinner.Model
	section  string
	cmd      string
	percent  float64
	done     bool
}

func newProgressModel() progressModel {
	return progressModel{
		progress: progress.New(
			progress.WithScaledGradient("#FF00FF", "#00FFFF", "#FFFF00"), // Multi-color gradient: magenta-cyan-yellow
			progress.WithWidth(80),
			progress.WithSolidFill("█"),
		),
		spinner: spinner.New(
			spinner.WithSpinner(spinner.Line),
			spinner.WithStyle(lipgloss.NewStyle().Foreground(lipgloss.Color("#FF4500"))), // Orange red spinner
		),
	}
}

func (m progressModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, tea.Tick(100*time.Millisecond, func(t time.Time) tea.Msg {
		return tickMsg{}
	}))
}

type tickMsg struct{}

func (m progressModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg.(type) {
	case tickMsg:
		if m.percent < 1.0 {
			m.percent += 0.05 // Wolniejsza animacja dla lepszego efektu
			return m, tea.Tick(100*time.Millisecond, func(t time.Time) tea.Msg {
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
	return lipgloss.JoinHorizontal(lipgloss.Top,
		m.spinner.View(),
		" ",
		infoStyle.Render(m.section+": "+m.cmd),
		" ",
		m.progress.ViewAs(m.percent),
		" ",
		warningStyle.Render(fmt.Sprintf("%.0f%%", m.percent*100)),
	)
}

// Model Bubble Tea dla menu z animacjami (spinner podczas ładowania opcji)
type menuModel struct {
	choices  []string
	cursor   int
	selected string
	spinner  spinner.Model
	animating bool
}

func initialMenuModel() menuModel {
	return menuModel{
		choices: []string{"Exit", "Shutdown", "Reboot", "Show Logs"},
		spinner: spinner.New(
			spinner.WithSpinner(spinner.Dot),
			spinner.WithStyle(lipgloss.NewStyle().Foreground(lipgloss.Color("#00FF7F"))), // Spring green spinner
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
	s.WriteString(panelStyle.Render(
		lipgloss.JoinVertical(lipgloss.Center,
			successStyle.Render("Update Process Completed Successfully"),
			lipgloss.NewStyle().Foreground(lipgloss.Color("#E0FFFF")).Render("Select Your Next Action:"),
			lipgloss.NewStyle().Foreground(lipgloss.Color("#FFFF55")).Render("(E)xit | (S)hutdown | (R)eboot | (H) Show Logs"),
		),
	))
	s.WriteString("\n")
	if m.animating {
		s.WriteString(m.spinner.View() + " Loading Options...\n")
	}
	for i, choice := range m.choices {
		cursor := " "
		if m.cursor == i {
			cursor = "> "
		}
		color := lipgloss.Color("#FFFFFF")
		if m.cursor == i {
			color = lipgloss.Color("#55FF55") // Neon green for selected
		}
		s.WriteString(lipgloss.NewStyle().Foreground(color).Render(cursor + choice + "\n"))
	}
	return s.String()
}

// Główna funkcja
func main() {
	fmt.Println(printHeader())
	fmt.Println()

	var logs []string
	for _, section := range sections {
		fmt.Println(printSectionHeader(section.Name))
		for _, cmd := range section.Commands {
			// Ładniejszy pasek postępu
			pm := newProgressModel()
			pm.section = section.Name
			pm.cmd = cmd.Name
			p := tea.NewProgram(pm)
			go func(cmd Command) {
				snippet, stderr, success := runCommand(cmd.Cmd, cmd.Color)
				logStatus := "Success"
				if !success {
					logStatus = "Failed"
				}
				logs = append(logs, fmt.Sprintf("%s|%s|%s|%s", section.Name, cmd.Name, logStatus, snippet+stderr))
			}(cmd)
			if _, err := p.Run(); err != nil {
				fmt.Println(errorStyle.Render("Error: " + err.Error()))
				os.Exit(1)
			}
			snippet, stderr, success := runCommand(cmd.Cmd, cmd.Color)
			fmt.Println(snippet)
			if stderr != "" {
				fmt.Println(stderr)
			}
			statusPanel := successStyle
			if !success {
				statusPanel = errorStyle
			}
			fmt.Println(statusPanel.Render(cmd.Name + ": " + func() string {
				if success {
					return "Completed"
				}
				return "Failed"
			}()))
			fmt.Println()
		}
		fmt.Println(successStyle.Render(section.Name + " Completed Successfully"))
		fmt.Println()
		time.Sleep(500 * time.Millisecond)
	}

	// Wyświetlanie ładniejszej tabeli statusów
	fmt.Println(showStatusTable(logs))
	fmt.Println()

	// Menu z animacjami
	p := tea.NewProgram(initialMenuModel())
	m, err := p.Run()
	if err != nil {
		fmt.Println(errorStyle.Render("Error running menu: " + err.Error()))
		os.Exit(1)
	}
	model := m.(menuModel)
	switch model.selected {
	case "Exit":
		fmt.Println(panelStyle.Render("Exiting Update Utility"))
	case "Shutdown":
		fmt.Println(panelStyle.Render("Shutting Down System"))
		runCommand("sudo poweroff", successStyle)
	case "Reboot":
		fmt.Println(panelStyle.Render("Rebooting System"))
		runCommand("sudo reboot", successStyle)
	case "Show Logs":
		fmt.Println(showStatusTable(logs))
	}
}
