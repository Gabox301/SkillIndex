pub mod batch;
pub mod single;
#[cfg(test)]
pub mod tests;

pub use batch::{install_all, install_all_with_client};
pub use single::{install_skill, install_skill_with_client};
