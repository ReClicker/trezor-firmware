use crate::{
    strutil::StringType,
    ui::{
        component::{text::common::TextBox, Child, Component, ComponentExt, Event, EventCtx},
        display::Icon,
        geometry::Rect,
        util::char_to_string,
    },
};

use super::super::{
    theme, ButtonDetails, ButtonLayout, ChangingTextLine, ChoiceFactory, ChoiceItem, ChoicePage,
    ChoicePageMsg,
};
use core::marker::PhantomData;
use heapless::String;

pub enum PassphraseEntryMsg {
    Confirmed,
    Cancelled,
}

/// Defines the choices currently available on the screen
#[derive(PartialEq, Clone, Copy)]
enum ChoiceCategory {
    Menu,
    LowercaseLetter,
    UppercaseLetter,
    Digit,
    SpecialSymbol,
}

const MAX_PASSPHRASE_LENGTH: usize = 50;

const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const LOWERCASE_LETTERS: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const UPPERCASE_LETTERS: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const SPECIAL_SYMBOLS: [char; 32] = [
    '_', '<', '>', '.', ':', '@', '/', '|', '\\', '!', '(', ')', '+', '%', '&', '-', '[', ']', '?',
    '{', '}', ',', '\'', '`', ';', '"', '~', '$', '^', '=', '*', '#',
];
const MENU_LENGTH: usize = 8;
const SHOW_INDEX: usize = 0;
const CANCEL_DELETE_INDEX: usize = 1;
const ENTER_INDEX: usize = 2;
const LOWERCASE_INDEX: usize = 3;
const UPPERCASE_INDEX: usize = 4;
const DIGITS_INDEX: usize = 5;
const SPECIAL_INDEX: usize = 6;
const SPACE_INDEX: usize = 7;
const MENU: [&str; MENU_LENGTH] = [
    "SHOW",
    "CANCEL_OR_DELETE", // will be chosen dynamically
    "ENTER",
    "abc",
    "ABC",
    "123",
    "#$!",
    "SPACE",
];

/// Get a character at a specified index for a specified category.
fn get_char(current_category: &ChoiceCategory, index: usize) -> char {
    match current_category {
        ChoiceCategory::LowercaseLetter => LOWERCASE_LETTERS[index],
        ChoiceCategory::UppercaseLetter => UPPERCASE_LETTERS[index],
        ChoiceCategory::Digit => DIGITS[index],
        ChoiceCategory::SpecialSymbol => SPECIAL_SYMBOLS[index],
        ChoiceCategory::Menu => unreachable!(),
    }
}

/// Return category from menu based on page index.
fn get_category_from_menu(page_index: usize) -> ChoiceCategory {
    match page_index {
        LOWERCASE_INDEX => ChoiceCategory::LowercaseLetter,
        UPPERCASE_INDEX => ChoiceCategory::UppercaseLetter,
        DIGITS_INDEX => ChoiceCategory::Digit,
        SPECIAL_INDEX => ChoiceCategory::SpecialSymbol,
        _ => unreachable!(),
    }
}

/// How many choices are available for a specified category.
/// (does not count the extra MENU choice for characters)
fn get_category_length(current_category: &ChoiceCategory) -> usize {
    match current_category {
        ChoiceCategory::LowercaseLetter => LOWERCASE_LETTERS.len(),
        ChoiceCategory::UppercaseLetter => UPPERCASE_LETTERS.len(),
        ChoiceCategory::Digit => DIGITS.len(),
        ChoiceCategory::SpecialSymbol => SPECIAL_SYMBOLS.len(),
        ChoiceCategory::Menu => MENU.len(),
    }
}

/// Whether this index is the MENU index - the last one in the list.
fn is_menu_choice(current_category: &ChoiceCategory, page_index: usize) -> bool {
    if let ChoiceCategory::Menu = current_category {
        unreachable!()
    }
    let category_length = get_category_length(current_category);
    page_index == category_length
}

struct ChoiceFactoryPassphrase<T> {
    current_category: ChoiceCategory,
    /// Used to either show DELETE or CANCEL
    is_empty: bool,
    _phantom: PhantomData<T>,
}

