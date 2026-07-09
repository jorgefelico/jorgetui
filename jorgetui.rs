use crossterm::event::{self, Event, KeyEventKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Cell, Row, Table};
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
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| app(frame, &linux_packages))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    break Ok::<(), std::io::Error>(());
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

fn app(frame: &mut Frame, linux_packages: &[LinuxPackage]) {
    render(frame, linux_packages);
}

fn linux_package_list(frame: &mut Frame, linux_packages: &[LinuxPackage]) {
    let widths = [Constraint::Length(20), Constraint::Length(15)];

    let status_str = |s: &Status| -> &'static str {
        match s {
            Status::Installed => "Installed",
            Status::NotInstalled => "Not Installed",
            Status::FailedInstall => "Failed",
        }
    };

    let rows = linux_packages.iter().map(|p| {
        let status_text = status_str(&p.status);
        let style = match p.status {
            Status::Installed => Style::default().fg(Color::Green),
            Status::NotInstalled => Style::default().fg(Color::Yellow),
            Status::FailedInstall => Style::default().fg(Color::Red),
        };
        Row::new(vec![
            Cell::from(p.name.as_str()),
            Cell::from(status_text).style(style),
        ])
    });

    let table = Table::new(rows, widths).block(Block::bordered().title("Linux Packages"));

    let vertical_section = Layout::new(
        Direction::Vertical,
        [Constraint::Percentage(80), Constraint::Percentage(20)],
    )
    .split(frame.area());

    Layout::new(Direction::Horizontal, [Constraint::Percentage(50)]).split(vertical_section[0]);

    frame.render_widget(table, vertical_section[0])
}

fn render(frame: &mut Frame, linux_packages: &[LinuxPackage]) {
    linux_package_list(frame, linux_packages);
}
