use crate::app::common::{HelpItem, HelpMenu};

pub fn get_help_menu() -> HelpMenu {
    let help_items = vec![
        HelpItem {
            key: "j".to_string(),
            desc: String::from("Select below item"),
        },
        HelpItem {
            key: "<Down>".to_string(),
            desc: String::from("Select below item"),
        },
        HelpItem {
            key: "k".to_string(),
            desc: String::from("Select above item"),
        },
        HelpItem {
            key: "<Down>".to_string(),
            desc: String::from("Select above item"),
        },
        HelpItem {
            key: String::from("<Space>"),
            desc: String::from("Toggle Port Forward"),
        },
        HelpItem {
            key: String::from("d"),
            desc: String::from("Delete Port Forward"),
        },
    ];

    HelpMenu::new(String::from("SideBar"), help_items)
}