impl<T> ChoiceFactoryPassphrase<T>
where
    T: StringType,
{
    fn new(current_category: ChoiceCategory, is_empty: bool) -> Self {
        Self {
            current_category,
            is_empty,
            _phantom: PhantomData,
        }
    }

    /// MENU choices with accept and cancel hold-to-confirm side buttons.
    fn get_menu_item(&self, choice_index: usize) -> ChoiceItem<T> {
        // More options for CANCEL/DELETE button
        let choice = if choice_index == CANCEL_DELETE_INDEX {
            if self.is_empty {
                "CANCEL"
            } else {
                "DELETE"
            }
        } else {
            MENU[choice_index]
        };

        let mut menu_item = ChoiceItem::new(
            String::<50>::from(choice),
            ButtonLayout::default_three_icons(),
        );

        // Action buttons have different middle button text
        if [CANCEL_DELETE_INDEX, SHOW_INDEX, ENTER_INDEX].contains(&choice_index) {
            let confirm_btn = ButtonDetails::armed_text("CONFIRM".into());
            menu_item.set_middle_btn(Some(confirm_btn));
        }

        // Including icons for some items.
        match choice_index {
            CANCEL_DELETE_INDEX => {
                if self.is_empty {
                    menu_item = menu_item.with_icon(Icon::new(theme::ICON_CANCEL));
                } else {
                    menu_item = menu_item.with_icon(Icon::new(theme::ICON_DELETE));
                }
            }
            SHOW_INDEX => {
                menu_item = menu_item.with_icon(Icon::new(theme::ICON_EYE));
            }
            ENTER_INDEX => {
                menu_item = menu_item.with_icon(Icon::new(theme::ICON_TICK));
            }
            SPACE_INDEX => {
                menu_item = menu_item.with_icon(Icon::new(theme::ICON_SPACE));
            }
            _ => {}
        }

        menu_item
    }

    /// Character choices with a BACK to MENU choice at the end (visible from
    /// start) to return back
    fn get_character_item(&self, choice_index: usize) -> ChoiceItem<T> {
        if is_menu_choice(&self.current_category, choice_index) {
            ChoiceItem::new("BACK", ButtonLayout::arrow_armed_arrow("RETURN".into()))
                .with_icon(Icon::new(theme::ICON_ARROW_BACK_UP))
        } else {
            let ch = get_char(&self.current_category, choice_index);
            ChoiceItem::new(char_to_string::<1>(ch), ButtonLayout::default_three_icons())
        }
    }
}

impl<T> ChoiceFactory<T> for ChoiceFactoryPassphrase<T>
where
    T: StringType,
{
    type Item = ChoiceItem<T>;
    fn count(&self) -> usize {
        let length = get_category_length(&self.current_category);
        // All non-MENU categories have an extra item for returning back to MENU
        match self.current_category {
            ChoiceCategory::Menu => length,
            _ => length + 1,
        }
    }
    fn get(&self, choice_index: usize) -> ChoiceItem<T> {
        match self.current_category {
            ChoiceCategory::Menu => self.get_menu_item(choice_index),
            _ => self.get_character_item(choice_index),
        }
    }
}

/// Component for entering a passphrase.
pub struct PassphraseEntry<T>
where
    T: StringType,
{
    choice_page: ChoicePage<ChoiceFactoryPassphrase<T>, T>,
    passphrase_dots: Child<ChangingTextLine<String<MAX_PASSPHRASE_LENGTH>>>,
    show_plain_passphrase: bool,
    textbox: TextBox<MAX_PASSPHRASE_LENGTH>,
    current_category: ChoiceCategory,
    menu_position: usize, // position in the menu so we can return back
}

impl<T> PassphraseEntry<T>
where
    T: StringType,
{
    pub fn new() -> Self {
        Self {
            choice_page: ChoicePage::new(ChoiceFactoryPassphrase::new(ChoiceCategory::Menu, true))
                .with_carousel(true)
                .with_initial_page_counter(LOWERCASE_INDEX),
            passphrase_dots: Child::new(ChangingTextLine::center_mono(String::new())),
            show_plain_passphrase: false,
            textbox: TextBox::empty(),
            current_category: ChoiceCategory::Menu,
            menu_position: 0,
        }
    }

    fn update_passphrase_dots(&mut self, ctx: &mut EventCtx) {
        let text_to_show = if self.show_plain_passphrase {
            String::from(self.passphrase())
        } else {
            let mut dots: String<MAX_PASSPHRASE_LENGTH> = String::new();
            for _ in 0..self.textbox.len() {
                unwrap!(dots.push_str("*"));
            }
            dots
        };
        self.passphrase_dots.mutate(ctx, |ctx, passphrase_dots| {
            passphrase_dots.update_text(text_to_show);
            passphrase_dots.request_complete_repaint(ctx);
        });
    }

    fn append_char(&mut self, ctx: &mut EventCtx, ch: char) {
        self.textbox.append(ctx, ch);
    }

    fn delete_last_digit(&mut self, ctx: &mut EventCtx) {
        self.textbox.delete_last(ctx);
    }

    /// Displaying the MENU
    fn show_menu_page(&mut self, ctx: &mut EventCtx) {
        let menu_choices = ChoiceFactoryPassphrase::new(ChoiceCategory::Menu, self.is_empty());
        // Going back to the last MENU position before showing the MENU
        self.choice_page
            .reset(ctx, menu_choices, Some(self.menu_position), true);
    }

    /// Displaying the character category
    fn show_category_page(&mut self, ctx: &mut EventCtx) {
        let category_choices = ChoiceFactoryPassphrase::new(self.current_category, self.is_empty());
        self.choice_page.reset(ctx, category_choices, Some(0), true);
    }

    pub fn passphrase(&self) -> &str {
        self.textbox.content()
    }

    fn is_empty(&self) -> bool {
        self.textbox.is_empty()
    }

    fn is_full(&self) -> bool {
        self.textbox.is_full()
    }
}

