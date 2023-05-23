use crate::ui::{
    component::{text::common::TextBox, Child, Component, ComponentExt, Event, EventCtx},
    display::Icon,
    geometry::Rect,
    util::char_to_string,
};

use heapless::String;

use super::super::{
    theme, ButtonDetails, ButtonLayout, ChangingTextLine, ChoiceFactory, ChoiceItem, ChoicePage,
};

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

const DIGITS: &str = "0123456789";
const LOWERCASE_LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const SPECIAL_SYMBOLS: &str = "_<>.:@/|\\!()+%&-[]?{},\'`\"~$^=*#";

const MENU_LENGTH: usize = 8;
const SHOW_INDEX: usize = 0;
const CANCEL_DELETE_INDEX: usize = 1;
const ENTER_INDEX: usize = 2;
const LOWERCASE_INDEX: usize = 3;
const UPPERCASE_INDEX: usize = 4;
const DIGITS_INDEX: usize = 5;
const SPECIAL_INDEX: usize = 6;
const SPACE_INDEX: usize = 7;

// Menu text, action, icon data, middle button with CONFIRM
const MENU: [(&str, PassphraseAction, Option<&'static [u8]>, bool); MENU_LENGTH] = [
    ("SHOW", PassphraseAction::Show, Some(theme::ICON_EYE), true),
    (
        "CANCEL_OR_DELETE", // will be chosen dynamically
        PassphraseAction::CancelOrDelete,
        None,
        true,
    ),
    (
        "ENTER",
        PassphraseAction::Enter,
        Some(theme::ICON_TICK),
        true,
    ),
    (
        "abc",
        PassphraseAction::Category(ChoiceCategory::LowercaseLetter),
        None,
        false,
    ),
    (
        "ABC",
        PassphraseAction::Category(ChoiceCategory::UppercaseLetter),
        None,
        false,
    ),
    (
        "123",
        PassphraseAction::Category(ChoiceCategory::Digit),
        None,
        false,
    ),
    (
        "#$!",
        PassphraseAction::Category(ChoiceCategory::SpecialSymbol),
        None,
        false,
    ),
    (
        "SPACE",
        PassphraseAction::Character(' '),
        Some(theme::ICON_SPACE),
        false,
    ),
];

#[derive(Clone, Copy)]
enum PassphraseAction {
    Menu,
    Show,
    CancelOrDelete,
    Enter,
    Category(ChoiceCategory),
    Character(char),
}

/// Get a character at a specified index for a specified category.
fn get_char(current_category: &ChoiceCategory, index: usize) -> char {
    let group = match current_category {
        ChoiceCategory::LowercaseLetter => LOWERCASE_LETTERS,
        ChoiceCategory::UppercaseLetter => UPPERCASE_LETTERS,
        ChoiceCategory::Digit => DIGITS,
        ChoiceCategory::SpecialSymbol => SPECIAL_SYMBOLS,
        ChoiceCategory::Menu => unreachable!(),
    };
    unwrap!(group.chars().nth(index))
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

struct ChoiceFactoryPassphrase {
    current_category: ChoiceCategory,
    /// Used to either show DELETE or CANCEL
    is_empty: bool,
}

impl ChoiceFactoryPassphrase {
    fn new(current_category: ChoiceCategory, is_empty: bool) -> Self {
        Self {
            current_category,
            is_empty,
        }
    }

    /// MENU choices with accept and cancel hold-to-confirm side buttons.
    fn get_menu_item(&self, choice_index: usize) -> (ChoiceItem, PassphraseAction) {
        // More options for CANCEL/DELETE button
        let (mut text, action, mut icon, show_confirm) = MENU[choice_index];
        if matches!(action, PassphraseAction::CancelOrDelete) {
            if self.is_empty {
                text = "CANCEL";
                icon = Some(theme::ICON_CANCEL);
            } else {
                text = "DELETE";
                icon = Some(theme::ICON_DELETE);
            }
        }

        let mut menu_item = ChoiceItem::new(text, ButtonLayout::default_three_icons());

        // Action buttons have different middle button text
        if show_confirm {
            let confirm_btn = ButtonDetails::armed_text("CONFIRM");
            menu_item.set_middle_btn(Some(confirm_btn));
        }

        if let Some(icon) = icon {
            menu_item = menu_item.with_icon(Icon::new(icon));
        }
        (menu_item, action)
    }

    /// Character choices with a BACK to MENU choice at the end (visible from
    /// start) to return back
    fn get_character_item(&self, choice_index: usize) -> (ChoiceItem, PassphraseAction) {
        if is_menu_choice(&self.current_category, choice_index) {
            (
                ChoiceItem::new("BACK", ButtonLayout::arrow_armed_arrow("RETURN"))
                    .with_icon(Icon::new(theme::ICON_ARROW_BACK_UP)),
                PassphraseAction::Menu,
            )
        } else {
            let ch = get_char(&self.current_category, choice_index);
            (
                ChoiceItem::new(char_to_string(ch), ButtonLayout::default_three_icons()),
                PassphraseAction::Character(ch),
            )
        }
    }
}

impl ChoiceFactory for ChoiceFactoryPassphrase {
    type Action = PassphraseAction;

    fn count(&self) -> usize {
        let length = get_category_length(&self.current_category);
        // All non-MENU categories have an extra item for returning back to MENU
        match self.current_category {
            ChoiceCategory::Menu => length,
            _ => length + 1,
        }
    }
    fn get(&self, choice_index: usize) -> (ChoiceItem, Self::Action) {
        match self.current_category {
            ChoiceCategory::Menu => self.get_menu_item(choice_index),
            _ => self.get_character_item(choice_index),
        }
    }
}

/// Component for entering a passphrase.
pub struct PassphraseEntry {
    choice_page: ChoicePage<ChoiceFactoryPassphrase, PassphraseAction>,
    passphrase_dots: Child<ChangingTextLine<String<MAX_PASSPHRASE_LENGTH>>>,
    show_plain_passphrase: bool,
    textbox: TextBox<MAX_PASSPHRASE_LENGTH>,
    current_category: ChoiceCategory,
    menu_position: usize, // position in the menu so we can return back
}

impl PassphraseEntry {
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

impl Component for PassphraseEntry {
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

        if let Some(action) = self.choice_page.event(ctx, event) {
            match action {
                PassphraseAction::CancelOrDelete => {
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
                PassphraseAction::Enter => {
                    return Some(PassphraseEntryMsg::Confirmed);
                }
                PassphraseAction::Show => {
                    self.show_plain_passphrase = true;
                    self.update_passphrase_dots(ctx);
                    ctx.request_paint();
                }
                PassphraseAction::Category(category) => {
                    self.menu_position = self.choice_page.page_index();
                    self.current_category = category;
                    self.show_category_page(ctx);
                    ctx.request_paint();
                }
                PassphraseAction::Menu => {
                    self.current_category = ChoiceCategory::Menu;
                    self.show_menu_page(ctx);
                    ctx.request_paint();
                }
                PassphraseAction::Character(ch) if !self.is_full() => {
                    self.append_char(ctx, ch);
                    self.update_passphrase_dots(ctx);
                    ctx.request_paint();
                }
                _ => {}
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
use super::super::trace::ButtonTrace;

#[cfg(feature = "ui_debug")]
use crate::strutil::ShortString;

#[cfg(feature = "ui_debug")]
impl ChoiceCategory {
    fn string(&self) -> String<25> {
        match self {
            ChoiceCategory::Menu => "MENU".into(),
            ChoiceCategory::LowercaseLetter => MENU[LOWERCASE_INDEX].0.into(),
            ChoiceCategory::UppercaseLetter => MENU[UPPERCASE_INDEX].0.into(),
            ChoiceCategory::Digit => MENU[DIGITS_INDEX].0.into(),
            ChoiceCategory::SpecialSymbol => MENU[SPECIAL_INDEX].0.into(),
        }
    }
}

#[cfg(feature = "ui_debug")]
impl ButtonTrace for PassphraseEntry {
    fn get_middle_action(&self) -> ShortString {
        let current_index = self.choice_page.page_index();
        match &self.current_category {
            ChoiceCategory::Menu => MENU[current_index].0.into(),
            _ => {
                // There is "MENU" option at the end
                match self.choice_page.has_next_choice() {
                    false => "BACK".into(),
                    true => {
                        let ch = get_char(&self.current_category, current_index);
                        char_to_string(ch)
                    }
                }
            }
        }
    }
}

#[cfg(feature = "ui_debug")]
impl crate::trace::Trace for PassphraseEntry {
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("PassphraseKeyboard");
        t.string("passphrase", self.textbox.content());
        t.string("current_category", &self.current_category.string());
        self.report_btn_actions(t);
        t.child("choice_page", &self.choice_page);
    }
}
