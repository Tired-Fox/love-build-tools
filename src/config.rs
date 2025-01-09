use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{Version, data_dir};

#[derive(Debug, Deserialize, Serialize)]
pub struct Build {
    pub name: String,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub icons: Vec<PathBuf>,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub targets: BTreeMap<Framework, Settings>,

    #[serde(default, skip_serializing_if="Option::is_none")]
    pub config: Option<Config>,
}

impl Build {
    pub fn parse_or_default() -> anyhow::Result<Self> {
        let cd = std::env::current_dir()?;
        let luarc = cd.join(".luarc.json");
        if luarc.exists() {
            let luarc = std::fs::read_to_string(luarc)?;
            let config: serde_json::Value = serde_json::from_str(&luarc)?;
            if let Some(build) = config.get("build") {
                return serde_json::from_value(build.clone()).map_err(anyhow::Error::new);
            }
        }

        Ok(Self {
            name: cd.file_name().unwrap().to_str().unwrap().to_string(),
            icons: Default::default(),
            targets: Default::default(),
            config: Default::default(),
        })
    }

    pub fn new(name: impl std::fmt::Display) -> Self {
        Self {
            name: name.to_string(),
            icons: Default::default(),
            targets: Default::default(),
            config: Default::default(),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let cd = std::env::current_dir()?;
        let value = serde_json::to_value(self)?;

        let luarc = cd.join(".luarc.json");
        if luarc.exists() {
            let text = std::fs::read_to_string(luarc)?;
            let mut config: serde_json::Value = serde_json::from_str(&text)?;
            config["build"] = value;
            std::fs::write(cd.join(".luarc.json"), serde_json::to_string(&config)?)?;
        } else {
            let config = serde_json::json!({
                "build": value
            });
            std::fs::write(cd.join(".luarc.json"), serde_json::to_string(&config)?)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Framework {
    Love,
    Lovr,
}

impl FromStr for Framework {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "love" => Ok(Self::Love),
            "lovr" => Ok(Self::Lovr),
            other => Err(format!("invalid framework: {other}")),
        }
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Love => "love",
                Self::Lovr => "lovr",
            }
        )
    }
}

impl Framework {
    #[inline]
    pub const fn owner(&self) -> &str {
        match self {
            Self::Love => "love2d",
            Self::Lovr => "bjornbytes",
        }
    }

    #[inline]
    pub const fn repo(&self) -> &str {
        match self {
            Self::Love => "love",
            Self::Lovr => "lovr",
        }
    }

    #[inline]
    pub const fn min_version(&self) -> Version {
        match self {
            Self::Love => Version::min_love_version(),
            Self::Lovr => Version::min_lovr_version(),
        }
    }

    #[inline]
    pub const fn latest(&self) -> Version {
        match self {
            Self::Love => Version::latest_love_version(),
            Self::Lovr => Version::latest_lovr_version(),
        }
    }

    #[inline]
    pub fn path(&self, target: Arch) -> PathBuf {
        data_dir().join(target.to_string()).join(self.to_string())
    }

    #[inline]
    pub fn exe(&self, target: Arch) -> PathBuf {
        self.path(target).join(format!("{self}.exe"))
    }

