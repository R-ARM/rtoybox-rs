use std::fmt::Debug;
use std::fmt;

use thiserror::Error;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Texture;

use sdl2::video::WindowBuildError;
use sdl2::IntegerOrSdlError;
use sdl2::ttf::InitError;
use sdl2::render::TextureValueError;
use sdl2::ttf::FontError;

#[derive(Error, Debug)]
pub enum ToolkitError {
    #[error("SDL Error: {0}")]
    SDLError(String),
    #[error("SDL_ttf input/output Error: {0}")]
    TTFError(std::io::Error),
    #[error("Integer overflow")]
    IntOverflow,
    #[error("TTF Context already initialized")]
    AlreadyInitialized,
    #[error("Input value not a multiple of two")]
    NotMultOfTwo,
    #[error("Invalid input text")]
    InvalidText,

    #[error("No tabs have been created")]
    NoTabs,
}

impl From<ToolkitError> for String {
    fn from(e: ToolkitError) -> String {
        match e {
            ToolkitError::SDLError(s) => format!("SDL Error: {}", s),
            ToolkitError::TTFError(s) => format!("SDL_ttf input/output Error: {}", s),
            ToolkitError::IntOverflow => "Integer overflow".to_string(),
            ToolkitError::AlreadyInitialized => "TTF context already initialized".to_string(),
            ToolkitError::NotMultOfTwo => "Input value not a multiple of two".to_string(),
            ToolkitError::InvalidText => "Invalid input text".to_string(),
            ToolkitError::NoTabs => "No tabs have been created".to_string(),
        }
    }
}

// sdl2 uses multiple types for errors...
impl From<String> for ToolkitError {
    fn from(s: String) -> ToolkitError {
        ToolkitError::SDLError(s)
    }
}

impl From<WindowBuildError> for ToolkitError {
    fn from(e: WindowBuildError) -> ToolkitError {
        ToolkitError::SDLError(e.to_string())
    }
}

impl From<IntegerOrSdlError> for ToolkitError {
    fn from(e: IntegerOrSdlError) -> ToolkitError {
        match e {
            IntegerOrSdlError::IntegerOverflows(_, _) => ToolkitError::IntOverflow,
            IntegerOrSdlError::SdlError(se) => ToolkitError::SDLError(se),
        }
    }
}

impl From<InitError> for ToolkitError {
    fn from(e: InitError) -> ToolkitError {
        match e {
            InitError::AlreadyInitializedError => ToolkitError::AlreadyInitialized,
            InitError::InitializationError(err) => ToolkitError::TTFError(err),
        }
    }
}

impl From<FontError> for ToolkitError {
    fn from(e: FontError) -> ToolkitError {
        match e {
            FontError::InvalidLatin1Text(_) => ToolkitError::InvalidText,
            FontError::SdlError(se) => ToolkitError::SDLError(se),
        }
    }
}

impl From<TextureValueError> for ToolkitError {
    fn from(e: TextureValueError) -> ToolkitError {
        match e {
            TextureValueError::WidthOverflows(_) => ToolkitError::IntOverflow,
            TextureValueError::HeightOverflows(_) => ToolkitError::IntOverflow,
            TextureValueError::WidthMustBeMultipleOfTwoForFormat(_, _) => ToolkitError::NotMultOfTwo,
            TextureValueError::SdlError(se) => ToolkitError::SDLError(se),
        }
    }
}

// For almost everything we want to draw on the screen
pub trait Drawable {
    fn draw(&self) -> Result<(), ToolkitError>;
}

impl Debug for dyn Drawable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FIXME: Debug for Drawable")
    }
}

#[derive(Debug, Clone, Copy)]
enum ButtonType {
    Normal,
}

//#[derive(Debug)]
struct Button<'a> {
    name: &'static str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    typ: ButtonType,
    text: Texture<'a>,
}

impl Button<'_> {
    fn name(&self) -> &'static str { self.name }
    fn x(&self) -> i32 { self.x }
    fn y(&self) -> i32 { self.y }
    fn w(&self) -> i32 { self.w }
    fn h(&self) -> i32 { self.h }
    fn typ(&self) -> ButtonType { self.typ }

    fn new<'a>(tk: &'a Toolkit, name: &'static str, x: i32, y: i32) -> Result<Button<'a>, ToolkitError> {
        let texture = tk.render_text(name)?;
        let attr = texture.query();
        Ok(Button {
            name: name,
            x: x,
            y: y,
            w: attr.width as i32,
            h: attr.height as i32,
            typ: ButtonType::Normal,
            text: texture,
        })
    }
}

