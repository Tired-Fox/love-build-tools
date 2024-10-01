use std::path::PathBuf;

use spinoff::{spinners, Spinner, Color};

mod version;

pub mod git;
pub mod config;
pub mod build;

pub use version::Version;

lazy_static::lazy_static! {
    pub static ref DATA: PathBuf = dirs::data_local_dir().unwrap().join("love-build-tools");
}

pub fn love_path() -> PathBuf {
    DATA.join(std::env::consts::OS).join("love")
}

pub fn lovr_path() -> PathBuf {
    DATA.join(std::env::consts::OS).join("lovr")
}

pub trait SpinnerPrint {
    fn print(&mut self, msg: impl std::fmt::Display);
}

impl SpinnerPrint for Spinner {
    fn print(&mut self, msg: impl std::fmt::Display) {
        self.stop_with_message(msg.to_string().as_str());
        *self = Spinner::new(spinners::Dots, msg.to_string(), Color::Yellow);
    }
}

pub trait SpinnerError<T> {
    fn ok_or_spin(self, spinner: &mut Spinner, msg: impl std::fmt::Display) -> Option<T>;
    fn log_err_in_spin(self, spinner: &mut Spinner, msg: impl std::fmt::Display) -> Self;
}

impl<T, E: std::fmt::Display> SpinnerError<T> for Result<T, E> {
    fn ok_or_spin(self, spinner: &mut Spinner, msg: impl std::fmt::Display) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(err) => {
                spinner.fail(format!("{msg}\n  {err}").as_str()); 
                *spinner = Spinner::new(spinners::Dots, msg.to_string(), Color::Yellow);
                None
            }
        }
    }

    fn log_err_in_spin(self, spinner: &mut Spinner, msg: impl std::fmt::Display) -> Self {
        if let Err(err) = self.as_ref() {
            spinner.fail(format!("{msg}\n  {err}").as_str()); 
            *spinner = Spinner::new(spinners::Dots, msg.to_string(), Color::Yellow);
        }
        self
    }
}
