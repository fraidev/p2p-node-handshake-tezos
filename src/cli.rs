use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    /// The pattern to look for
    pub peer: Option<String>,
    /// The path to the file to read
    pub identity_path: Option<std::path::PathBuf>,
    /// The chain Name
    pub chain_name: Option<String>,
}
