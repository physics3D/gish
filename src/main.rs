//macro that returns the path to the repository
macro_rules! repo {
    () => {
        &args().nth(1).unwrap();
    };
}

use std::{
    env::args,
    os::unix::process::CommandExt,
    process::{exit, Command},
    thread,
};

use gio::{prelude::ApplicationExtManual, ApplicationExt};
use glib::PRIORITY_DEFAULT;
use gtk::{ContainerExt, Grid, GridExt, GtkWindowExt, LabelExt, WidgetExt};

use hotwatch::Hotwatch;

mod myterminal;
use myterminal::MyTerminal;

const GIT_LOG: &str = "git log --reverse --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"; // from https://ma.ttias.be/pretty-git-log-in-one-line/
const GIT_BRANCH: &str = "git branch";
const GIT_STATUS: &str = "git status --short";

fn build_ui(application: &gtk::Application) {
    let repo_path = repo!();
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("gish - a git shell");
    window.set_default_size(1000, 600);

    let mut main_terminal = MyTerminal::new();
    main_terminal.spawn_shell(repo_path);
    main_terminal.widget.set_property_expand(true);

    let mut git_log_terminal = MyTerminal::new();
    git_log_terminal.spawn_command(GIT_LOG, repo_path);
    git_log_terminal.terminal.set_can_focus(false);
    git_log_terminal.label.set_text("Git Log");
    git_log_terminal.widget.set_vexpand(false);

    let mut git_status_terminal = MyTerminal::new();
    git_status_terminal.spawn_command(GIT_STATUS, repo_path);
    git_status_terminal.label.set_text("Git Status");
    git_status_terminal.terminal.set_can_focus(false);
    git_status_terminal.widget.set_hexpand(false);

    let mut git_branch_terminal = MyTerminal::new();
    git_branch_terminal.spawn_command(GIT_BRANCH, repo_path);
    git_branch_terminal.label.set_text("Git Branch");
    git_branch_terminal.terminal.set_can_focus(false);
    git_branch_terminal.widget.set_hexpand(false);

    let grid = Grid::new();

    grid.attach(&git_log_terminal.widget, 0, 0, 3, 1);
    grid.attach(&git_status_terminal.widget, 0, 1, 1, 1);
    grid.attach(&main_terminal.widget, 1, 1, 1, 1);
    grid.attach(&git_branch_terminal.widget, 2, 1, 1, 1);

    //set focus
    main_terminal.terminal.grab_focus();

    //set icon
    window.set_icon_name(Some("git"));

    window.add(&grid);

    window.show_all();

    {
        let (sender, receiver) = glib::MainContext::channel(PRIORITY_DEFAULT);
        thread::spawn(move || {
            let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
            hotwatch
                .watch(repo!(), move |_| {
                    let _ = sender.send("");
                })
                .expect("failed to watch directory!");
            loop {}
        });

        receiver.attach(None, move |_| {
            git_status_terminal.restart();
            git_log_terminal.restart();
            git_branch_terminal.restart();

            glib::Continue(true)
        });
    }
}

fn print_help_text() {
    println!("gish: a shell for the git command with additional information about the repository");
    println!();
    println!("Usage: gish [-h|--help] [PATH]");
    println!("Options:");
    println!("\t-h|--help\tPrints this help text");
    println!("\tPATH\t\tA path to a git repository");
}

fn check_git_repo(directory: &str) -> bool {
    let stderr = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(directory)
        .output()
        .expect("failed to check for git repo")
        .stderr;
    stderr == vec![]
}

fn ask_for_path_and_restart() {
    use native_dialog::FileDialog;
    let path = FileDialog::new()
        .set_location("~")
        .show_open_single_dir()
        .unwrap();
    let path = match path {
        Some(path) => path,
        None => exit(1),
    };
    Command::new("/proc/self/exe").arg(path).exec();
}

fn main() {
    // cheat: make gtk believe we don't handle cli args, give them an empty vector as cli args
    // and get the arguments ourselves

    if args().len() < 2 {
        //no path supplied
        //restart
        ask_for_path_and_restart();
    } else {
        //path given
        let arg = args().nth(1).unwrap();
        if arg == "-h" || arg == "--help" {
            // print help text
            print_help_text();
        } else {
            //check if dir is a git repo
            if check_git_repo(repo!()) {
                // run app
                let application =
                    gtk::Application::new(Some("com.kroener.tobias"), Default::default())
                        .expect("Initialization failed...");

                application.connect_activate(|app| {
                    build_ui(app);
                });

                let empty_args = vec!["".to_string()];
                application.run(&empty_args);
            } else {
                //its not, error
                println!("Directory is not a git repository.");
                exit(1);
            }
        }
    }
}
