use crate::{
    strutil::StringType,
    trezorhal::wordlist::Wordlist,
    ui::{
        component::{text::common::TextBox, Child, Component, ComponentExt, Event, EventCtx},
        display::Icon,
        geometry::Rect,
        util::char_to_string,
    },
};

use super::super::{
    theme, trace::ButtonTrace, ButtonLayout, ChangingTextLine, ChoiceFactory, ChoiceItem,
    ChoicePage, ChoicePageMsg,
};
use heapless::{String, Vec};

pub enum WordlistEntryMsg {
    WordIndex(usize),
}

const MAX_WORD_LENGTH: usize = 10;
const MAX_LETTERS_LENGTH: usize = 26;

/// Offer words when there will be fewer of them than this
const OFFER_WORDS_THRESHOLD: usize = 10;

/// Where will be the DELETE option - at the first position
const DELETE_INDEX: usize = 0;
/// Which index will be used at the beginning.
/// (Accounts for DELETE to be at index 0)
const INITIAL_PAGE_COUNTER: usize = DELETE_INDEX + 1;

const PROMPT: &str = "_";

/// Type of the wordlist, deciding the list of words to be used
pub enum WordlistType {
    Bip39,
    Slip39,
}

/// We are offering either letters or words.
enum ChoiceFactoryWordlist {
    Letters(Vec<char, MAX_LETTERS_LENGTH>),
    Words(Vec<&'static str, OFFER_WORDS_THRESHOLD>),
}

impl ChoiceFactoryWordlist {
    fn letters(letter_choices: Vec<char, MAX_LETTERS_LENGTH>) -> Self {
        Self::Letters(letter_choices)
    }

    fn words(word_choices: Vec<&'static str, OFFER_WORDS_THRESHOLD>) -> Self {
        Self::Words(word_choices)
    }

