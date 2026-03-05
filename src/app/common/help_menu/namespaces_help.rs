use crate::app::common::HelpItem;

pub const HELP_ITEMS: [HelpItem; 13] = [
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
        key: "l",
        desc: "Select next block in submenu",
    },
    HelpItem {
        key: "<right>",
        desc: "Select next block in submenu",
    },
    HelpItem {
        key: "h",
        desc: "Select previous block in submenu",
    },
    HelpItem {
        key: "<left>",
        desc: "Select previous block in submenu",
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