impl Debug for Button<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Button")
            .field("name", &self.name())
            .field("x", &self.x())
            .field("y", &self.y())
            .field("type", &self.typ())
            .finish()
    }
}

impl Drawable for Button<'_> {
    fn draw(&self) -> Result<(), ToolkitError> {
        println!("Drawing button {}", self.name());
        Ok(())
    }
}

#[derive(Debug)]
struct Tab<'a> {
    items: Vec<Button<'a>>,
    item_pos: usize,
    name: &'static str,
}

impl Tab<'_> {
    fn new(name: &'static str) -> Tab {
        Tab {
            items: Vec::new(),
            item_pos: 0,
            name: name,
        }
    }
    fn name(&self) -> &'static str { self.name }
}

impl Drawable for Tab<'_> {
    fn draw(&self) -> Result<(), ToolkitError> {
        for button in self.items.iter() {
            button.draw()?;
        }
        Ok(())
    }
}

pub struct Toolkit<'a> {
    tabs: Vec<Tab<'a>>,
    tab_pos: usize,
    items: Vec<Box<dyn Drawable>>,
    run: bool,

    ctx: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pump: sdl2::EventPump,
    ttf: sdl2::ttf::Sdl2TtfContext,
    font: sdl2::ttf::Font<'a, 'static>,
    text_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,

    bg_color: Color,
}

impl Debug for Toolkit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Toolkit")
            .field("tabs", &self.tabs)
            .field("tab_pos", &self.tab_pos)
            .field("items", &self.items)
            .field("run", &self.run)
            .field("bg_color", &self.bg_color)
            .finish()
    }
}

impl Toolkit<'_> {
    pub fn tick(&mut self) -> Result<bool, ToolkitError> {
        for event in self.pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    self.run = false;
                },
                Event::KeyDown {keycode, ..} => {
                    match keycode {
                        Some(Keycode::Escape) => {
                            self.run = false;
                        },
                        _ => { },
                    }
                },
                _ => { },
            }
        }

        self.redraw()?;

        Ok(self.run)
    }

    fn redraw(&mut self) -> Result<(), ToolkitError> {
        self.canvas.set_draw_color(self.bg_color);
        self.canvas.clear();

        for btn in &self.items {
            btn.draw()?;
        }

        match self.tabs.get(self.tab_pos) {
            Some(tab) => tab.draw()?,
            None => /*println!("No tab")*/(),
        }

        self.canvas.present();
        
        Ok(())
    }

    fn render_text<'a>(&'a self, input: &'static str) -> Result<Texture<'a>, ToolkitError> {
        let surface = self.font.render(input).blended(Color::RGBA(255, 255, 255, 255))?;
        let texture = self.text_creator.create_texture_from_surface(&surface)?;

        Ok(texture)
    }

    pub fn add_tab(&mut self, name: &'static str) -> Result<(), ToolkitError> {
        let tab = Tab::new(name);
        self.tabs.push(tab);
        Ok(())
    }

/*    pub fn add_btn<'a>(&mut self, name: &'static str) -> Result<(), ToolkitError> {
        let btn = Button::new(self, name, 0, 0)?;
        match self.tabs.get_mut(self.tab_pos) {
            Some(tab) => return Ok(tab.items.push(btn)),
            None => return Err(ToolkitError::NoTabs),
        }
    }*/

    pub fn new<'a>() -> Result<Toolkit<'a>, ToolkitError> {
        let sdl2 = sdl2::init()?;
        let video = sdl2.video()?;
        let window = video.window("rtoolkit window", 480, 320).build()?;
        let mut canvas = window.into_canvas().present_vsync().build()?;
        let pump = sdl2.event_pump()?;
        let bg_color = Color::RGBA(0, 0, 0, 100);
        let ttf = sdl2::ttf::init()?;
        let font = ttf.load_font("/usr/share/fonts/liberation/LiberationSans.ttf", 28)?;
        let text_creator = canvas.texture_creator();

        canvas.set_draw_color(bg_color);
        canvas.clear();
        canvas.present();

        Ok(Toolkit {
            tabs: Vec::new(),
            tab_pos: 0,
            items: Vec::new(),
            run: true,
            ctx: sdl2,
            video: video,
            canvas: canvas,
            pump: pump,
            bg_color: bg_color,
            ttf: ttf,
            font: font,
            text_creator: text_creator,
        })
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.bg_color = Color::RGBA(0, 0, 0, alpha);
    }
}
