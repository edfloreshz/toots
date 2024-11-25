// SPDX-License-Identifier: {{LICENSE}}

use crate::config::TootConfig;
use crate::pages::Page;
use crate::widgets::status::StatusHandles;
use crate::{fl, pages};
use cosmic::app::{context_drawer, Core, Task};
use cosmic::cosmic_config;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::widget::about::About;
use cosmic::widget::menu::{ItemHeight, ItemWidth};
use cosmic::widget::{self, menu, nav_bar};
use cosmic::{Application, ApplicationExt, Apply, Element};
use mastodon_async::helpers::toml;
use mastodon_async::prelude::{Account, Status};
use mastodon_async::registration::Registered;
use mastodon_async::{Data, Mastodon, Registration};
use std::collections::HashMap;

const REPOSITORY: &str = "https://github.com/edfloreshz/toot";
const SUPPORT: &str = "https://github.com/edfloreshz/toot/issues";

pub struct AppModel {
    core: Core,
    about: About,
    nav: nav_bar::Model,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: TootConfig,
    handler: Option<cosmic_config::Config>,
    instance: String,
    code: String,
    registration: Option<Registered>,
    mastodon: Option<Mastodon>,
    home: pages::home::Home,
    notifications: pages::notifications::Notifications,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open(String),
    ToggleContextPage(ContextPage),
    ToggleContextDrawer,
    UpdateConfig(TootConfig),
    InstanceEdit(String),
    RegisterMastodonClient,
    CompleteRegistration,
    StoreMastodonData(Mastodon),
    CodeUpdate(String),
    StoreRegistration(Option<Registered>),
    Home(pages::home::Message),
    Notifications(pages::notifications::Message),
}

pub struct Flags {
    pub config: TootConfig,
    pub handler: Option<cosmic_config::Config>,
}

impl Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = "dev.edfloreshz.Toot";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        for page in Page::variants() {
            let id = nav
                .insert()
                .text(page.to_string())
                .icon(widget::icon::from_name(page.icon()))
                .data::<Page>(page.clone())
                .id();

