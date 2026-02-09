use crate::app::common::HelpItem;

pub const HELP_ITEMS: [HelpItem; 5] = [
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
        key: "<Space>",
        desc: "Toggle Port Forward",
    },
];