    #[inline]
    pub fn sample(&self) -> &'static str {
        match self {
            Self::Love => indoc::indoc! {r#"
                function love.draw()
                    love.graphics.print("Hello World!", 400, 300)
                end
            "#},
            Self::Lovr => indoc::indoc! {r#"
                function lovr.draw(pass)
                    pass:text("hello world", 0, 1.7, -3, 0.5)
                end
            "#},
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Arch {
    Win64,
    Macos,
    Linux,
    Ios,
    Android,
}

impl Default for Arch {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::Win64
        }
        #[cfg(target_os = "macos")]
        {
            Self::Macos
        }
        #[cfg(target_os = "linux")]
        {
            Self::Linux
        }
        #[cfg(target_os = "ios")]
        {
            Self::Ios
        }
        #[cfg(target_os = "android")]
        {
            Self::Android
        }
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Win64 => "windows",
                Self::Macos => "macos",
                Self::Linux => "linux",
                Self::Ios => "ios",
                Self::Android => "android",
            }
        )
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub version: Version,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub targets: Vec<Arch>,

    #[serde(default, skip_serializing_if="Option::is_none")]
    pub entry: Option<String>,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub exclude: Vec<PathBuf>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub identity: Option<String>,
    #[serde(default)]
    pub appendidentity: bool,
    #[serde(default)]
    pub version: Option<Version>,
    #[serde(default)]
    pub console: bool,
    #[serde(default="default_true")]
    pub accelerometerjoystick: bool,
    #[serde(default)]
    pub externalstorage: bool,
    #[serde(default)]
    pub gammacorrect: bool,

    #[serde(default)]
    pub audio: Option<Audio>,
    #[serde(default)]
    pub window: Option<Window>,
    #[serde(default)]
    pub modules: Option<Modules>,
}

impl Config {
    pub const fn param() -> &'static str {
        "t"
    } 
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let param = Config::param();
        if let Some(identity) = self.identity.as_deref() { writeln!(f, r#"  {param}.identity = "{identity}""#)?; }
        if self.appendidentity { writeln!(f, "  {param}.appendidentity = true")?; }
        if let Some(version) = self.version.as_ref() { writeln!(f, r#"  {param}.version = "{version}""#)?; }
        if self.console { writeln!(f, r#"  {param}.console = true"#)?; }
        if !self.accelerometerjoystick { writeln!(f, r#"  {param}.accelerometerjoystick = false"#)?; }
        if self.externalstorage { writeln!(f, r#"  {param}.externalstorage = true"#)?; }
        if self.gammacorrect { writeln!(f, r#"  {param}.gammacorrect = true"#)?; }

        if let Some(audio) = self.audio.as_ref() {
            writeln!(f, "{audio}")?;
        }
        if let Some(window) = self.window.as_ref() {
            writeln!(f, "{window}")?;
        }
        if let Some(modules) = self.modules.as_ref() {
            writeln!(f, "{modules}")?;
        }

        Ok(())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Audio {
    #[serde(default)]
    pub mic: bool,
    #[serde(default)]
    pub mixwithsystem: bool,
}

impl std::fmt::Display for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let param = Config::param();

        if self.mic { writeln!(f, "  {param}.audio.mic = true")?; }
        if self.mixwithsystem { writeln!(f, "  {param}.audio.mixwithsystem = true")?; }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Window {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    #[serde(default)]
    pub borderless: bool,
    #[serde(default)]
    pub resizable: bool,
    pub minwidth: Option<usize>,
    pub minheight: Option<usize>,
    #[serde(default)]
    pub fullscreen: bool,
    pub fullscreentype: Option<FullscreenType>,
    pub vsync: Option<usize>,
    pub msaa: Option<usize>,
    pub depth: Option<usize>,
    pub stencil: Option<usize>,
    pub display: Option<usize>,
    #[serde(default)]
    pub highdpi: bool,
    #[serde(default="default_true")]
    pub usedpiscale: bool,
    pub x: Option<usize>,
    pub y: Option<usize>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: None,
            icon: None,
            width: None,
            height: None,
            borderless: false,
            resizable: false,
            minwidth: None,
            minheight: None,
            fullscreen: false,
            fullscreentype: None,
            vsync: None,
            msaa: None,
            depth: None,
            stencil: None,
            display: None,
            highdpi: false,
            usedpiscale: true,
            x: None,
            y: None,
        }
    }
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let param = Config::param();
        if let Some(title) = self.title.as_deref() { writeln!(f, r#"  {param}.window.title = "{title}""#)?; }
        if let Some(icon) = self.icon.as_deref() { writeln!(f, r#"  {param}.window.icon = "{icon}""#)?; }
        if let Some(width) = self.width.as_ref() { writeln!(f, r#"  {param}.window.width = "{width}""#)?; }
        if let Some(height) = self.height.as_ref() { writeln!(f, r#"  {param}.window.height = "{height}""#)?; }
        if let Some(minwidth) = self.minwidth.as_ref() { writeln!(f, r#"  {param}.window.minwidth = "{minwidth}""#)?; }
        if let Some(minheight) = self.minheight.as_ref() { writeln!(f, r#"  {param}.window.minheight = "{minheight}""#)?; }
        if let Some(fullscreentype) = self.fullscreentype.as_ref() { writeln!(f, r#"  {param}.window.fullscreentype = "{fullscreentype}""#)?; }
        if let Some(vsync) = self.vsync.as_ref() { writeln!(f, r#"  {param}.window.vsync = "{vsync}""#)?; }
        if let Some(msaa) = self.msaa.as_ref() { writeln!(f, r#"  {param}.window.msaa = "{msaa}""#)?; }
        if let Some(depth) = self.depth.as_ref() { writeln!(f, r#"  {param}.window.depth = "{depth}""#)?; }
        if let Some(stencil) = self.stencil.as_ref() { writeln!(f, r#"  {param}.window.stencil = "{stencil}""#)?; }
        if let Some(display) = self.display.as_ref() { writeln!(f, r#"  {param}.window.display = "{display}""#)?; }
        if let Some(x) = self.x.as_ref() { writeln!(f, r#"  {param}.window.x = "{x}""#)?; }
        if let Some(y) = self.y.as_ref() { writeln!(f, r#"  {param}.window.y = "{y}""#)?; }

        if self.borderless { writeln!(f, "  {param}.window.borderless = true")?; }
        if self.resizable { writeln!(f, "  {param}.window.resizable = true")?; }
        if self.fullscreen { writeln!(f, "  {param}.window.fullscreen = true")?; }
        if self.highdpi { writeln!(f, "  {param}.window.highdpi = true")?; }
        if !self.usedpiscale { writeln!(f, "  {param}.window.usedpiscale = false")?; }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="snake_case")]
pub enum FullscreenType {
    Desktop,
    Exclusive,
}

impl std::fmt::Display for FullscreenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "desktop"),
            Self::Exclusive => write!(f, "exclusive"),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Modules {
    #[serde(default="default_true")]
    pub audio: bool,
    #[serde(default="default_true")]
    pub data: bool,
    #[serde(default="default_true")]
    pub event: bool,
    #[serde(default="default_true")]
    pub font: bool,
    #[serde(default="default_true")]
    pub graphics: bool,
    #[serde(default="default_true")]
    pub image: bool,
    #[serde(default="default_true")]
    pub joystick: bool,
    #[serde(default="default_true")]
    pub keyboard: bool,
    #[serde(default="default_true")]
    pub math: bool,
    #[serde(default="default_true")]
    pub mouse: bool,
    #[serde(default="default_true")]
    pub physics: bool,
    #[serde(default="default_true")]
    pub sound: bool,
    #[serde(default="default_true")]
    pub system: bool,
    #[serde(default="default_true")]
    pub thread: bool,
    #[serde(default="default_true")]
    pub timer: bool,
    #[serde(default="default_true")]
    pub touch: bool,
    #[serde(default="default_true")]
    pub video: bool,
    #[serde(default="default_true")]
    pub window: bool,
}

impl Default for Modules {
    fn default() -> Self {
        Self {
            audio: true,
            data: true,
            event: true,
            font: true,
            graphics: true,
            image: true,
            joystick: true,
            keyboard: true,
            math: true,
            mouse: true,
            physics: true,
            sound: true,
            system: true,
            thread: true,
            timer: true,
            touch: true,
            video: true,
            window: true,
        }
    }
}

impl std::fmt::Display for Modules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let param = Config::param();
        if !self.audio { writeln!(f, "  {param}.modules.audio = false")?; }
        if !self.data { writeln!(f, "  {param}.modules.data = false")?; }
        if !self.event { writeln!(f, "  {param}.modules.event = false")?; }
        if !self.font { writeln!(f, "  {param}.modules.font = false")?; }
        if !self.graphics { writeln!(f, "  {param}.modules.graphics = false")?; }
        if !self.image { writeln!(f, "  {param}.modules.image = false")?; }
        if !self.joystick { writeln!(f, "  {param}.modules.joystick = false")?; }
        if !self.keyboard { writeln!(f, "  {param}.modules.keyboard = false")?; }
        if !self.math { writeln!(f, "  {param}.modules.math = false")?; }
        if !self.mouse { writeln!(f, "  {param}.modules.mouse = false")?; }
        if !self.physics { writeln!(f, "  {param}.modules.physics = false")?; }
        if !self.sound { writeln!(f, "  {param}.modules.sound = false")?; }
        if !self.system { writeln!(f, "  {param}.modules.system = false")?; }
        if !self.thread { writeln!(f, "  {param}.modules.thread = false")?; }
        if !self.timer { writeln!(f, "  {param}.modules.timer = false")?; }
        if !self.touch { writeln!(f, "  {param}.modules.touch = false")?; }
        if !self.video { writeln!(f, "  {param}.modules.video = false")?; }
        if !self.window { writeln!(f, "  {param}.modules.window = false")?; }

        Ok(())
    }
}
