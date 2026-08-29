use ratatui::widgets::{ListState, ScrollbarState};

use crate::app::{
    cache::{FilterableListCache, StateCache},
    common::{Filter, FilterableList},
};

impl<Item, ItemCache> From<FilterableList<Item>> for FilterableListCache<ItemCache>
where
    Item: Into<ItemCache>,
{
    fn from(value: FilterableList<Item>) -> Self {
        let (filter, is_filter_mod) = value.filter.into_parts();

        Self {
            filter,
            filtered_list: value.filtered_list,
            is_filter_mod,
            show_scrollable: value.show_scrollable,
            list: value
                .inner_list
                .into_iter()
                .map(|item| item.into())
                .collect(),

            state: StateCache {
                selected: value.state.selected(),
            },
            is_filterable: value.is_filterable,
            title: value.title,
        }
    }
}

impl<ItemCache, Item> From<FilterableListCache<ItemCache>> for FilterableList<Item>
where
    ItemCache: Into<Item>,
{
    fn from(value: FilterableListCache<ItemCache>) -> Self {
        let mut state = ListState::default();
        state.select(value.state.selected);

        Self {
            scrollbar_state: ScrollbarState::new(value.filtered_list.len())
                .position(state.selected().unwrap_or(0)),
            filter: Filter::from_parts(value.filter, value.is_filter_mod),
            filtered_list: value.filtered_list,
            show_scrollable: value.show_scrollable,
            inner_list: value.list.into_iter().map(|item| item.into()).collect(),
            state,
            is_filterable: value.is_filterable,
            title: value.title,
        }
    }
}
