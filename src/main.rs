mod daemon;

use std::env;

fn main() {
    let mut args = env::args();
    if args.len() > 2 {
        eprintln!("[Error] Too many args!");
        return;
    }
    args.next();

    if let Some(flag) = args.next() {
        if flag == "--daemon" {
            daemon::launch_daemon();
        } else {
            eprintln!("[Error] Unknown flag: {}", flag);
        }
    } else {
        // launch gui
    }
}
