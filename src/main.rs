use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

mod help;

fn main() {
    let args = Args::parse();
    println!("Hello, world!");

    if help::ask_yes_no("Do you want to continue?") {
        println!("You chose yes.");
    } else {
        println!("You chose no.");
    }
}
