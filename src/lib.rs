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
    pub const SKILLS_MAP_JSON: &str = include_str!("../skills_map.json");

    pub fn skills_map_json() -> &'static str {
        SKILLS_MAP_JSON
    }
}
