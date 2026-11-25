pub mod generic_scanner;
pub mod param_sfo;
pub mod ps3_scanner;

use crate::domain::scanner::GameDiscoveryPort;

use self::generic_scanner::PlatformExtensionDiscovery;
use self::ps3_scanner::Ps3GameDiscovery;

pub fn discovery_for_console(console_slug: &str) -> Option<Box<dyn GameDiscoveryPort>> {
    match console_slug {
        "ps3" => Some(Box::new(Ps3GameDiscovery)),
        "ps2" => Some(Box::new(PlatformExtensionDiscovery::for_ps2())),
        "wii" => Some(Box::new(PlatformExtensionDiscovery::for_wii())),
        "xbox" => Some(Box::new(PlatformExtensionDiscovery::for_xbox())),
        "xbox360" => Some(Box::new(PlatformExtensionDiscovery::for_xbox360())),
        "ps4" => Some(Box::new(PlatformExtensionDiscovery::for_ps4())),
        _ => None,
    }
}
