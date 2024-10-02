use spinoff::{spinners, Color, Spinner};

pub struct Progress(Spinner);
impl Progress {
    pub fn new(msg: impl AsRef<str>) -> Self {
        Self(Spinner::new(
            spinners::Dots,
            msg.as_ref().to_string(),
            Color::Yellow,
        ))
    }

    pub fn start() -> Self {
        Self(Spinner::new(spinners::Dots, "", Color::Yellow))
    }

    pub fn success(&mut self, msg: impl AsRef<str>) {
        self.0.success(msg.as_ref());
        self.0 = Spinner::new(spinners::Dots, msg.as_ref().to_string(), Color::Yellow);
    }

    pub fn fail(&mut self, msg: impl AsRef<str>) {
        self.0.fail(msg.as_ref());
        self.0 = Spinner::new(spinners::Dots, msg.as_ref().to_string(), Color::Yellow);
    }

    pub fn log(&mut self, msg: impl AsRef<str>) {
        self.0.stop_with_message(msg.as_ref());
        self.0 = Spinner::new(spinners::Dots, msg.as_ref().to_string(), Color::Yellow);
    }

    pub fn update(&mut self, msg: impl AsRef<str>) {
        self.0.update_text(msg.as_ref().to_string());
    }

    pub fn finish_success(&mut self, msg: &str) {
        self.0.success(msg);
    }

    pub fn finish_fail(&mut self, msg: &str) {
        self.0.fail(msg);
    }
}
