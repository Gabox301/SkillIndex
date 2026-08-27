pub mod args;
pub mod banner;
pub mod cache;
pub mod claude;
pub mod detect;
pub mod display;
pub mod dotnet;
pub mod frontend;
pub mod gradle;
pub mod hash;
pub mod installer;
pub mod prompt;
pub mod registry;
pub mod ui;
pub mod workspace;

pub mod skills_map {
    include!(concat!(env!("OUT_DIR"), "/skills_map.rs"));
}