            if page == Page::default() {
                nav.activate(id);
            }
        }

        let about = About::default()
            .name(fl!("app-title"))
            .version("0.1.0")
            .icon(Self::APP_ID)
            .author("Eduardo Flores")
            .developers([("Eduardo Flores", "edfloreshz@proton.me")])
            .links([(fl!("repository"), REPOSITORY), (fl!("support"), SUPPORT)]);

        let mastodon = match keytar::get_password(Self::APP_ID, "mastodon-data") {
            Ok(data) => {
                if data.success {
                    let data: Data = toml::from_str(&data.password).unwrap();
                    Some(Mastodon::from(data))
                } else {
                    None
                }
            }
            Err(err) => {
                tracing::error!("{err}");
                None
            }
        };

        let mut app = AppModel {
            core,
            about,
            nav,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            config: flags.config.clone(),
            handler: flags.handler,
            instance: flags.config.server,
            code: String::new(),
            registration: None,
            mastodon: mastodon.clone(),
            home: pages::home::Home::new(),
            notifications: pages::notifications::Notifications::new(),
        };

        let mut tasks = vec![app.update_title()];

        if mastodon.is_some() {
            tasks.push(
                app.home
                    .update(pages::home::Message::SetClient(app.mastodon.clone())),
            );
        }

        (app, Task::batch(tasks))
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(
                    fl!("about"),
                    Some(widget::icon::from_name("help-info-symbolic").into()),
                    MenuAction::About,
                )],
            ),
        )])
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(260))
        .spacing(spacing.space_xxxs.into());

        vec![menu_bar.into()]
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav.activate(id);
        let mut tasks = vec![];
        match self.nav.data::<Page>(id).unwrap() {
            Page::Home => {
                if self.home.mastodon.is_none() {
                    tasks.push(
                        self.home
                            .update(pages::home::Message::SetClient(self.mastodon.clone())),
                    )
                }
            }
            Page::Notifications => {
                if self.notifications.mastodon.is_none() {
                    tasks.push(
                        self.notifications
                            .update(pages::notifications::Message::SetClient(
                                self.mastodon.clone(),
                            )),
                    )
                }
            }
            Page::Search => (),
            Page::Favorites => (),
            Page::Bookmarks => (),
            Page::Hashtags => (),
            Page::Lists => (),
            Page::Explore => (),
            Page::Local => (),
            Page::Federated => (),
        };
        tasks.push(self.update_title());
        Task::batch(tasks)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match &self.context_page {
            ContextPage::About => {
                context_drawer::about(&self.about, Message::Open, Message::ToggleContextDrawer)
                    .title(self.context_page.title())
            }
            ContextPage::Account(account) => {
                context_drawer::context_drawer(self.account(account), Message::ToggleContextDrawer)
                    .title(self.context_page.title())
            }
            ContextPage::Status((status, handles)) => context_drawer::context_drawer(
                self.status(status, handles.clone()),
                Message::ToggleContextDrawer,
            )
            .title(self.context_page.title()),
        })
    }

    fn on_escape(&mut self) -> Task<Self::Message> {
        if self.core.window.show_context {
            self.core.window.show_context = false;
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        match self.mastodon {
            Some(_) => match self.nav.active_data::<Page>() {
                Some(page) => match page {
                    Page::Home => self.home.view().map(Message::Home),
                    Page::Notifications => self.notifications.view().map(Message::Notifications),
                    _ => widget::text("Not yet implemented").into(),
                },
                None => widget::text("Select a page").into(),
            },
            None => {
                if self.registration.is_some() {
                    self.code()
                } else {
                    self.login()
                }
            }
        }
        .apply(widget::container)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = vec![self
            .core()
            .watch_config::<TootConfig>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config))];

        subscriptions.push(
            self.home
                .subscription()
                .map(|message| Message::Home(message)),
        );
        subscriptions.push(
            self.notifications
                .subscription()
                .map(|message| Message::Notifications(message)),
        );

        if let Some(mastodon) = self.mastodon.clone() {
            subscriptions.push(crate::subscriptions::mastodon_user_events(mastodon));
        }

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let mut tasks = vec![];
        match message {
            Message::Home(message) => tasks.push(self.home.update(message)),
            Message::Notifications(message) => tasks.push(self.notifications.update(message)),
            Message::InstanceEdit(instance) => {
                self.instance = instance.clone();
                if let Some(ref handler) = self.handler {
                    match self.config.set_server(handler, instance) {
                        Ok(true) => (),
                        Ok(false) => tracing::error!("Failed to write config"),
                        Err(err) => tracing::error!("{err}"),
                    }
                }
            }
            Message::RegisterMastodonClient => {
                let mut registration = Registration::new(self.config.url());
                tasks.push(Task::perform(
                    async move {
                        match registration.client_name("Toot").build().await {
                            Ok(registration) => Some(registration),
                            Err(err) => {
                                tracing::error!("{err}");
                                None
                            }
                        }
                    },
                    |registration| {
                        cosmic::app::message::app(Message::StoreRegistration(registration))
                    },
                ));
            }
            Message::StoreRegistration(registration) => {
                if let Some(ref registration) = registration {
                    if let Ok(url) = registration.authorize_url() {
                        if let Err(err) = open::that_detached(url) {
                            tracing::error!("{err}");
                        }
                    }
                }
                self.registration = registration;
            }
            Message::CompleteRegistration => {
                if let Some(registration) = self.registration.take() {
                    let code = self.code.clone();
                    let task = Task::perform(
                        async move {
                            match registration.complete(code).await {
                                Ok(mastodon) => Some(mastodon),
                                Err(err) => {
                                    tracing::error!("{err}");
                                    None
                                }
                            }
                        },
                        |data| match data {
                            Some(data) => {
                                cosmic::app::message::app(Message::StoreMastodonData(data))
                            }
                            None => cosmic::app::message::none(),
                        },
                    );
                    tasks.push(task);
                }
            }
            Message::StoreMastodonData(mastodon) => {
                let data = &toml::to_string(&mastodon.data).unwrap();
                match keytar::set_password(Self::APP_ID, "mastodon-data", data) {
                    Ok(_) => {
                        self.mastodon = Some(mastodon.clone());
                        tasks.push(self.on_nav_select(self.nav.active()));
                    }
                    Err(err) => tracing::error!("{err}"),
                }
            }
            Message::CodeUpdate(code) => self.code = code,
            Message::Open(url) => {
                if let Err(err) = open::that_detached(url) {
                    tracing::error!("{err}")
                }
            }
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
        Task::batch(tasks)
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

    fn login(&self) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        widget::column()
            .push(widget::icon::from_name("network-server-symbolic").size(48))
            .push(widget::text::title3(fl!("server-question")))
            .push(widget::text::body(fl!("server-description")))
            .push(
                widget::text_input(fl!("server-url"), &self.instance)
                    .on_input(Message::InstanceEdit)
                    .on_submit(Message::RegisterMastodonClient),
            )
            .push(
                widget::button::suggested(fl!("continue"))
                    .on_press(Message::RegisterMastodonClient),
            )
            .spacing(spacing.space_xs)
            .align_x(Horizontal::Center)
            .max_width(400.)
            .into()
    }

    fn code(&self) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        widget::column()
            .push(widget::icon::from_name("network-server-symbolic").size(48))
            .push(widget::text::title3(fl!("confirm-authorization")))
            .push(widget::text::body(fl!("confirm-authorization-description")))
            .push(
                widget::text_input(fl!("authorization-code"), &self.code)
                    .on_input(Message::CodeUpdate)
                    .on_submit(Message::CompleteRegistration),
            )
            .push(
                widget::button::suggested(fl!("continue")).on_press(Message::CompleteRegistration),
            )
            .spacing(spacing.space_xs)
            .align_x(Horizontal::Center)
            .max_width(400.)
            .into()
    }

    fn status(&self, status: &Status, handles: StatusHandles) -> Element<Message> {
        widget::column()
            .push(
                crate::widgets::status(status, &handles)
                    .map(pages::home::Message::Status)
                    .map(Message::Home)
                    .apply(widget::container)
                    .class(cosmic::theme::Container::Dialog),
            )
            .into()
    }

    fn account(&self, _account: &Account) -> Element<Message> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Account(Account),
    Status((Status, StatusHandles)),
}
impl ContextPage {
    fn title(&self) -> String {
        match self {
            ContextPage::About => fl!("about"),
            ContextPage::Account(_) => fl!("profile"),
            ContextPage::Status(_) => fl!("status"),
        }
    }
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
