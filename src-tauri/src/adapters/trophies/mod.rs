pub mod rpcs3_trophy_reader;

use crate::domain::trophy::TrophyReaderPort;
pub use rpcs3_trophy_reader::Rpcs3TrophyReader;

pub fn trophy_reader_for_console(console_slug: &str) -> Option<Box<dyn TrophyReaderPort>> {
    match console_slug {
        "ps3" => Some(Box::new(Rpcs3TrophyReader::new())),
        _ => None,
    }
}
