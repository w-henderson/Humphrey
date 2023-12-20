use humphrey_server::config::Config;
use humphrey_server::logger::Logger;
use humphrey_server::server::server;

#[cfg(test)]
mod tests;

fn main() {
    match Config::load() {
        Ok(config) => server::main(config),
        Err(error) => {
            let logger = Logger::default();
            logger.error(error);
        }
    }
}
