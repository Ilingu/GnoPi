use relm4::{
    actions::{RelmAction, RelmActionGroup},
    adw, gtk, ComponentParts, ComponentSender, SimpleComponent,
};

pub struct HeaderModel;

#[derive(Debug)]
pub enum HeaderOutput {
    Preferences,
    About,
}

relm4::new_action_group!(HeaderMenuActionGroup, "win");
relm4::new_stateless_action!(OpenPreference, HeaderMenuActionGroup, "preferences");
relm4::new_stateless_action!(OpenAbout, HeaderMenuActionGroup, "about");

#[relm4::component(pub)]
impl SimpleComponent for HeaderModel {
    type Init = ();
    type Input = ();
    type Output = HeaderOutput;

    view! {
        #[root]
        header = adw::HeaderBar {
            pack_end = &gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                #[wrap(Some)]
                    set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {}
            },
        }
    }

    menu! {
        main_menu: {
            "Preferences" => OpenPreference,
            "About GnoPi" => OpenAbout,
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderModel;
        let widgets = view_output!();

        let senderp = sender.clone();
        let action_preference: RelmAction<OpenPreference> = {
            RelmAction::new_stateless(move |_| {
                senderp
                    .output(HeaderOutput::Preferences)
                    .expect("Failed to open preference");
            })
        };

        let sendera = sender.clone();
        let action_about: RelmAction<OpenAbout> = {
            RelmAction::new_stateless(move |_| {
                sendera
                    .output(HeaderOutput::About)
                    .expect("Failed to open about");
            })
        };

        let mut group = RelmActionGroup::<HeaderMenuActionGroup>::new();
        group.add_action(action_preference);
        group.add_action(action_about);
        group.register_for_widget(&widgets.header);

        ComponentParts { model, widgets }
    }
}
