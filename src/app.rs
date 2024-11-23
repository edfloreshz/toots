// SPDX-License-Identifier: {{LICENSE}}

use crate::config::Config;
use crate::fl;
use cosmic::app::{context_drawer, Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::widget::about::About;
use cosmic::widget::{self, menu, nav_bar};
use cosmic::{Application, ApplicationExt, Apply, Element};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::fmt::Display;

const REPOSITORY: &str = "https://github.com/edfloreshz/toot";
const SUPPORT: &str = "https://github.com/edfloreshz/toot/issues";

pub struct AppModel {
    core: Core,
    about: About,
    context_page: ContextPage,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open(String),
    SubscriptionChannel,
    ToggleContextPage(ContextPage),
    ToggleContextDrawer,
    UpdateConfig(Config),
}

impl Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "dev.edfloreshz.Toot";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        for page in Page::variants() {
            nav.insert()
                .text(page.to_string())
                .icon(widget::icon::from_name(page.icon()));
        }

        let about = About::default()
            .name(fl!("app-title"))
            .version("0.1.0")
            .icon(Self::APP_ID)
            .author("Eduardo Flores")
            .developers([("Eduardo Flores", "edfloreshz@proton.me")])
            .links([(fl!("repository"), REPOSITORY), (fl!("support"), SUPPORT)]);

        let mut app = AppModel {
            core,
            about,
            context_page: ContextPage::default(),
            nav,
            key_binds: HashMap::new(),
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
        };

        let command = app.update_title();

        (app, command)
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav.activate(id);
        self.update_title()
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => {
                context_drawer::about(&self.about, Message::Open, Message::ToggleContextDrawer)
                    .title(fl!("about"))
            }
        })
    }

    fn view(&self) -> Element<Self::Message> {
        widget::text::title1(fl!("welcome"))
            .apply(widget::container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct MySubscription;

        Subscription::batch(vec![
            Subscription::run_with_id(
                std::any::TypeId::of::<MySubscription>(),
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;

                    futures_util::future::pending().await
                }),
            ),
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Open(url) => {
                if let Err(err) = open::that_detached(url) {
                    tracing::error!("{err}")
                }
            }
            Message::SubscriptionChannel => {}
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }
            Message::ToggleContextDrawer => {
                self.core.window.show_context = !self.core.window.show_context;
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
        }
        Task::none()
    }
}

impl AppModel {
    pub fn update_title(&mut self) -> Task<Message> {
        let mut window_title = fl!("app-title");
        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

pub enum Page {
    Home,
    Notifications,
    Search,
    Favorites,
    Bookmarks,
    Hashtags,
    Lists,
    Explore,
    Local,
    Federated,
}

impl Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Home => write!(f, "{}", fl!("home")),
            Page::Notifications => write!(f, "{}", fl!("notifications")),
            Page::Search => write!(f, "{}", fl!("search")),
            Page::Favorites => write!(f, "{}", fl!("favorites")),
            Page::Bookmarks => write!(f, "{}", fl!("bookmarks")),
            Page::Hashtags => write!(f, "{}", fl!("hashtags")),
            Page::Lists => write!(f, "{}", fl!("lists")),
            Page::Explore => write!(f, "{}", fl!("explore")),
            Page::Local => write!(f, "{}", fl!("local")),
            Page::Federated => write!(f, "{}", fl!("federated")),
        }
    }
}

impl Page {
    pub fn variants() -> [Self; 10] {
        [
            Self::Home,
            Self::Notifications,
            Self::Search,
            Self::Favorites,
            Self::Bookmarks,
            Self::Hashtags,
            Self::Lists,
            Self::Explore,
            Self::Local,
            Self::Federated,
        ]
    }

    fn icon(&self) -> &str {
        match self {
            Page::Home => "user-home-symbolic",
            Page::Notifications => "emblem-important-symbolic",
            Page::Search => "folder-saved-search-symbolic",
            Page::Favorites => "emoji-body-symbolic",
            Page::Bookmarks => "bookmark-new-symbolic",
            Page::Hashtags => "lang-include-symbolic",
            Page::Lists => "view-list-symbolic",
            Page::Explore => "find-location-symbolic",
            Page::Local => "network-server-symbolic",
            Page::Federated => "network-workgroup-symbolic",
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
