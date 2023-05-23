mod button;
mod button_controller;
mod common;
mod hold_to_confirm;
mod input_methods;
mod loader;
mod result;
mod welcome_screen;

#[cfg(feature = "ui_debug")]
use super::trace;
use super::{constant, theme};
pub use button::{
    Button, ButtonAction, ButtonActions, ButtonContent, ButtonDetails, ButtonLayout, ButtonMsg,
    ButtonPos, ButtonStyle, ButtonStyleSheet,
};
pub use button_controller::{ButtonController, ButtonControllerMsg};
pub use hold_to_confirm::{HoldToConfirm, HoldToConfirmMsg};
pub use input_methods::{
    choice::{Choice, ChoiceFactory, ChoicePage},
    choice_item::ChoiceItem,
};
pub use loader::{Loader, LoaderMsg, LoaderStyle, LoaderStyleSheet};
pub use result::ResultScreen;
pub use welcome_screen::WelcomeScreen;

mod address_details;
mod changing_text;
mod coinjoin_progress;
mod flow;
mod flow_pages;
mod flow_pages_helpers;
mod frame;
mod homescreen;
mod no_btn_dialog;
mod page;
mod progress;
mod result_anim;
mod result_popup;
mod scrollbar;
mod share_words;
mod show_more;
mod title;

pub use address_details::{AddressDetails, AddressDetailsMsg};

pub use changing_text::ChangingTextLine;
pub use coinjoin_progress::CoinJoinProgress;
pub use flow::{Flow, FlowMsg};
pub use flow_pages::{FlowPages, Page};
pub use frame::{Frame, ScrollableContent, ScrollableFrame};
pub use homescreen::{Homescreen, HomescreenMsg, Lockscreen};
pub use input_methods::{
    number_input::NumberInput,
    passphrase::{PassphraseEntry, PassphraseEntryMsg},
    pin::{PinEntry, PinEntryMsg},
    simple_choice::SimpleChoice,
    wordlist::{WordlistEntry, WordlistType},
};
pub use no_btn_dialog::NoBtnDialog;
pub use page::ButtonPage;
pub use progress::Progress;
pub use result_anim::{ResultAnim, ResultAnimMsg};
pub use result_popup::{ResultPopup, ResultPopupMsg};
pub use scrollbar::ScrollBar;
pub use share_words::ShareWords;
pub use show_more::{CancelInfoConfirmMsg, ShowMore};
