use iced::executor;
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
};
use iced::widget::{
    button, checkbox, Column, container, pick_list, row, slider, scrollable, text,
};
use iced::theme::{self, Theme};
//use iced::window;
//use iced::time;

use netcdf::open;
use std::env;

pub fn main() -> iced::Result {
    Hello::run(Settings::default())
}

struct Hello {
    file_info: String,
}

impl Application for Hello {
    type Executor = executor::Default;
    type Flags = ();
    type Message = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let args: Vec<String> = env::args().collect();
        let file_info = if args.len() == 2 {
            let path = &args[1];
            match open(path) {
                Ok(file) => {
                    let mut info = String::new();
                    info.push_str("Dimensions:\n");
                    for dimension in file.dimensions() {
                        info.push_str(&format!("  Name: {}, Size: {}\n", dimension.name(), dimension.len()));
                    }
                    info.push_str("\nVariables:\n");
                    for variable in file.variables() {
                        info.push_str(&format!("  Name: {}\n", variable.name()));
                        info.push_str(&format!("  Dimensions: {:?}\n", variable.dimensions()));
                        info.push_str(&format!("  Type: {:?}\n", variable.vartype()));
                        info.push_str("  Attributes:\n");
                        for attr in variable.attributes() {
                            info.push_str(&format!("    Name: {}, Value: {:?}\n", attr.name(), attr.value()));
                        }
                        info.push_str("\n");
                    }
                    info.push_str("Global Attributes:\n");
                    if let Some(root) = file.root() {
                        for attr in root.attributes() {
                            info.push_str(&format!("  Name: {}, Value: {:?}\n", attr.name(), attr.value()));
                        }
                    } else {
                        info.push_str("No root group found.\n");
                    }
                    info
                },
                Err(_) => "Failed to open file. Check the file path.\n".to_string(),
            }
        } else {
            "Usage: pass the <path-to-netcdf-file> as an argument.\n".to_string()
        };

        (Self { file_info }, Command::none())
    }

    fn title(&self) -> String {
        String::from("RuNeVis")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        // let content = "Hello, world!".into();
        let content = text::Text::new(&self.file_info).size(16);
        //let scroll = scrollable::Scrollable::new(scrollable::State::new()).push(content);
        Column::new().push(content).into()
    }
}