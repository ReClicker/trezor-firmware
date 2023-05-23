use crate::{
    strutil::StringType,
    trezorhal::random,
    ui::{
        component::{text::common::TextBox, Child, Component, ComponentExt, Event, EventCtx},
        display::Icon,
        geometry::Rect,
    },
};

use super::super::{
    theme, ButtonDetails, ButtonLayout, ChangingTextLine, ChoiceFactory, ChoiceItem, ChoicePage,
    ChoicePageMsg,
};
use core::marker::PhantomData;
use heapless::String;

pub enum PinEntryMsg {
    Confirmed,
    Cancelled,
}

const MAX_PIN_LENGTH: usize = 50;

const CHOICE_LENGTH: usize = 13;
const DELETE_INDEX: usize = 0;
const SHOW_INDEX: usize = 1;
const ENTER_INDEX: usize = 2;
const NUMBER_START_INDEX: usize = 3;
const CHOICES: [&str; CHOICE_LENGTH] = [
    "DELETE", "SHOW", "ENTER", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
];

struct ChoiceFactoryPIN<T>
where
    T: StringType,
{
    _phantom: PhantomData<T>,
}

impl<T> ChoiceFactoryPIN<T>
where
    T: StringType,
{
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> ChoiceFactory<T> for ChoiceFactoryPIN<T>
where
    T: StringType,
{
    type Item = ChoiceItem<T>;

    fn get(&self, choice_index: usize) -> ChoiceItem<T> {
        let choice_str = CHOICES[choice_index];

        let mut choice_item = ChoiceItem::new(choice_str, ButtonLayout::default_three_icons());

        // Action buttons have different middle button text
        if [DELETE_INDEX, SHOW_INDEX, ENTER_INDEX].contains(&(choice_index)) {
            let confirm_btn = ButtonDetails::armed_text("CONFIRM");
            choice_item.set_middle_btn(Some(confirm_btn));
        }

        // Adding icons for appropriate items
        match choice_index {
            DELETE_INDEX => {
                choice_item = choice_item.with_icon(Icon::new(theme::ICON_DELETE));
            }
            SHOW_INDEX => {
                choice_item = choice_item.with_icon(Icon::new(theme::ICON_EYE));
            }
            ENTER_INDEX => {
                choice_item = choice_item.with_icon(Icon::new(theme::ICON_TICK));
            }
            _ => {}
        }

        choice_item
    }

    fn count(&self) -> usize {
        CHOICE_LENGTH
    }
}

/// Component for entering a PIN.
pub struct PinEntry<T>
where
    T: StringType,
{
    choice_page: ChoicePage<ChoiceFactoryPIN<T>, T>,
    pin_line: Child<ChangingTextLine<String<MAX_PIN_LENGTH>>>,
    subprompt_line: Child<ChangingTextLine<T>>,
    prompt: T,
    show_real_pin: bool,
    textbox: TextBox<MAX_PIN_LENGTH>,
}

impl<T> PinEntry<T>
where
    T: StringType,
{
    pub fn new(prompt: T, subprompt: T) -> Self {
        let choices = ChoiceFactoryPIN::new();

        Self {
            // Starting at the digit 0
            choice_page: ChoicePage::new(choices)
                .with_initial_page_counter(NUMBER_START_INDEX)
                .with_carousel(true),
            pin_line: Child::new(ChangingTextLine::center_bold(String::from(
                prompt.clone().as_ref(),
            ))),
            subprompt_line: Child::new(ChangingTextLine::center_mono(subprompt)),
            prompt,
            show_real_pin: false,
            textbox: TextBox::empty(),
        }
    }

    fn append_new_digit(&mut self, ctx: &mut EventCtx, page_counter: usize) {
        let digit = CHOICES[page_counter];
        self.textbox.append_slice(ctx, digit);
    }

    fn delete_last_digit(&mut self, ctx: &mut EventCtx) {
        self.textbox.delete_last(ctx);
    }

    /// Performs overall update of the screen.
    fn update(&mut self, ctx: &mut EventCtx) {
        self.update_header_info(ctx);
        ctx.request_paint();
    }

    /// Update the header information - (sub)prompt and visible PIN.
    /// If PIN is empty, showing prompt in `pin_line` and sub-prompt in the
    /// `subprompt_line`. Otherwise disabling the `subprompt_line` and showing
    /// the PIN - either in real numbers or masked in asterisks.
    fn update_header_info(&mut self, ctx: &mut EventCtx) {
        let show_prompts = self.is_empty();

        let text = if show_prompts {
            String::from(self.prompt.as_ref())
        } else if self.show_real_pin {
            String::from(self.pin())
        } else {
            let mut dots: String<MAX_PIN_LENGTH> = String::new();
            for _ in 0..self.textbox.len() {
                unwrap!(dots.push_str("*"));
            }
            dots
        };

        // Force repaint of the whole header.
        // Putting the current text into the PIN line.
        self.pin_line.mutate(ctx, |ctx, pin_line| {
            pin_line.update_text(text);
            pin_line.request_complete_repaint(ctx);
        });
        // Showing subprompt only conditionally.
        self.subprompt_line.mutate(ctx, |ctx, subprompt_line| {
            subprompt_line.show_or_not(show_prompts);
            subprompt_line.request_complete_repaint(ctx);
        });
    }

    pub fn pin(&self) -> &str {
        self.textbox.content()
    }

    fn is_full(&self) -> bool {
        self.textbox.is_full()
    }

    fn is_empty(&self) -> bool {
        self.textbox.is_empty()
    }
}

impl<T> Component for PinEntry<T>
where
    T: StringType,
{
    type Msg = PinEntryMsg;

    fn place(&mut self, bounds: Rect) -> Rect {
        let pin_height = self.pin_line.inner().needed_height();
        let subtitle_height = self.subprompt_line.inner().needed_height();
        let (title_area, subtitle_and_choice_area) = bounds.split_top(pin_height);
        let (subtitle_area, choice_area) = subtitle_and_choice_area.split_top(subtitle_height);
        self.pin_line.place(title_area);
        self.subprompt_line.place(subtitle_area);
        self.choice_page.place(choice_area);
        bounds
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        // Any event when showing real PIN should hide it
        if self.show_real_pin {
            self.show_real_pin = false;
            self.update(ctx)
        }

        let msg = self.choice_page.event(ctx, event);
        if let Some(ChoicePageMsg::Choice(page_counter)) = msg {
            // Performing action under specific index or appending new digit
            match page_counter {
                DELETE_INDEX => {
                    self.delete_last_digit(ctx);
                    self.update(ctx);
                }
                SHOW_INDEX => {
                    self.show_real_pin = true;
                    self.update(ctx);
                }
                ENTER_INDEX => return Some(PinEntryMsg::Confirmed),
                _ => {
                    if !self.is_full() {
                        self.append_new_digit(ctx, page_counter);
                        // Choosing random digit to be shown next, but different
                        // from the current choice.
                        let new_page_counter = random::uniform_between_except(
                            NUMBER_START_INDEX as u32,
                            (CHOICE_LENGTH - 1) as u32,
                            page_counter as u32,
                        );
                        self.choice_page
                            .set_page_counter(ctx, new_page_counter as usize);
                        self.update(ctx);
                    }
                }
            }
        }
        None
    }

    fn paint(&mut self) {
        self.pin_line.paint();
        self.subprompt_line.paint();
        self.choice_page.paint();
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
use super::super::{trace::ButtonTrace, ButtonAction, ButtonPos};

#[cfg(feature = "ui_debug")]
impl<T> ButtonTrace for PinEntry<T>
where
    T: StringType,
{
    fn get_btn_action(&self, pos: ButtonPos) -> String<25> {
        match pos {
            ButtonPos::Left => ButtonAction::PrevPage.string(),
            ButtonPos::Right => ButtonAction::NextPage.string(),
            ButtonPos::Middle => {
                let current_index = self.choice_page.page_index();
                match current_index {
                    DELETE_INDEX => ButtonAction::Action("DELETE").string(),
                    SHOW_INDEX => ButtonAction::Action("SHOW").string(),
                    ENTER_INDEX => ButtonAction::Action("ENTER").string(),
                    _ => ButtonAction::select_item(CHOICES[current_index]),
                }
            }
        }
    }
}

#[cfg(feature = "ui_debug")]
impl<T> crate::trace::Trace for PinEntry<T>
where
    T: StringType,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("PinKeyboard");
        t.string("prompt", self.prompt.as_ref());
        let subprompt = self.subprompt_line.inner().get_text();
        if !subprompt.as_ref().is_empty() {
            t.string("subprompt", subprompt.as_ref());
        }
        t.string("pin", self.textbox.content());
        self.report_btn_actions(t);
        t.child("choice_page", &self.choice_page);
    }
}
