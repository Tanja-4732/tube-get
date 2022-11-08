use std::sync::Arc;

use anyhow::Result;
use indicatif::MultiProgress;

mod cli;
mod constants;
mod download;
mod extractor;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
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
    let cli_options = cli::get_options(&matches)?;

    let client = extractor::make_client(&cli_options.token)?;

    let episodes_data = extractor::get_episodes(
        &client,
        cli_options.skip_count.unwrap_or(0),
        cli_options.limit_count.unwrap_or(99999),
        cli_options.uuid,
        cli_options.verbosity,
    )
    .await?;

    let course = extractor::extract_course_data(&episodes_data)?;

    if !cli_options.no_download {
        let multi_bar = Arc::new(MultiProgress::new());
        let jh = tokio::spawn(download::download_course(
            cli_options,
            course,
            Arc::clone(&multi_bar),
            &client,
        ));

        multi_bar.join()?;
        jh.await??;
    } else {
        println!("{:#?}", course);
    }

    Ok(())
}