impl<T> Component for PassphraseEntry<T>
where
    T: StringType,
{
    type Msg = PassphraseEntryMsg;

    fn place(&mut self, bounds: Rect) -> Rect {
        let passphrase_area_height = self.passphrase_dots.inner().needed_height();
        let (passphrase_area, choice_area) = bounds.split_top(passphrase_area_height);
        self.passphrase_dots.place(passphrase_area);
        self.choice_page.place(choice_area);
        bounds
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        // Any event when showing real passphrase should hide it
        if self.show_plain_passphrase {
            self.show_plain_passphrase = false;
            self.update_passphrase_dots(ctx);
        }

        if let Some(ChoicePageMsg::Choice(page_counter)) = self.choice_page.event(ctx, event) {
            // Event handling based on MENU vs CATEGORY
            if self.current_category == ChoiceCategory::Menu {
                // Going to new category, applying some action or returning the result
                match page_counter {
                    CANCEL_DELETE_INDEX => {
                        if self.is_empty() {
                            return Some(PassphraseEntryMsg::Cancelled);
                        } else {
                            self.delete_last_digit(ctx);
                            self.update_passphrase_dots(ctx);
                            if self.is_empty() {
                                // Allowing for DELETE/CANCEL change
                                self.menu_position = CANCEL_DELETE_INDEX;
                                self.show_menu_page(ctx);
                            }
                            ctx.request_paint();
                        }
                    }
                    ENTER_INDEX => {
                        return Some(PassphraseEntryMsg::Confirmed);
                    }
                    SHOW_INDEX => {
                        self.show_plain_passphrase = true;
                        self.update_passphrase_dots(ctx);
                        ctx.request_paint();
                    }
                    SPACE_INDEX => {
                        if !self.is_full() {
                            self.append_char(ctx, ' ');
                            self.update_passphrase_dots(ctx);
                            ctx.request_paint();
                        }
                    }
                    _ => {
                        self.menu_position = page_counter;
                        self.current_category = get_category_from_menu(page_counter);
                        self.show_category_page(ctx);
                        ctx.request_paint();
                    }
                }
            } else {
                // Coming back to MENU or adding new character
                if is_menu_choice(&self.current_category, page_counter) {
                    self.current_category = ChoiceCategory::Menu;
                    self.show_menu_page(ctx);
                    ctx.request_paint();
                } else if !self.is_full() {
                    let new_char = get_char(&self.current_category, page_counter);
                    self.append_char(ctx, new_char);
                    self.update_passphrase_dots(ctx);
                    ctx.request_paint();
                }
            }
        }

        None
    }

    fn paint(&mut self) {
        self.passphrase_dots.paint();
        self.choice_page.paint();
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
use super::super::{trace::ButtonTrace, ButtonAction, ButtonPos};
#[cfg(feature = "ui_debug")]
use crate::ui::util;

#[cfg(feature = "ui_debug")]
impl ChoiceCategory {
    fn string(&self) -> String<25> {
        match self {
            ChoiceCategory::Menu => "MENU".into(),
            ChoiceCategory::LowercaseLetter => MENU[LOWERCASE_INDEX].into(),
            ChoiceCategory::UppercaseLetter => MENU[UPPERCASE_INDEX].into(),
            ChoiceCategory::Digit => MENU[DIGITS_INDEX].into(),
            ChoiceCategory::SpecialSymbol => MENU[SPECIAL_INDEX].into(),
        }
    }
}

#[cfg(feature = "ui_debug")]
impl<T> ButtonTrace for PassphraseEntry<T>
where
    T: StringType,
{
    fn get_btn_action(&self, pos: ButtonPos) -> String<25> {
        match pos {
            ButtonPos::Left => ButtonAction::PrevPage.string(),
            ButtonPos::Right => ButtonAction::NextPage.string(),
            ButtonPos::Middle => {
                let current_index = self.choice_page.page_index();
                match &self.current_category {
                    ChoiceCategory::Menu => ButtonAction::select_item(MENU[current_index]),
                    _ => {
                        // There is "MENU" option at the end
                        match self.choice_page.has_next_choice() {
                            false => ButtonAction::Action("BACK").string(),
                            true => {
                                let ch = match &self.current_category {
                                    ChoiceCategory::LowercaseLetter => {
                                        LOWERCASE_LETTERS[current_index]
                                    }
                                    ChoiceCategory::UppercaseLetter => {
                                        UPPERCASE_LETTERS[current_index]
                                    }
                                    ChoiceCategory::Digit => DIGITS[current_index],
                                    ChoiceCategory::SpecialSymbol => SPECIAL_SYMBOLS[current_index],
                                    ChoiceCategory::Menu => unreachable!(),
                                };
                                ButtonAction::select_item(util::char_to_string::<1>(ch))
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "ui_debug")]
impl<T> crate::trace::Trace for PassphraseEntry<T>
where
    T: StringType,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("PassphraseKeyboard");
        t.string("passphrase", self.textbox.content());
        t.string("current_category", &self.current_category.string());
        self.report_btn_actions(t);
        t.child("choice_page", &self.choice_page);
    }
}
