mod cli;
mod constants;

fn main() {
    // The working directory
    let pwd = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    // Parse the command line parameters into arg-matches
    let matches = cli::configure_parser(&pwd).get_matches();

    // Print the name and version of the application along its license notice
    println!("{} {}", constants::NAME, constants::VERSION);
    println!("{}\n", constants::LICENSE);

    // Try to extract the desired configuration from the arg-matches
    let cli_options = cli::get_options(matches)?;

    // Make a new client for issuing HTTP(S) requests
    let client = reqwest::Client::new();
}
