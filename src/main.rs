// macro that returns the path to the repository
macro_rules! repo {
    () => {
        &args().nth(1).unwrap()
    };
}

use std::{
    env::args,
    process::{exit, Command},
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

use gio::{prelude::ApplicationExtManual, traits::ApplicationExt};

use glib::PRIORITY_DEFAULT;
use gtk::prelude::{ContainerExt, GtkWindowExt, LabelExt, PanedExt, WidgetExt};

mod myterminal;
use ignore::WalkBuilder;
use myterminal::MyTerminal;
use notify::{watcher, RecursiveMode, Watcher};

const GIT_LOG: &str = "git log --reverse --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"; // from https://ma.ttias.be/pretty-git-log-in-one-line/
const GIT_BRANCH: &str = "git branch -a";
const GIT_STATUS: &str = "git status --short";

const WAIT_TIME: Duration = Duration::from_secs(1);

fn build_ui(application: &gtk::Application) {
    let repo_path = repo!();
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("gish - a git shell");
    window.set_default_size(1000, 600);

    let mut main_terminal = MyTerminal::new();
    main_terminal.spawn_shell(repo_path);
    main_terminal.widget.set_expand(true);

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

    //packing in vpaned for draging
    let lower_right_paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    lower_right_paned.pack1(&main_terminal.widget, true, false);
    lower_right_paned.pack2(&git_branch_terminal.widget, false, false);

    let left_paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    left_paned.pack1(&git_status_terminal.widget, false, false);
    left_paned.pack2(&lower_right_paned, true, false);

    let main_paned = gtk::Paned::new(gtk::Orientation::Vertical);
    main_paned.pack1(&git_log_terminal.widget, false, false);
    main_paned.pack2(&left_paned, true, false);

    //set focus
    main_terminal.terminal.grab_focus();

    //set icon
    window.set_icon_name(Some("git"));

    window.add(&main_paned);

    window.show_all();

    {
        let (sender, receiver) = glib::MainContext::channel(PRIORITY_DEFAULT);
        let mut last_watch_time = Instant::now();

        thread::spawn(move || {
            let (fake_sender, fake_receiver) = channel(); // just to get events
            let mut watcher = watcher(fake_sender, WAIT_TIME).unwrap();

            for result in WalkBuilder::new(repo!())
                .hidden(true)
                .follow_links(true)
                .max_depth(None)
                .max_filesize(None)
                .build()
            {
                match result {
                    Ok(entry) => watcher
                        .watch(entry.path(), RecursiveMode::NonRecursive)
                        .unwrap(),
                    Err(err) => println!("ERROR: {}", err),
                }
            }

            loop {
                let _ = fake_receiver.recv().unwrap();
                if Instant::now() - last_watch_time > WAIT_TIME {
                    thread::sleep(WAIT_TIME);
                    last_watch_time = Instant::now();
                    sender.send(()).unwrap();
                }
            }
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

fn main() {
    if args().len() < 2 {
        //no path supplied
        println!("Please supply a path to a git repository.");
        exit(2);
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
                let mut flags = gio::ApplicationFlags::HANDLES_OPEN;
                flags.insert(gio::ApplicationFlags::NON_UNIQUE);

                let application = gtk::Application::new(Some("com.kroener.tobias"), flags);

                application.connect_activate(|app| {
                    build_ui(app);
                });

                application.connect_open(|app, _, _| {
                    build_ui(app);
                });

                application.run();
            } else {
                //its not, error
                println!("Directory is not a git repository.");
                exit(1);
            }
        }
    }
}
