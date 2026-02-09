use crate::app::common::HelpItem;

pub const HELP_ITEMS: [HelpItem; 9] = [
    HelpItem {
        key: "j",
        desc: "Select below item",
    },
    HelpItem {
        key: "<Down>",
        desc: "Select below item",
    },
    HelpItem {
        key: "k",
        desc: "Select above item",
    },
    HelpItem {
        key: "<Down>",
        desc: "Select above item",
    },
    HelpItem {
        key: "/",
        desc: "Search",
    },
    HelpItem {
        key: "<Esc>",
        desc: "Reset Search",
    },
    HelpItem {
        key: "[",
        desc: "Select next submenu",
    },
    HelpItem {
        key: "]",
        desc: "Select prev submenu",
    },
    HelpItem {
        key: "<Enter>",
        desc: "Load pods for selected namespace",
    },
];
