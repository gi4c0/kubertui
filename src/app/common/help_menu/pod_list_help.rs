use crate::app::common::HelpItem;

pub const HELP_ITEMS: [HelpItem; 8] = [
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
        key: "p",
        desc: "Port Forward",
    },
    HelpItem {
        key: "l",
        desc: "Logs",
    },
];
