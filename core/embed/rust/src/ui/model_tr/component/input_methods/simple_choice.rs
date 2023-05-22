use crate::{
    strutil::StringType,
    ui::{
        component::{Component, Event, EventCtx},
        geometry::Rect,
    },
};

use super::super::{ButtonLayout, ChoiceFactory, ChoiceItem, ChoicePage, ChoicePageMsg};
use heapless::Vec;

pub enum SimpleChoiceMsg {
    ResultIndex(usize),
}

// So that there is only one implementation, and not multiple generic ones
// as would be via `const N: usize` generics.
const MAX_LENGTH: usize = 5;

struct ChoiceFactorySimple<T> {
    choices: Vec<T, MAX_LENGTH>,
    carousel: bool,
}

impl<T> ChoiceFactorySimple<T> {
    fn new(choices: Vec<T, MAX_LENGTH>, carousel: bool) -> Self {
        Self { choices, carousel }
    }
}

impl<T> ChoiceFactory<T> for ChoiceFactorySimple<T>
where
    T: StringType,
{
    type Item = ChoiceItem<T>;

    fn count(&self) -> usize {
        self.choices.len()
    }

    fn get(&self, choice_index: usize) -> ChoiceItem<T> {
        let text = &self.choices[choice_index];
        let mut choice_item = ChoiceItem::new(text, ButtonLayout::default_three_icons());

        // Disabling prev/next buttons for the first/last choice when not in carousel.
        // (could be done to the same button if there is only one)
        if !self.carousel {
            if choice_index == 0 {
                choice_item.set_left_btn(None);
            }
            if choice_index == self.count() - 1 {
                choice_item.set_right_btn(None);
            }
        }

        choice_item
    }
}

/// Simple wrapper around `ChoicePage` that allows for
/// inputting a list of values and receiving the chosen one.
pub struct SimpleChoice<T>
where
    T: StringType,
{
    choices: Vec<T, MAX_LENGTH>,
    choice_page: ChoicePage<ChoiceFactorySimple<T>, T>,
    pub return_index: bool,
}

impl<T> SimpleChoice<T>
where
    T: StringType,
{
    pub fn new(str_choices: Vec<T, MAX_LENGTH>, carousel: bool) -> Self {
        let choices = ChoiceFactorySimple::new(str_choices.clone(), carousel);
        Self {
            choices: str_choices,
            choice_page: ChoicePage::new(choices).with_carousel(carousel),
            return_index: false,
        }
    }

    /// Show only the currently selected item, nothing left/right.
    pub fn with_only_one_item(mut self) -> Self {
        self.choice_page = self.choice_page.with_only_one_item(true);
        self
    }

    /// Show choices even when they do not fit entirely.
    pub fn with_show_incomplete(mut self) -> Self {
        self.choice_page = self.choice_page.with_incomplete(true);
        self
    }

    /// Returning chosen page index instead of the string result.
    pub fn with_return_index(mut self) -> Self {
        self.return_index = true;
        self
    }

    /// Translating the resulting index into actual string choice.
    pub fn result_by_index(&self, index: usize) -> &str {
        self.choices[index].as_ref()
    }
}

impl<T> Component for SimpleChoice<T>
where
    T: StringType,
{
    type Msg = SimpleChoiceMsg;

    fn place(&mut self, bounds: Rect) -> Rect {
        self.choice_page.place(bounds)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        match self.choice_page.event(ctx, event) {
            Some(ChoicePageMsg::Choice(page_counter)) => {
                Some(SimpleChoiceMsg::ResultIndex(page_counter))
            }
            _ => None,
        }
    }

    fn paint(&mut self) {
        self.choice_page.paint();
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
use super::super::{trace::ButtonTrace, ButtonAction, ButtonPos};
#[cfg(feature = "ui_debug")]
use heapless::String;

#[cfg(feature = "ui_debug")]
impl<T> ButtonTrace for SimpleChoice<T>
where
    T: StringType,
{
    fn get_btn_action(&self, pos: ButtonPos) -> String<25> {
        match pos {
            ButtonPos::Left => match self.choice_page.has_previous_choice() {
                true => ButtonAction::PrevPage.string(),
                false => ButtonAction::empty(),
            },
            ButtonPos::Right => match self.choice_page.has_next_choice() {
                true => ButtonAction::NextPage.string(),
                false => ButtonAction::empty(),
            },
            ButtonPos::Middle => {
                let current_index = self.choice_page.page_index();
                ButtonAction::select_item(self.choices[current_index].as_ref())
            }
        }
    }
}

#[cfg(feature = "ui_debug")]
impl<T> crate::trace::Trace for SimpleChoice<T>
where
    T: StringType,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("SimpleChoice");
        self.report_btn_actions(t);
        t.child("choice_page", &self.choice_page);
    }
}
