use crate::{
    strutil::StringType,
    ui::{
        component::{
            text::{
                layout::{LayoutFit, LayoutSink, TextNoOp, TextRenderer},
                TextStyle,
            },
            LineBreaking, Paginate, TextLayout,
        },
        display::{Font, Icon},
        geometry::{Alignment, Offset, Rect},
        util::ResultExt,
    },
};

use core::marker::PhantomData;
use heapless::Vec;

use super::{
    flow_pages_helpers::{Op, ToDisplay},
    theme, ButtonActions, ButtonDetails, ButtonLayout,
};

/// Holding specific workflows that are created in `layout.rs`.
/// Is returning a `Page` (page/screen) on demand
/// based on the current page in `Flow`.
/// Before, when `layout.rs` was defining a `heapless::Vec` of `Page`s,
/// it was a very stack-expensive operation and StackOverflow was encountered.
/// With this "lazy-loading" approach (creating each page on demand) we can
/// have theoretically unlimited number of pages without triggering SO.
/// (Currently only the current page is stored on stack - in
/// `Flow::current_page`.)
pub struct FlowPages<F, const M: usize, T>
where
    T: StringType,
{
    /// Function/closure that will return appropriate page on demand.
    get_page: F,
    /// Number of pages in the flow.
    page_count: usize,
    _phantom: PhantomData<T>,
}

impl<F, const M: usize, T> FlowPages<F, M, T>
where
    F: Fn(usize) -> Page<M, T>,
    T: StringType,
{
    pub fn new(get_page: F, page_count: usize) -> Self {
        Self {
            get_page,
            page_count,
            _phantom: PhantomData,
        }
    }

    /// Returns a page on demand on a specified index.
    pub fn get(&self, page_index: usize) -> Page<M, T> {
        (self.get_page)(page_index)
    }

    /// Total amount of pages.
    pub fn count(&self) -> usize {
        self.page_count
    }

    /// How many scrollable pages are there in the flow
    /// (each page can have arbitrary number of "sub-pages").
    pub fn scrollbar_page_count(&self, bounds: Rect) -> usize {
        self.scrollbar_page_index(bounds, self.page_count)
    }

    /// Active scrollbar position connected with the beginning of a specific
    /// page index.
    pub fn scrollbar_page_index(&self, bounds: Rect, page_index: usize) -> usize {
        let mut page_count = 0;
        for i in 0..page_index {
            let mut current_page = self.get(i);
            current_page.place(bounds);
            page_count += current_page.page_count;
        }
        page_count
    }
}

#[derive(Clone)]
pub struct Page<const M: usize, T>
where
    T: StringType,
{
    ops: Vec<Op<T>, M>,
    text_layout: TextLayout,
    btn_layout: ButtonLayout<T>,
    btn_actions: ButtonActions,
    current_page: usize,
    page_count: usize,
    char_offset: usize,
    title: Option<T>,
}

// For `layout.rs`
impl<const M: usize, T> Page<M, T>
where
    T: StringType,
{
    pub fn new(
        btn_layout: ButtonLayout<T>,
        btn_actions: ButtonActions,
        initial_text_font: Font,
    ) -> Self {
        let style = TextStyle::new(
            initial_text_font,
            theme::FG,
            theme::BG,
            theme::FG,
            theme::FG,
        )
        .with_ellipsis_icon(
            Icon::new(theme::ICON_NEXT_PAGE),
            theme::ELLIPSIS_ICON_MARGIN,
        )
        .with_prev_page_icon(
            Icon::new(theme::ICON_PREV_PAGE),
            theme::PREV_PAGE_ICON_MARGIN,
        );
        Self {
            ops: Vec::new(),
            text_layout: TextLayout::new(style),
            btn_layout,
            btn_actions,
            current_page: 0,
            page_count: 1,
            char_offset: 0,
            title: None,
        }
    }

    pub const fn with_line_breaking(mut self, line_breaking: LineBreaking) -> Self {
        self.text_layout.style.line_breaking = line_breaking;
        self
    }
}

// For `flow.rs`
impl<const M: usize, T> Page<M, T>
where
    T: StringType,
{
    /// Adding title.
    pub fn with_title(mut self, title: T) -> Self {
        self.title = Some(title);
        self
    }

    pub fn paint(&mut self) {
        self.change_page(self.current_page);
        self.layout_content(&mut TextRenderer);
    }

    pub fn place(&mut self, bounds: Rect) -> Rect {
        self.text_layout.bounds = bounds;
        self.page_count = self.page_count();
        bounds
    }

    pub fn btn_layout(&self) -> ButtonLayout<T> {
        // When we are in pagination inside this flow,
        // show the up and down arrows on appropriate sides.
        let current = self.btn_layout.clone();

        // On the last page showing only the narrow arrow, so the right
        // button with possibly long text has enough space.
        let btn_left = if self.has_prev_page() && !self.has_next_page() {
            Some(ButtonDetails::up_arrow_icon())
        } else if self.has_prev_page() {
            Some(ButtonDetails::up_arrow_icon_wide())
        } else {
            current.btn_left
        };

        // Middle button should be shown only on the last page, not to collide
        // with the fat right button.
        let (btn_middle, btn_right) = if self.has_next_page() {
            (None, Some(ButtonDetails::down_arrow_icon_wide()))
        } else {
            (current.btn_middle, current.btn_right)
        };

        ButtonLayout::new(btn_left, btn_middle, btn_right)
    }

    pub fn btn_actions(&self) -> ButtonActions {
        self.btn_actions
    }

    pub fn title(&self) -> Option<T> {
        self.title.clone()
    }

    pub fn has_prev_page(&self) -> bool {
        self.current_page > 0
    }

    pub fn has_next_page(&self) -> bool {
        self.current_page < self.page_count - 1
    }

    pub fn go_to_prev_page(&mut self) {
        self.current_page -= 1;
    }

    pub fn go_to_next_page(&mut self) {
        self.current_page += 1;
    }

    pub fn get_current_page(&self) -> usize {
        self.current_page
    }
}