    /// NOTE: done to remediate some type-inconsistencies
    /// with using self.count() in self.get()
    fn self_count(&self) -> usize {
        // Accounting for the DELETE option
        match self {
            Self::Letters(letter_choices) => letter_choices.len() + 1,
            Self::Words(word_choices) => word_choices.len() + 1,
        }
    }
}

impl<T> ChoiceFactory<T> for ChoiceFactoryWordlist
where
    T: StringType,
{
    type Item = ChoiceItem<T>;

    fn count(&self) -> usize {
        self.self_count()
    }

    fn get(&self, choice_index: usize) -> ChoiceItem<T> {
        // Letters have a carousel, words do not
        // Putting DELETE as the first option in both cases
        // (is a requirement for WORDS, doing it for LETTERS as well to unite it)
        match self {
            Self::Letters(letter_choices) => {
                if choice_index == DELETE_INDEX {
                    ChoiceItem::new("DELETE", ButtonLayout::arrow_armed_arrow("CONFIRM".into()))
                        .with_icon(Icon::new(theme::ICON_DELETE))
                } else {
                    let letter = letter_choices[choice_index - 1];
                    ChoiceItem::new(
                        char_to_string::<1>(letter),
                        ButtonLayout::default_three_icons(),
                    )
                }
            }
            Self::Words(word_choices) => {
                if choice_index == DELETE_INDEX {
                    ChoiceItem::new("DELETE", ButtonLayout::none_armed_arrow("CONFIRM".into()))
                        .with_icon(Icon::new(theme::ICON_DELETE))
                } else {
                    let word = word_choices[choice_index - 1];
                    let mut item = ChoiceItem::new(word, ButtonLayout::default_three_icons());
                    if choice_index == self.self_count() - 1 {
                        item.set_right_btn(None);
                    }
                    item
                }
            }
        }
    }
}

/// Component for entering a mnemonic from a wordlist - BIP39 or SLIP39.
pub struct WordlistEntry<T>
where
    T: StringType,
{
    choice_page: ChoicePage<ChoiceFactoryWordlist, T>,
    chosen_letters: Child<ChangingTextLine<String<{ MAX_WORD_LENGTH + 1 }>>>,
    letter_choices: Vec<char, MAX_LETTERS_LENGTH>,
    textbox: TextBox<MAX_WORD_LENGTH>,
    offer_words: bool,
    words_list: Wordlist,
    wordlist_type: WordlistType,
}

impl<T> WordlistEntry<T>
where
    T: StringType,
{
    pub fn new(wordlist_type: WordlistType) -> Self {
        let words_list = Self::get_fresh_wordlist(&wordlist_type);
        let letter_choices: Vec<char, MAX_LETTERS_LENGTH> =
            words_list.get_available_letters("").collect();
        let choices = ChoiceFactoryWordlist::letters(letter_choices.clone());

        Self {
            // Starting at second page because of DELETE option
            choice_page: ChoicePage::new(choices)
                .with_incomplete(true)
                .with_carousel(true)
                .with_initial_page_counter(INITIAL_PAGE_COUNTER),
            chosen_letters: Child::new(ChangingTextLine::center_mono(String::from(PROMPT))),
            letter_choices,
            textbox: TextBox::empty(),
            offer_words: false,
            words_list,
            wordlist_type,
        }
    }

    /// Get appropriate wordlist with all possible words
    fn get_fresh_wordlist(wordlist_type: &WordlistType) -> Wordlist {
        match wordlist_type {
            WordlistType::Bip39 => Wordlist::bip39(),
            WordlistType::Slip39 => Wordlist::slip39(),
        }
    }

    /// Gets up-to-date choices for letters or words.
    fn get_current_choices(&mut self) -> ChoiceFactoryWordlist {
        // Narrowing the word list
        self.words_list = self.words_list.filter_prefix(self.textbox.content());

        // Offering words when there is only a few of them
        // Otherwise getting relevant letters
        if self.words_list.len() < OFFER_WORDS_THRESHOLD {
            self.offer_words = true;
            let word_choices = self.words_list.iter().collect();
            ChoiceFactoryWordlist::words(word_choices)
        } else {
            self.offer_words = false;
            self.letter_choices = self
                .words_list
                .get_available_letters(self.textbox.content())
                .collect();
            ChoiceFactoryWordlist::letters(self.letter_choices.clone())
        }
    }

    /// Updates the whole page.
    fn update(&mut self, ctx: &mut EventCtx) {
        self.update_chosen_letters(ctx);
        let new_choices = self.get_current_choices();
        // Not using carousel in case of words, as that looks weird in case
        // there is only one word to choose from.
        self.choice_page.reset(
            ctx,
            new_choices,
            Some(INITIAL_PAGE_COUNTER),
            !self.offer_words,
        );
        ctx.request_paint();
    }

    /// Reflects currently chosen letters in the textbox.
    fn update_chosen_letters(&mut self, ctx: &mut EventCtx) {
        let text = build_string!({ MAX_WORD_LENGTH + 1 }, self.textbox.content(), PROMPT);
        self.chosen_letters.mutate(ctx, |ctx, chosen_letters| {
            chosen_letters.update_text(text);
            chosen_letters.request_complete_repaint(ctx);
        });
    }

    fn append_letter(&mut self, ctx: &mut EventCtx, letter: char) {
        self.textbox.append(ctx, letter);
    }

    fn delete_last_letter(&mut self, ctx: &mut EventCtx) {
        self.textbox.delete_last(ctx);
    }

    fn reset_wordlist(&mut self) {
        self.words_list = Self::get_fresh_wordlist(&self.wordlist_type);
    }

    /// Translating the resulting index into actual word.
    pub fn word_by_index(&self, index: usize) -> &str {
        self.words_list.get(index).unwrap_or_default()
    }
}

impl<T> Component for WordlistEntry<T>
where
    T: StringType,
{
    type Msg = WordlistEntryMsg;

    fn place(&mut self, bounds: Rect) -> Rect {
        let letters_area_height = self.chosen_letters.inner().needed_height();
        let (letters_area, choice_area) = bounds.split_top(letters_area_height);
        self.chosen_letters.place(letters_area);
        self.choice_page.place(choice_area);
        bounds
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        let msg = self.choice_page.event(ctx, event);
        if let Some(ChoicePageMsg::Choice(page_counter)) = msg {
            if page_counter == DELETE_INDEX {
                // Clicked DELETE.
                // Deleting last letter, updating wordlist and updating choices.
                self.delete_last_letter(ctx);
                self.reset_wordlist();
                self.update(ctx);
            } else {
                // Clicked SELECT.
                // When we already offer words, return the index of chosen word.
                // Otherwise, resetting the choice page with up-to-date choices.
                let index = page_counter - 1;
                if self.offer_words {
                    return Some(WordlistEntryMsg::WordIndex(index));
                } else {
                    let new_letter = self.letter_choices[index];
                    self.append_letter(ctx, new_letter);
                    self.update(ctx);
                }
            }
        }

        None
    }

    fn paint(&mut self) {
        self.chosen_letters.paint();
        self.choice_page.paint();
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
use super::super::{ButtonAction, ButtonPos};
#[cfg(feature = "ui_debug")]
use crate::ui::util;

#[cfg(feature = "ui_debug")]
impl<T> ButtonTrace for WordlistEntry<T>
where
    T: StringType,
{
    fn get_btn_action(&self, pos: ButtonPos) -> String<25> {
        match pos {
            ButtonPos::Left => ButtonAction::PrevPage.string(),
            ButtonPos::Right => ButtonAction::NextPage.string(),
            ButtonPos::Middle => {
                let current_index = self.choice_page.page_index();
                let choice: String<10> = if current_index == DELETE_INDEX {
                    String::from("DELETE")
                } else {
                    let index = current_index - 1;
                    if self.offer_words {
                        self.words_list.get(index).unwrap_or_default().into()
                    } else {
                        util::char_to_string(self.letter_choices[index])
                    }
                };
                ButtonAction::select_item(choice)
            }
        }
    }
}

#[cfg(feature = "ui_debug")]
impl<T> crate::trace::Trace for WordlistEntry<T>
where
    T: StringType,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        match self.wordlist_type {
            WordlistType::Bip39 => t.component("Bip39Entry"),
            WordlistType::Slip39 => t.component("Slip39Entry"),
        }
        t.string("textbox", self.textbox.content());

        self.report_btn_actions(t);

        if self.offer_words {
            t.in_list("word_choices", &|list_t| {
                for word in self.words_list.iter() {
                    list_t.string(word);
                }
            });
        } else {
            t.in_list("letter_choices", &|list_t| {
                for ch in &self.letter_choices {
                    list_t.string(&util::char_to_string::<1>(*ch));
                }
            });
        }

        t.child("choice_page", &self.choice_page);
    }
}
