use std::{path::Path, process::exit};

use gio::Cancellable;
use glib::SpawnFlags;
use gtk::{
    Box, BoxBuilder, ContainerExt, Label, LabelBuilder, Orientation, ScrolledWindow,
    ScrolledWindowBuilder, WidgetExt,
};
use vte::{PtyFlags, Terminal, TerminalExt};

#[derive(Clone)]
pub struct MyTerminal {
    pub widget: Box,
    pub label: Label,
    pub terminal: Terminal,
    pub scrolled_window: ScrolledWindow,
    last_command: String,
    last_dir: String,
}

impl MyTerminal {
    pub fn new() -> Self {
        let widget = BoxBuilder::new().orientation(Orientation::Vertical).build();
        let label = LabelBuilder::new().label("MyTerminal").build();
        let terminal = Terminal::new();
        terminal.set_property_expand(true);
        let scrolled_window = ScrolledWindowBuilder::new()
            .min_content_height(100)
            .min_content_width(120)
            .build();
        scrolled_window.add(&terminal);

        widget.add(&label);
        widget.add(&scrolled_window);

        Self {
            widget,
            label,
            scrolled_window,
            terminal,
            last_command: "".to_string(),
            last_dir: "".to_string(),
        }
    }

    pub fn spawn_command(&mut self, command: &str, directory: &str) {
        let shell = get_shell();
        let command_path = [Path::new(&shell), Path::new("-c"), Path::new(command)];

        self.terminal
            .spawn_sync(
                PtyFlags::DEFAULT,
                Some(directory),
                &command_path,
                &[],
                SpawnFlags::DEFAULT,
                Some(&mut || {}),
                Some(&Cancellable::new()),
            )
            .unwrap();

        self.last_command = command.to_string();
        self.last_dir = directory.to_string();
    }

    pub fn spawn_shell(&mut self, directory: &str) {
        let shell = get_shell();

        self.terminal
            .spawn_sync(
                PtyFlags::DEFAULT,
                Some(directory),
                &[Path::new(&shell)],
                &[],
                SpawnFlags::DEFAULT,
                Some(&mut || {}),
                Some(&Cancellable::new()),
            )
            .unwrap();

        self.terminal.connect_child_exited(|_, _| exit(0));
    }

    pub fn restart(&self) {
        let shell = get_shell();
        let command = "clear && ".to_string() + &self.last_command;
        let command_path = [Path::new(&shell), Path::new("-c"), Path::new(&command)];

        self.terminal
            .spawn_sync(
                PtyFlags::DEFAULT,
                Some(&self.last_dir),
                &command_path,
                &[],
                SpawnFlags::DEFAULT,
                Some(&mut || {}),
                Some(&Cancellable::new()),
            )
            .unwrap();
    }
}

fn get_shell() -> String {
    env!("SHELL").to_string()
}
