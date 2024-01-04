use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::Config as Log4rsConfig,
    config::{Appender, Root},
};

pub fn setup_logger() {
    let stdout = ConsoleAppender::builder().build();

    let config = Log4rsConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
