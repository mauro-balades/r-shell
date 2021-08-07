extern crate ctrlc;
use ctrlc::CtrlC;
use std::process::Command;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::process::Stdio;
use std::process::Child;
use std::path::Path;
use std::env;
use hostname;
use termion::{color, style};

fn getShell() -> String {
    
    let hostnameStr = hostname::get()
                            .unwrap()
                            .into_string()
                            .unwrap();

    let path = env::current_dir().unwrap();
    let format = format!(
        "{}{}{}{}:{}{}{}{}$ ",  
        color::Fg(color::LightGreen), 
        hostnameStr,
        color::Fg(color::LightGreen), 
        style::Reset,
        color::Fg(color::LightBlue), 
        style::Bold,
        path.display(), 
        style::Reset
    );
    return format;
}

fn main() -> std::io::Result<()> {

    CtrlC::set_handler(move || {
        println!("\n\n: TIPE \"exit\" TO EXIT\n");
        print!("{}", getShell());
        stdout().flush();
    });

    loop {

        print!("{}", getShell());
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // must be peekable so we know when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next()  {

            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    let new_dir = args.peekable().peek()
                        .map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                },
                "exit" => return Ok(()),
                command => {
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => { previous_command = Some(output); },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            final_command.wait();
        }

    }
}