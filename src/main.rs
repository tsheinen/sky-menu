mod application;
mod error;
mod icon;
mod style;
mod utility;
mod widgets;

use crate::application::generate_application_list;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use iced::{
    button, executor, keyboard, text_input, window, Align, Application, Button, Color, Column,
    Command, Container, Element, HorizontalAlignment, Length, Sandbox, Settings, Subscription,
    Text, TextInput,
};
use iced_native::{event, subscription, Event};
use itertools::Itertools;
use log::info;
pub fn main() -> iced::Result {
    Launcher::run(Settings {
        window: window::Settings {
            size: (400, 500),
            resizable: false,
            transparent: true,
            decorations: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Default)]
struct Launcher {
    value: i32,
    increment_button: button::State,
    decrement_button: button::State,
    items: Vec<application::Entry>,
    selected: usize,
    search_state: text_input::State,
    search_val: String,
}

#[derive(Debug, Clone)]
pub enum SearchMessage {
    Append(char),
}

#[derive(Debug, Clone)]
pub enum Message {
    MoveSelectedUp,
    MoveSelectedDown,
    ResetSelected,
    Search(SearchMessage),
}

impl Application for Launcher {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                items: vec![],
                ..Self::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("sky-menu")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::MoveSelectedUp => {
                self.selected = (self.selected as isize - 1).rem_euclid(8) as usize
            }
            Message::MoveSelectedDown => self.selected = (self.selected + 1).rem_euclid(8),
            Message::ResetSelected => {
                self.selected = 0;
            }
            Message::Search(search_message) => match search_message {
                SearchMessage::Append(ch) if ch == '\x08' => {
                    self.search_val.pop();
                }
                SearchMessage::Append(ch) if ch >= '\x20' && ch <= '\x7e' => {
                    self.search_val.push(ch)
                }
                SearchMessage::Append(ch) => {
                    info!("trying to append non printable ascii {:?}", ch);
                }
            },
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    modifiers,
                    key_code,
                }) => handle_hotkey(key_code),
                Event::Keyboard(keyboard::Event::CharacterReceived(c)) => {
                    Some(Message::Search(SearchMessage::Append(c)))
                }
                _ => None,
            }
        })
    }

    fn view(&mut self) -> Element<Message> {
        let selected = self.selected;
        let mut column = Column::new()
            .align_items(Align::Start)
            .width(Length::Fill)
            .push(
                Container::new(Text::new(self.search_val.clone()).size(30))
                    .width(Length::Fill)
                    .height(Length::FillPortion(1)),
            );
        Container::new(
            self.get_menu_applications(&self.search_val)
                .iter()
                .enumerate()
                .fold(column, |sum, (idx, app)| {
                    sum.push(
                        Container::new(crate::widgets::get_entry(
                            app.name.clone(),
                            icon::lookup_icon(app.icon.clone()).unwrap_or(
                                icon::lookup_icon("application-x-executable".to_string())
                                    .unwrap_or("".to_string()),
                            ),
                        ))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(if selected == idx {
                            crate::style::Styles::Highlighted
                        } else {
                            crate::style::Styles::Transparent
                        }),
                    )
                }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(crate::style::Styles::TransparentDark)
        .into()
    }
}

impl Launcher {
    fn get_menu_applications(&self, query: &str) -> Vec<application::Entry> {
        let matcher = SkimMatcherV2::default();

        generate_application_list()
            .values()
            .cloned()
            .map(|x| (matcher.fuzzy_match(&x.name, &query).unwrap_or(0), x))
            .sorted_by(|a, b| b.0.cmp(&a.0))
            .map(|x| x.1)
            .take(8)
            .collect()
    }
}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    println!("{:?}", key_code);
    match key_code {
        keyboard::KeyCode::Up => Some(Message::MoveSelectedUp),
        keyboard::KeyCode::Down => Some(Message::MoveSelectedDown),
        _ => None,
    }
}
