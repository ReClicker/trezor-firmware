use crate::{
    strutil::StringType,
    ui::{
        component::{Component, Event, EventCtx},
        geometry::Rect,
    },
};

use super::super::{ButtonLayout, ChoiceFactory, ChoiceItem, ChoicePage};
use heapless::Vec;

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

impl<T> ChoiceFactory for ChoiceFactorySimple<T>
where
    T: StringType,
{
    type Action = usize;

    fn count(&self) -> usize {
        self.choices.len()
    }

    fn get(&self, choice_index: usize) -> (ChoiceItem, Self::Action) {
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

        (choice_item, choice_index)
    }
}

/// Simple wrapper around `ChoicePage` that allows for
/// inputting a list of values and receiving the chosen one.
pub struct SimpleChoice<T>
where
    T: StringType + Clone,
{
    choices: Vec<T, MAX_LENGTH>,
    choice_page: ChoicePage<ChoiceFactorySimple<T>, usize>,
    pub return_index: bool,
}

impl<T> SimpleChoice<T>
where
    T: StringType + Clone,
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
    T: StringType + Clone,
{
    type Msg = usize;

    fn place(&mut self, bounds: Rect) -> Rect {
        self.choice_page.place(bounds)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        self.choice_page.event(ctx, event)
    }

    fn paint(&mut self) {
        self.choice_page.paint();
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
use super::super::trace::ButtonTrace;
#[cfg(feature = "ui_debug")]
use crate::strutil::ShortString;

#[cfg(feature = "ui_debug")]
impl<T> ButtonTrace for SimpleChoice<T>
where
    T: StringType + Clone,
{
    fn get_left_action(&self) -> ShortString {
        match self.choice_page.has_previous_choice() {
            true => "Prev".into(),
            false => "None".into(),
        }
    }

    fn get_middle_action(&self) -> ShortString {
        let current_index = self.choice_page.page_index();
        self.choices[current_index].as_ref().into()
    }

    fn get_right_action(&self) -> ShortString {
        match self.choice_page.has_previous_choice() {
            true => "Next".into(),
            false => "None".into(),
        }
    }
}

#[cfg(feature = "ui_debug")]
impl<T> crate::trace::Trace for SimpleChoice<T>
where
    T: StringType + Clone,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("SimpleChoice");
        self.report_btn_actions(t);
        t.child("choice_page", &self.choice_page);
    }
}
