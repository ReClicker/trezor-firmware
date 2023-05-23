use crate::{
    strutil::StringType,
    ui::{
        component::{Child, Component, Event, EventCtx},
        geometry::{Insets, Rect},
    },
};

use super::{theme, ButtonController, ButtonControllerMsg, ButtonLayout, ButtonPos};

pub enum CancelInfoConfirmMsg {
    Cancelled,
    Info,
    Confirmed,
}

pub struct ShowMore<T, U>
where
    U: StringType,
{
    content: Child<T>,
    buttons: Child<ButtonController<U>>,
}

impl<T, U> ShowMore<T, U>
where
    T: Component,
    U: StringType,
{
    pub fn new(content: T) -> Self {
        let btn_layout = ButtonLayout::cancel_armed_text("CONFIRM".into(), "i".into());
        Self {
            content: Child::new(content),
            buttons: Child::new(ButtonController::new(btn_layout)),
        }
    }
}

impl<T, U> Component for ShowMore<T, U>
where
    T: Component,
    U: StringType,
{
    type Msg = CancelInfoConfirmMsg;

    fn place(&mut self, bounds: Rect) -> Rect {
        let (content_area, button_area) = bounds.split_bottom(theme::BUTTON_HEIGHT);
        let content_area = content_area.inset(Insets::top(1));
        self.content.place(content_area);
        self.buttons.place(button_area);
        bounds
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        let button_event = self.buttons.event(ctx, event);

        if let Some(ButtonControllerMsg::Triggered(pos)) = button_event {
            match pos {
                ButtonPos::Left => {
                    return Some(CancelInfoConfirmMsg::Cancelled);
                }
                ButtonPos::Middle => {
                    return Some(CancelInfoConfirmMsg::Confirmed);
                }
                ButtonPos::Right => {
                    return Some(CancelInfoConfirmMsg::Info);
                }
            }
        }
        None
    }

    fn paint(&mut self) {
        self.content.paint();
        self.buttons.paint();
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
use super::trace::ButtonTrace;

#[cfg(feature = "ui_debug")]
use crate::strutil::ShortString;

#[cfg(feature = "ui_debug")]
impl<T, U> ButtonTrace for ShowMore<T, U>
where
    T: crate::trace::Trace + Component,
    U: StringType,
{
    fn get_left_action(&self) -> ShortString {
        "Cancel".into()
    }

    fn get_middle_action(&self) -> ShortString {
        "Confirm".into()
    }

    fn get_right_action(&self) -> ShortString {
        "Info".into()
    }
}

#[cfg(feature = "ui_debug")]
impl<T, U> crate::trace::Trace for ShowMore<T, U>
where
    T: crate::trace::Trace + Component,
    U: StringType,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("ShowMore");
        self.report_btn_actions(t);
        t.child("buttons", &self.buttons);
        t.child("content", &self.content);
    }
}
