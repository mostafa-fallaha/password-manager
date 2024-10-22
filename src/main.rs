use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::home_dir;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use tui::{
    backend::{CrosstermBackend, TermionBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Row, Table},
    Terminal,
};

#[derive(Serialize, Deserialize, Clone)]
struct PasswordEntry {
    service: String,
    email: String,
    username: String,
    password: String,
}

fn save_passwords(entries: &[PasswordEntry], path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, entries)?;
    Ok(())
}

fn load_passwords(path: &str) -> std::io::Result<Vec<PasswordEntry>> {
    // Check if the file exists and is empty
    let file = OpenOptions::new().read(true).open(path);
    if let Ok(file) = file {
        let reader = BufReader::new(file);
        let entries: Vec<PasswordEntry> =
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new());

        // If the file contains no data, return an empty list
        if entries.is_empty() {
            return Ok(Vec::new());
        }
        Ok(entries)
    } else {
        // If the file doesn't exist, treat it as empty
        Ok(Vec::new())
    }
}

static PASSWORD_PATH: Lazy<String> = Lazy::new(|| {
    let mut path = home_dir().expect("Could not retrieve home directory");
    path.push(".password_manager");
    path.push("passwords.json");
    path.to_str()
        .expect("Path to string conversion failed")
        .to_string()
});

fn get_password_path() -> &'static str {
    &PASSWORD_PATH
}

fn main() -> Result<(), io::Error> {
    let pass_path = get_password_path();

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut passwords = load_passwords(pass_path).unwrap_or_else(|_| Vec::new());

    // Check if the file was empty and insert the header-like entry if necessary
    if passwords.is_empty() {
        passwords.push(PasswordEntry {
            service: "Service".to_string(),
            email: "Email".to_string(),
            username: "Username".to_string(),
            password: "Password".to_string(),
        });
    }

    terminal.draw(|f| {
        let size = f.size();
        let items: Vec<ListItem> = vec![
            ListItem::new("1. Add Password"),
            ListItem::new("2. Get Passwords"),
            ListItem::new("3. Delete Password"),
        ];

        let list = List::new(items).block(
            Block::default()
                .title("Password Manager")
                .borders(Borders::ALL),
        );

        f.render_widget(list, size);
    })?;

    loop {
        // Handle user input
        if let event::Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('1') => {
                    add_password(&mut passwords, &mut terminal)?;
                }
                KeyCode::Char('2') => {
                    display_passwords(&passwords, &mut terminal)?;
                }
                KeyCode::Char('3') => {
                    delete_password(&mut passwords, &mut terminal)?;
                }
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                } // Quit if 'q' is pressed
                _ => {}
            }
        }
        enable_raw_mode()?;
    }

    save_passwords(&passwords, pass_path)?;
    Ok(())
}

fn add_password(
    passwords: &mut Vec<PasswordEntry>,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    terminal.clear()?;

    // Collect service, username, and password from user
    let service = prompt_user("Enter service: ")?;
    let email = prompt_user("Enter email: ")?;
    let username = prompt_user("Enter username: ")?;
    let password = prompt_user("Enter password: ")?;

    passwords.push(PasswordEntry {
        service,
        email,
        username,
        password,
    });

    println!("Password added successfully.");
    std::thread::sleep(std::time::Duration::from_millis(600));

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        let size = f.size();
        let items: Vec<ListItem> = vec![
            ListItem::new("1. Add Password"),
            ListItem::new("2. Get Passwords"),
            ListItem::new("3. Delete Password"),
        ];

        let list = List::new(items).block(
            Block::default()
                .title("Password Manager")
                .borders(Borders::ALL),
        );

        f.render_widget(list, size);
    })?;

    Ok(())
}

fn display_passwords(
    passwords: &[PasswordEntry],
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> io::Result<()> {
    terminal.clear()?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.clear()?;
    // terminal.show_cursor()?;

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = RawTerminal::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal2 = Terminal::new(backend)?;

    // terminal2.clear()?;

    // Display all passwords in a tabular format
    terminal2.draw(|f| {
        let rows: Vec<Row> = passwords
            .iter()
            .map(|entry| {
                Row::new(vec![
                    entry.service.clone(),
                    entry.email.clone(),
                    entry.username.clone(),
                    entry.password.clone(),
                ])
            })
            .collect();

        let table = Table::new(rows)
            .block(
                Block::default()
                    .title("Stored Passwords")
                    .borders(Borders::ALL),
            )
            .widths(&[
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        f.render_widget(table, layout[0]);
    })?;

    let stdin = io::stdin();
    for c in stdin.keys() {
        match c? {
            Key::Char('q') => {
                terminal2.clear()?;
                terminal.draw(|f| {
                    let size = f.size();
                    let items: Vec<ListItem> = vec![
                        ListItem::new("1. Add Password"),
                        ListItem::new("2. Get Passwords"),
                        ListItem::new("3. Delete Password"),
                    ];

                    let list = List::new(items).block(
                        Block::default()
                            .title("Password Manager")
                            .borders(Borders::ALL),
                    );

                    f.render_widget(list, size);
                })?;
                break;
            } // Quit if 'q' is pressed
            _ => {}
        }
    }

    Ok(())
}

fn delete_password(
    passwords: &mut Vec<PasswordEntry>,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    terminal.clear()?;

    let service = prompt_user("Enter service to delete: ")?;
    passwords.retain(|entry| entry.service != service);
    println!("Password deleted successfully (if it existed).");

    std::thread::sleep(std::time::Duration::from_millis(600));

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        let size = f.size();
        let items: Vec<ListItem> = vec![
            ListItem::new("1. Add Password"),
            ListItem::new("2. Get Passwords"),
            ListItem::new("3. Delete Password"),
        ];

        let list = List::new(items).block(
            Block::default()
                .title("Password Manager")
                .borders(Borders::ALL),
        );

        f.render_widget(list, size);
    })?;

    Ok(())
}

fn prompt_user(prompt: &str) -> io::Result<String> {
    // Prompt user for input and read from stdin
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
