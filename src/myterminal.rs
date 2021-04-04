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

    fn spawn(&self, command: &str, directory: &str) {
        let command_path: Vec<&Path>;

        if command.starts_with("/bin/") {
            //shell
            command_path = vec![Path::new(command)];
        } else {
            //command
            command_path = vec![Path::new("/bin/sh"), Path::new("-c"), Path::new(command)];
        }

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
    }

    fn store_command(&mut self, command: &str, directory: &str) {
        self.last_command = command.to_string();
        self.last_dir = directory.to_string();
    }

    fn connect_exit(&mut self) {
        self.terminal.connect_child_exited(|_, _| exit(0));
    }

    pub fn spawn_command(&mut self, command: &str, directory: &str) {
        self.spawn(command, directory);
        self.store_command(command, directory);
    }

    pub fn spawn_shell(&mut self, directory: &str) {
        let shell = env!("SHELL");
        self.spawn(shell, directory);
        self.store_command(shell, directory);
        self.connect_exit();
    }

    #[allow(dead_code)]
    pub fn spawn_shell_in_home_dir(&mut self) {
        let home = env!("HOME");
        self.spawn_shell(home);
    }

    pub fn restart(&mut self) {
        self.spawn(
            &("clear; ".to_string() + &self.last_command),
            &self.last_dir,
        );
    }
}
