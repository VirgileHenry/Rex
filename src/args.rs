pub enum Args {
    File(std::path::PathBuf),
}

pub fn parse_args() -> Vec<Args> {
    let mut result = Vec::new();
    let args = std::env::args();

    for arg in args.into_iter() {
        result.push(Args::File(arg.into()));
    }

    result
}
