use crate::{
    strutil::StringType,
    ui::{
        component::{Child, Component, ComponentExt, Event, EventCtx, Pad, Paginate},
        geometry::Rect,
    },
};

use super::{
    scrollbar::SCROLLBAR_SPACE, theme, title::Title, ButtonAction, ButtonController,
    ButtonControllerMsg, ButtonLayout, ButtonPos, FlowPages, Page, ScrollBar,
};

/// To be returned directly from Flow.
pub enum FlowMsg {
    Confirmed,
    ConfirmedIndex(usize),
    Cancelled,
    Info,
}

pub struct Flow<F, const M: usize, T>
where
    F: Fn(usize) -> Page<M, T>,
    T: StringType,
{
    /// Function to get pages from
    pages: FlowPages<F, M, T>,
    /// Instance of the current Page
    current_page: Page<M, T>,
    /// Title being shown at the top in bold
    title: Option<Title<T>>,
    scrollbar: Child<ScrollBar>,
    content_area: Rect,
    title_area: Rect,
    pad: Pad,
    buttons: Child<ButtonController<T>>,
    page_counter: usize,
    return_confirmed_index: bool,
}

impl<F, const M: usize, T> Flow<F, M, T>
where
    F: Fn(usize) -> Page<M, T>,
    T: StringType,
{
    pub fn new(pages: FlowPages<F, M, T>) -> Self {
        let current_page = pages.get(0);
        Self {
            pages,
            current_page,
            title: None,
            content_area: Rect::zero(),
            title_area: Rect::zero(),
            scrollbar: Child::new(ScrollBar::to_be_filled_later()),
            pad: Pad::with_background(theme::BG),
            // Setting empty layout for now, we do not yet know how many sub-pages the first page
            // has. Initial button layout will be set in `place()` after we can call
            // `content.page_count()`.
            buttons: Child::new(ButtonController::new(ButtonLayout::empty())),
            page_counter: 0,
            return_confirmed_index: false,
        }
    }

    /// Adding a common title to all pages. The title will not be colliding
    /// with the page content, as the content will be offset.
    pub fn with_common_title(mut self, title: T) -> Self {
        self.title = Some(Title::new(title));
        self
    }

    /// Causing the Flow to return the index of the page that was confirmed.
    pub fn with_return_confirmed_index(mut self) -> Self {
        self.return_confirmed_index = true;
        self
    }

    /// Getting new current page according to page counter.
    /// Also updating the possible title and moving the scrollbar to correct
    /// position.
    fn change_current_page(&mut self, ctx: &mut EventCtx) {
        self.current_page = self.pages.get(self.page_counter);
        if self.title.is_some() {
            if let Some(title) = self.current_page.title() {
                self.title = Some(Title::new(title));
                self.title.place(self.title_area);
            }
        }
        let scrollbar_active_index = self
            .pages
            .scrollbar_page_index(self.content_area, self.page_counter);
        self.scrollbar.mutate(ctx, |_ctx, scrollbar| {
            scrollbar.change_page(scrollbar_active_index);
        });
    }

    /// Placing current page, setting current buttons and clearing.
    fn update(&mut self, ctx: &mut EventCtx, get_new_page: bool) {
        if get_new_page {
            self.change_current_page(ctx);
        }
        self.current_page.place(self.content_area);
        self.set_buttons(ctx);
        self.scrollbar.request_complete_repaint(ctx);
        self.clear_and_repaint(ctx);
    }

    /// Clearing the whole area and requesting repaint.
    fn clear_and_repaint(&mut self, ctx: &mut EventCtx) {
        self.pad.clear();
        ctx.request_paint();
    }

    /// Going to the previous page.
    fn go_to_prev_page(&mut self, ctx: &mut EventCtx) {
        self.page_counter -= 1;
        self.update(ctx, true);
    }

    /// Going to the next page.
    fn go_to_next_page(&mut self, ctx: &mut EventCtx) {
        self.page_counter += 1;
        self.update(ctx, true);
    }

    /// Going to the first page.
    fn go_to_first_page(&mut self, ctx: &mut EventCtx) {
        self.page_counter = 0;
        self.update(ctx, true);
    }

    /// Going to the first page.
    fn go_to_last_page(&mut self, ctx: &mut EventCtx) {
        self.page_counter = self.pages.count() - 1;
        self.update(ctx, true);
    }

    /// Jumping to another page relative to the current one.
    fn go_to_page_relative(&mut self, jump: i16, ctx: &mut EventCtx) {
        self.page_counter += jump as usize;
        self.update(ctx, true);
    }

    /// Updating the visual state of the buttons after each event.
    /// All three buttons are handled based upon the current choice.
    /// If defined in the current choice, setting their text,
    /// whether they are long-pressed, and painting them.
    fn set_buttons(&mut self, ctx: &mut EventCtx) {
        let btn_layout = self.current_page.btn_layout();
        self.buttons.mutate(ctx, |_ctx, buttons| {
            buttons.set(btn_layout);
        });
    }

    /// Current choice is still the same, only its inner state has changed
    /// (its sub-page changed).
    fn update_after_current_choice_inner_change(&mut self, ctx: &mut EventCtx) {
        let inner_page = self.current_page.get_current_page();
        self.scrollbar.mutate(ctx, |ctx, scrollbar| {
            scrollbar.change_page(self.page_counter + inner_page);
            scrollbar.request_complete_repaint(ctx);
        });
        self.update(ctx, false);
    }

    /// When current choice contains paginated content, it may use the button
    /// event to just paginate itself.
    fn event_consumed_by_current_choice(&mut self, ctx: &mut EventCtx, pos: ButtonPos) -> bool {
        if matches!(pos, ButtonPos::Left) && self.current_page.has_prev_page() {
            self.current_page.go_to_prev_page();
            self.update_after_current_choice_inner_change(ctx);
            true
        } else if matches!(pos, ButtonPos::Right) && self.current_page.has_next_page() {
            self.current_page.go_to_next_page();
            self.update_after_current_choice_inner_change(ctx);
            true
        } else {
            false
        }
    }
}

