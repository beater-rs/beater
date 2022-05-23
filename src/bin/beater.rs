use beater::Beater;

fn main() {
    let mut args = std::env::args();

    let (username, password) = match args.len() {
        3 => (args.next().unwrap(), args.next().unwrap()),
        _ => {
            println!(
                "Usage: {} <username> <password> <track-id>",
                args.next().unwrap()
            );
            std::process::exit(1);
        }
    };
}
