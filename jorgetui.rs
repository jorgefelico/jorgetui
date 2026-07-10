use crossterm::event::{self, Event, KeyEventKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table};
use serde::Deserialize;
use std::fs;
use std::process::Command;

#[derive(Deserialize)]
struct Config {
    system_packages: Vec<ConfigPackageEntry>,
}

#[derive(Deserialize)]
struct ConfigPackageEntry {
    name: String,
    package_manager: String,
}
#[derive(Deserialize)]
struct LinuxPackage {
    name: String,
    package_manager: PackageManager,
    status: Status,
}

#[derive(Deserialize)]
enum PackageManager {
    Pacman,
    Yay,
}

#[derive(Deserialize)]
enum Status {
    Installed,
    NotInstalled,
    FailedInstall,
}

fn main() -> color_eyre::Result<()> {
    let mut linux_packages: Vec<LinuxPackage> = get_packages_from_config();
    check_if_linux_packages_are_installed(&mut linux_packages);

    color_eyre::install()?;
    let mut package_row: usize = 0;
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| app(frame, &linux_packages, &mut package_row))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        crossterm::event::KeyCode::Char('i')
                        | crossterm::event::KeyCode::Char('I') => {
                            if package_row < linux_packages.len() {
                                install_package(&linux_packages[package_row]);
                            }
                        }
                        crossterm::event::KeyCode::Up => {
                            if package_row > 0 {
                                package_row -= 1;
                            }
                        }
                        crossterm::event::KeyCode::Down => {
                            if package_row < linux_packages.len() {
                                package_row += 1;
                            }
                        }
                        crossterm::event::KeyCode::Esc => break Ok::<(), std::io::Error>(()),
                        _ => {}
                    }
                }
            }
        }
    })?;
    Ok(())
}

fn get_packages_from_config() -> Vec<LinuxPackage> {
    let toml_raw = fs::read_to_string("config.toml").unwrap();
    let data: Config = toml::from_str(&toml_raw).unwrap();
    Vec::from_iter(data.system_packages.iter().map(|f| LinuxPackage {
        name: f.name.clone(),
        package_manager: match f.package_manager.as_str() {
            "pacman" => PackageManager::Pacman,
            "yay" => PackageManager::Yay,
            _ => PackageManager::Pacman,
        },
        status: Status::NotInstalled,
    }))
}

fn exec_command(command: String, args: Vec<&str>) -> bool {
    match Command::new(command).args(args).output() {
        Ok(output) => {
            let _stdout = String::from_utf8_lossy(&output.stdout);
            output.status.success()
        }
        Err(_) => false,
    }
}

fn check_if_linux_packages_are_installed(linux_packages: &mut Vec<LinuxPackage>) {
    for package in linux_packages {
        match exec_command(
            String::from(match package.package_manager {
                PackageManager::Pacman => "pacman",
                PackageManager::Yay => "yay",
            }),
            vec!["-Q", &package.name],
        ) {
            true => package.status = Status::Installed,
            false => package.status = Status::NotInstalled,
        }
    }
}

fn app(frame: &mut Frame, linux_packages: &[LinuxPackage], package_row: &mut usize) {
    render(frame, linux_packages, package_row);
}

fn linux_package_list(frame: &mut Frame, linux_packages: &[LinuxPackage], package_row: &mut usize) {
    let widths = [
        Constraint::Length(20),
        Constraint::Length(15),
        Constraint::Length(12),
    ];

    let status_str = |s: &Status| -> &'static str {
        match s {
            Status::Installed => "Installed",
            Status::NotInstalled => "Not Installed",
            Status::FailedInstall => "Failed",
        }
    };

    let rows = linux_packages.iter().enumerate().map(|(idx, p)| {
        let status_text = status_str(&p.status);
        let style = match p.status {
            Status::Installed => Style::default().fg(Color::Green),
            Status::NotInstalled => Style::default().fg(Color::Yellow),
            Status::FailedInstall => Style::default().fg(Color::Red),
        };
        let (action_text, action_style) = match p.status {
            Status::Installed => (
                "[ Uninstall ]",
                Style::default().fg(Color::Black).bg(Color::Red),
            ),
            _ => (
                "[ Install ]",
                Style::default().fg(Color::Black).bg(Color::Green),
            ),
        };
        let action_style_selected =
            Style::default()
                .fg(Color::White)
                .bg(if matches!(p.status, Status::Installed) {
                    Color::Red
                } else {
                    Color::Green
                });
        Row::new(vec![
            Cell::from(p.name.as_str()),
            Cell::from(status_text).style(style),
            Cell::from(action_text).style(if idx == *package_row {
                action_style_selected
            } else {
                action_style
            }),
        ])
        .style(if idx == *package_row {
            Style::default().bg(Color::Blue)
        } else {
            Style::default()
        })
    });

    let table = Table::new(rows, widths).block(Block::bordered().title("Linux Packages"));

    let vertical_section = Layout::new(
        Direction::Vertical,
        [Constraint::Percentage(80), Constraint::Percentage(20)],
    )
    .split(frame.area());

    let table_section = Layout::new(Direction::Horizontal, [Constraint::Percentage(100)])
        .split(vertical_section[0]);
    frame.render_widget(table, table_section[0]);

    let buttons =
        Paragraph::new("Hello").block(Block::bordered().border_style(Color::Rgb(25, 25, 25)));
    frame.render_widget(buttons, vertical_section[1]);
}

fn install_package(package: &LinuxPackage) {
    let command = match package.package_manager {
        PackageManager::Pacman => "pacman".to_string(),
        PackageManager::Yay => "yay".to_string(),
    };
    exec_command(command, vec!["-S", &package.name]);
}

fn render(frame: &mut Frame, linux_packages: &[LinuxPackage], package_row: &mut usize) {
    linux_package_list(frame, linux_packages, package_row);
}