impl<F, const M: usize, T> Component for Flow<F, M, T>
where
    F: Fn(usize) -> Page<M, T>,
    T: StringType,
{
    type Msg = FlowMsg;

    fn place(&mut self, bounds: Rect) -> Rect {
        let (title_content_area, button_area) = bounds.split_bottom(theme::BUTTON_HEIGHT);
        // Accounting for possible title
        let (title_area, content_area) = if self.title.is_some() {
            title_content_area.split_top(theme::FONT_HEADER.line_height())
        } else {
            (Rect::zero(), title_content_area)
        };
        self.content_area = content_area;

        // Finding out the total amount of pages in this flow
        let complete_page_count = self.pages.scrollbar_page_count(content_area);
        // Redefining scrollbar now when we have its page_count
        self.scrollbar = Child::new(ScrollBar::new(complete_page_count));

        // Placing a title and scrollbar in case the title is there
        // (scrollbar will be active - counting pages - even when not placed and
        // painted)
        if self.title.is_some() {
            let (title_area, scrollbar_area) =
                title_area.split_right(self.scrollbar.inner().overall_width() + SCROLLBAR_SPACE);

            self.title.place(title_area);
            self.title_area = title_area;
            self.scrollbar.place(scrollbar_area);
        }

        // We finally found how long is the first page, and can set its button layout.
        self.current_page.place(content_area);
        self.buttons = Child::new(ButtonController::new(self.current_page.btn_layout()));

        self.pad.place(title_content_area);
        self.buttons.place(button_area);
        bounds
    }

    fn event(&mut self, ctx: &mut EventCtx, event: Event) -> Option<Self::Msg> {
        ctx.set_page_count(self.pages.scrollbar_page_count(self.content_area));
        self.title.event(ctx, event);
        let button_event = self.buttons.event(ctx, event);

        // Do something when a button was triggered
        // and we have some action connected with it
        if let Some(ButtonControllerMsg::Triggered(pos)) = button_event {
            // When there is a previous or next screen in the current flow,
            // handle that first and in case it triggers, then do not continue
            if self.event_consumed_by_current_choice(ctx, pos) {
                return None;
            }

            let actions = self.current_page.btn_actions();
            let action = actions.get_action(pos);
            if let Some(action) = action {
                match action {
                    ButtonAction::PrevPage => {
                        self.go_to_prev_page(ctx);
                        return None;
                    }
                    ButtonAction::NextPage => {
                        self.go_to_next_page(ctx);
                        return None;
                    }
                    ButtonAction::FirstPage => {
                        self.go_to_first_page(ctx);
                        return None;
                    }
                    ButtonAction::LastPage => {
                        self.go_to_last_page(ctx);
                        return None;
                    }
                    ButtonAction::Cancel => return Some(FlowMsg::Cancelled),
                    ButtonAction::Confirm => {
                        if self.return_confirmed_index {
                            return Some(FlowMsg::ConfirmedIndex(self.page_counter));
                        } else {
                            return Some(FlowMsg::Confirmed);
                        }
                    }
                    ButtonAction::Info => return Some(FlowMsg::Info),
                    ButtonAction::Select => {}
                    ButtonAction::Action(_) => {}
                }
            }
        };
        None
    }

    fn paint(&mut self) {
        self.pad.paint();
        // Scrollbars are painted only with a title
        if self.title.is_some() {
            self.scrollbar.paint();
            self.title.paint();
        }
        self.buttons.paint();
        // On purpose painting current page at the end, after buttons,
        // because we sometimes (in the case of QR code) need to use the
        // whole height of the display for showing the content
        // (and painting buttons last would cover the lower part).
        self.current_page.paint();
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
use super::trace::ButtonTrace;

#[cfg(feature = "ui_debug")]
use heapless::String;

#[cfg(feature = "ui_debug")]
impl<F, const M: usize, T> ButtonTrace for Flow<F, M, T>
where
    F: Fn(usize) -> Page<M, T>,
    T: StringType,
{
    /// Accounting for the possibility that button is connected with the
    /// currently paginated flow_page (only Prev or Next in that case).
    fn get_btn_action(&self, pos: ButtonPos) -> String<25> {
        if matches!(pos, ButtonPos::Left) && self.current_page.has_prev_page() {
            ButtonAction::PrevPage.string()
        } else if matches!(pos, ButtonPos::Right) && self.current_page.has_next_page() {
            ButtonAction::NextPage.string()
        } else {
            let btn_actions = self.current_page.btn_actions();

            match btn_actions.get_action(pos) {
                Some(action) => action.string(),
                None => ButtonAction::empty(),
            }
        }
    }
}

#[cfg(feature = "ui_debug")]
impl<F, const M: usize, T> crate::trace::Trace for Flow<F, M, T>
where
    F: Fn(usize) -> Page<M, T>,
    T: StringType,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("Flow");
        t.int("flow_page", self.page_counter as i64);
        t.int("flow_page_count", self.pages.count() as i64);

        self.report_btn_actions(t);

        if let Some(title) = &self.title {
            t.child("title", title);
        }
        t.child("scrollbar", &self.scrollbar);
        t.child("buttons", &self.buttons);
        t.child("flow_page", &self.current_page);
    }
}
