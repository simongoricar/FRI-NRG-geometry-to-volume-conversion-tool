use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "nrg-m2v",
    author = "Simon G. <simon.peter.goricar@gmail.com>",
    version
)]
pub struct CliArgs {
    #[arg(long = "console-logging-level")]
    pub console_logging_output_level_filter: Option<String>,

    #[arg(short = 'i', long = "input-file-path")]
    pub input_file_path: PathBuf,
    // TODO
}
