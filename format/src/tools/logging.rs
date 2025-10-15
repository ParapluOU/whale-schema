use simplelog::*;
use std::fs::File;

pub fn init_logger() {
    let file_logger = WriteLogger::new(
        LevelFilter::Trace,
        Config::default(),
        File::create("debug.log").unwrap(),
    );

    CombinedLogger::init(vec![file_logger]);
}
