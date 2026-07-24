//! A tiny `cowsay` clone, built as a learning project.
//!
//! Two ways to run it:
//!   cargo run -- Hello there!      # one-shot: the cow says the arguments
//!   cargo run                      # REPL: type lines, use /commands, /quit to exit
//!
//! Rust concepts on display here: enums + `match`, methods on enums,
//! iterators, string handling, ownership of `String` vs borrowing `&str`,
//! and a couple of external crates (`colored`, `rand`).

use std::io::{self, Write};

use colored::Colorize;
use rand::RngExt;

/// The cow's expression. Each mood maps to a pair of "eyes" and a "tongue",
/// exactly like the flags in the original cowsay (-d, -t, -g, ...).
#[derive(Clone, Copy)]
enum Mood {
    Default,
    Dead,
    Tired,
    Greedy,
    Stoned,
    Wired,
}

impl Mood {
    /// Parse a mood name typed by the user. Returns `None` if unknown,
    /// which lets the caller print a friendly error instead of panicking.
    fn parse(name: &str) -> Option<Mood> {
        let mood = match name {
            "default" | "normal" => Mood::Default,
            "dead" => Mood::Dead,
            "tired" => Mood::Tired,
            "greedy" => Mood::Greedy,
            "stoned" => Mood::Stoned,
            "wired" => Mood::Wired,
            _ => return None,
        };
        Some(mood)
    }

    /// (eyes, tongue) — both are drawn into the cow template below.
    /// `eyes` is always 2 chars; `tongue` is always 2 chars for alignment.
    fn face(self) -> (&'static str, &'static str) {
        match self {
            Mood::Default => ("oo", "  "),
            Mood::Dead => ("xx", "U "),
            Mood::Tired => ("--", "  "),
            Mood::Greedy => ("$$", "  "),
            Mood::Stoned => ("**", "U "),
            Mood::Wired => ("OO", "  "),
        }
    }
}

/// Cow wisdom for the `/fortune` command.
const FORTUNES: &[&str] = &[
    "The grass is always greener where you water it.",
    "Moo happens.",
    "A borrowed value must outlive the reference. So must a good friendship.",
    "Compile-time is just future you thanking present you.",
    "Never trust a mutable state you did not lock.",
    "Chew your data twice, allocate once.",
    "There is no `null` here. Only the quiet dignity of `Option`.",
    "Fear the borrow checker, and it becomes your shield.",
    "Every panic is a lesson wearing a scary mask.",
];

/// Wrap `text` into lines no longer than `width`, breaking on spaces.
/// A single word longer than `width` is left on its own line (no mid-word cut).
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current)); // hand off `current`, leaving it empty
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Build the speech bubble around the (already wrapped) lines.
/// Single line uses `< ... >`; multiple lines use the `/ | \` frame.
fn speech_bubble(lines: &[String]) -> String {
    let inner = lines.iter().map(|l| l.len()).max().unwrap_or(0);

    let mut out = String::new();
    out.push_str(&format!(" {}\n", "_".repeat(inner + 2))); // top border

    if lines.len() == 1 {
        out.push_str(&format!("< {:width$} >\n", lines[0], width = inner));
    } else {
        for (i, line) in lines.iter().enumerate() {
            let (left, right) = match i {
                0 => ('/', '\\'),
                _ if i == lines.len() - 1 => ('\\', '/'),
                _ => ('|', '|'),
            };
            out.push_str(&format!("{} {:width$} {}\n", left, line, right, width = inner));
        }
    }

    out.push_str(&format!(" {}", "-".repeat(inner + 2))); // bottom border
    out
}

/// The cow art, with the eyes and tongue slotted in for the current mood.
fn cow(mood: Mood) -> String {
    let (eyes, tongue) = mood.face();
    format!(
        r"        \   ^__^
         \  ({eyes})\_______
            (__)\       )\/\
             {tongue}||----w |
                ||     ||"
    )
}

/// Assemble the full picture: colored bubble on top, colored cow below.
fn render(message: &str, mood: Mood) -> String {
    let lines = wrap(message, 40);
    let bubble = speech_bubble(&lines);
    format!("{}\n{}", bubble.cyan(), cow(mood).yellow())
}

fn help() {
    println!("{}", "Commands:".bold());
    println!("  {}   change the cow's face", "/mood <name>".green());
    println!(
        "       names: {}",
        "default, dead, tired, greedy, stoned, wired".dimmed()
    );
    println!("  {}       cow speaks a random fortune", "/fortune".green());
    println!("  {}          show this help", "/help".green());
    println!("  {}          leave", "/quit".green());
    println!("  {}  anything else is said by the cow", "<text>".green());
}

fn main() {
    // One-shot mode: `cargo run -- some words` -> the cow says the words and exits.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        println!("{}", render(&args.join(" "), Mood::Default));
        return;
    }

    // Interactive REPL mode.
    println!("{}", "Talk to the cow. /help for commands.".bold());
    let mut mood = Mood::Default;
    let stdin = io::stdin();

    loop {
        print!("{} ", ">".bold());
        io::stdout().flush().unwrap(); // ensure the prompt shows before we block on input

        let mut line = String::new();
        // read_line returns Ok(0) at end-of-input (Ctrl-D or piped EOF).
        if stdin.read_line(&mut line).unwrap() == 0 {
            println!();
            break;
        }
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        // Commands start with '/'. Split into the command and the rest.
        if let Some(rest) = line.strip_prefix('/') {
            let mut parts = rest.splitn(2, ' ');
            let cmd = parts.next().unwrap_or("");
            let arg = parts.next().unwrap_or("").trim();

            match cmd {
                "quit" | "exit" | "q" => break,
                "help" | "h" => help(),
                "mood" => match Mood::parse(arg) {
                    Some(m) => {
                        mood = m;
                        println!("{}", format!("(the cow now looks {arg})").dimmed());
                    }
                    None => println!("{}", format!("unknown mood: {arg:?}").red()),
                },
                "fortune" => {
                    let idx = rand::rng().random_range(0..FORTUNES.len());
                    println!("{}", render(FORTUNES[idx], mood));
                }
                other => println!("{}", format!("unknown command: /{other}").red()),
            }
        } else {
            // Plain text: the cow says it.
            println!("{}", render(line, mood));
        }
    }

    println!("{}", "Bye. Moo.".dimmed());
}
