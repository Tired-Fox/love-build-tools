mod version;
pub use version::Version;

mod manager;
pub use manager::{Manager, Target, Bundle};

pub mod git;

#[cfg(all(target_os = "windows", target_arch = "x86"))]
pub static DEFAULT_PLATFORM: &str = "win32";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub static DEFAULT_PLATFORM: &str = "win64";
#[cfg(target_os = "linux")]
pub static DEFAULT_PLATFORM: &str = "linux";
#[cfg(target_os = "macos")]
pub static DEFAULT_PLATFORM: &str = "macos";
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub static DEFAULT_PLATFORM: &str = "";