// For `layout.rs` - single operations
impl<const M: usize, T> Page<M, T>
where
    T: StringType,
{
    pub fn with_new_item(mut self, item: Op<T>) -> Self {
        self.ops
            .push(item)
            .assert_if_debugging_ui("Could not push to self.ops");
        self
    }

    pub fn text(self, text: T) -> Self {
        self.with_new_item(Op::Text(ToDisplay::new(text)))
    }

    pub fn newline(self) -> Self {
        self.with_new_item(Op::Text(ToDisplay::new("\n".into())))
    }

    pub fn newline_half(self) -> Self {
        self.with_new_item(Op::Text(ToDisplay::new("\r".into())))
    }

    pub fn next_page(self) -> Self {
        self.with_new_item(Op::NextPage)
    }

    pub fn font(self, font: Font) -> Self {
        self.with_new_item(Op::Font(font))
    }

    pub fn offset(self, offset: Offset) -> Self {
        self.with_new_item(Op::CursorOffset(offset))
    }

    pub fn alignment(self, alignment: Alignment) -> Self {
        self.with_new_item(Op::Alignment(alignment))
    }
}

// For `layout.rs` - aggregating operations
impl<const M: usize, T> Page<M, T>
where
    T: StringType,
{
    pub fn text_normal(self, text: T) -> Self {
        self.font(Font::NORMAL).text(text)
    }

    pub fn text_mono(self, text: T) -> Self {
        self.font(Font::MONO).text(text)
    }

    pub fn text_bold(self, text: T) -> Self {
        self.font(Font::BOLD).text(text)
    }
}

// For painting and pagination
impl<const M: usize, T> Page<M, T>
where
    T: StringType,
{
    pub fn set_char_offset(&mut self, char_offset: usize) {
        self.char_offset = char_offset;
    }

    pub fn layout_content(&self, sink: &mut dyn LayoutSink) -> LayoutFit {
        let mut cursor = self.text_layout.initial_cursor();
        self.text_layout
            .layout_ops_new(self.ops.clone(), &mut cursor, self.char_offset, sink)
    }
}

// Pagination
impl<const M: usize, T> Paginate for Page<M, T>
where
    T: StringType,
{
    fn page_count(&mut self) -> usize {
        let mut page_count = 1; // There's always at least one page.
        let mut char_offset = 0;

        // Make sure we're starting from the beginning.
        self.set_char_offset(char_offset);

        // Looping through the content and counting pages
        // until we finally fit.
        loop {
            let fit = self.layout_content(&mut TextNoOp);
            match fit {
                LayoutFit::Fitting { .. } => {
                    break; // TODO: We should consider if there's more content
                           // to render.
                }
                LayoutFit::OutOfBounds {
                    processed_chars, ..
                } => {
                    page_count += 1;
                    char_offset += processed_chars;
                    self.set_char_offset(char_offset);
                }
            }
        }

        // Reset the char offset back to the beginning.
        self.set_char_offset(0);

        page_count
    }

    fn change_page(&mut self, to_page: usize) {
        let mut active_page = 0;
        let mut char_offset = 0;

        // Make sure we're starting from the beginning.
        self.set_char_offset(char_offset);

        // Looping through the content until we arrive at
        // the wanted page.
        while active_page < to_page {
            let fit = self.layout_content(&mut TextNoOp);
            match fit {
                LayoutFit::Fitting { .. } => {
                    break; // TODO: We should consider if there's more content
                           // to render.
                }
                LayoutFit::OutOfBounds {
                    processed_chars, ..
                } => {
                    active_page += 1;
                    char_offset += processed_chars;
                    self.set_char_offset(char_offset);
                }
            }
        }
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
impl<const M: usize, T> crate::trace::Trace for Page<M, T>
where
    T: StringType,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        use crate::ui::component::text::layout::trace::TraceSink;
        use core::cell::Cell;
        let fit: Cell<Option<LayoutFit>> = Cell::new(None);
        t.component("Page");
        if let Some(title) = &self.title {
            // Not calling it "title" as that is already traced by FlowPage
            t.string("page_title", title.as_ref());
        }
        t.int("active_page", self.current_page as i64);
        t.int("page_count", self.page_count as i64);
        t.in_list("text", &|l| {
            let result = self.layout_content(&mut TraceSink(l));
            fit.set(Some(result));
        });
    }
}
