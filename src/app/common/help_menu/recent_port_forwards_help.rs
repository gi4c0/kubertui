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
        key: "<Space>",
        desc: "Toggle Port Forward",
    },
];
