use crate::{
    strutil::StringType,
    ui::{
        display::{rect_fill, rect_fill_corners, rect_outline_rounded, Font, Icon},
        geometry::{Offset, Rect, BOTTOM_LEFT, BOTTOM_RIGHT},
    },
};
use heapless::String;

use super::super::{
    common::{display, display_inverse, display_right},
    theme, ButtonDetails, ButtonLayout, Choice,
};

const ICON_RIGHT_PADDING: i16 = 2;
const STRING_LEN: usize = 50;

/// Simple string component used as a choice item.
#[derive(Clone)]
pub struct ChoiceItem<T>
where
    T: StringType,
{
    text: String<STRING_LEN>,
    icon: Option<Icon>,
    btn_layout: ButtonLayout<T>,
    font: Font,
}

impl<T> ChoiceItem<T>
where
    T: StringType,
{
    pub fn new<F>(text: F, btn_layout: ButtonLayout<T>) -> Self
    where
        F: AsRef<str>,
    {
        Self {
            text: String::from(text.as_ref()),
            icon: None,
            btn_layout,
            font: theme::FONT_CHOICE_ITEMS,
        }
    }

    /// Allows to add the icon.
    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Allows to change the font.
    pub fn with_font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Getting the offset of the icon to center it vertically.
    /// Depending on its size and used font.
    fn icon_vertical_offset(&self) -> Offset {
        if let Some(icon) = self.icon {
            let height_diff = self.font.text_height() - icon.toif.height();
            Offset::y(-height_diff / 2)
        } else {
            Offset::zero()
        }
    }

    /// Getting the visible text width in pixels.
    fn visible_text_width(&self) -> i16 {
        self.font.visible_text_width(&self.text)
    }

    /// Getting the initial x-bearing of the text in pixels,
    /// so that we can adjust its positioning to center it properly.
    fn text_x_bearing(&self) -> i16 {
        self.font.start_x_bearing(&self.text)
    }

    /// Getting the non-central width in pixels.
    /// It will show an icon if defined, otherwise the text, not both.
    fn width_side(&self) -> i16 {
        if let Some(icon) = self.icon {
            icon.toif.width()
        } else {
            self.visible_text_width()
        }
    }

    /// Whether the whole item fits into the given rectangle.
    fn fits(&self, rect: Rect) -> bool {
        self.width_side() <= rect.width()
    }

    /// Draws highlight around this choice item.
    /// Must be called before the item is drawn, otherwise it will
    /// cover the item.
    fn paint_rounded_highlight(&self, area: Rect, inverse: bool) {
        let bound = theme::BUTTON_OUTLINE;
        let left_bottom =
            area.bottom_center() + Offset::new(-self.width_center() / 2 - bound, bound + 1);
        let x_size = self.width_center() + 2 * bound;
        let y_size = self.font.text_height() + 2 * bound;
        let outline_size = Offset::new(x_size, y_size);
        let outline = Rect::from_bottom_left_and_size(left_bottom, outline_size);
        if inverse {
            rect_fill(outline, theme::FG);
            rect_fill_corners(outline, theme::BG);
        } else {
            rect_outline_rounded(outline, theme::FG, theme::BG, 1);
        }
    }

    /// Painting the item as a choice on the left side from center.
    /// Showing only the icon, if available, otherwise the text.
    fn render_left(&self, area: Rect) {
        if let Some(icon) = self.icon {
            icon.draw(
                area.bottom_right() + self.icon_vertical_offset(),
                BOTTOM_RIGHT,
                theme::FG,
                theme::BG,
            );
        } else {
            display_right(area.bottom_right(), &self.text, self.font);
        }
    }

    /// Painting the item as a choice on the right side from center.
    /// Showing only the icon, if available, otherwise the text.
    fn render_right(&self, area: Rect) {
        if let Some(icon) = self.icon {
            icon.draw(
                area.bottom_left() + self.icon_vertical_offset(),
                BOTTOM_LEFT,
                theme::FG,
                theme::BG,
            );
        } else {
            display(area.bottom_left(), &self.text, self.font);
        }
    }

    /// Setting left button.
    pub fn set_left_btn(&mut self, btn_left: Option<ButtonDetails<T>>) {
        self.btn_layout.btn_left = btn_left;
    }

    /// Setting middle button.
    pub fn set_middle_btn(&mut self, btn_middle: Option<ButtonDetails<T>>) {
        self.btn_layout.btn_middle = btn_middle;
    }

    /// Setting right button.
    pub fn set_right_btn(&mut self, btn_right: Option<ButtonDetails<T>>) {
        self.btn_layout.btn_right = btn_right;
    }

    /// Changing the text.
    pub fn set_text(&mut self, text: String<STRING_LEN>) {
        self.text = text;
    }
}

impl<T> Choice<T> for ChoiceItem<T>
where
    T: StringType,
{
    /// Painting the item as the main choice in the middle.
    /// Showing both the icon and text, if the icon is available.
    fn paint_center(&self, area: Rect, inverse: bool) {
        self.paint_rounded_highlight(area, inverse);

        let mut baseline = area.bottom_center() + Offset::x(-self.width_center() / 2);
        if let Some(icon) = self.icon {
            let fg_color = if inverse { theme::BG } else { theme::FG };
            let bg_color = if inverse { theme::FG } else { theme::BG };
            icon.draw(
                baseline + self.icon_vertical_offset(),
                BOTTOM_LEFT,
                fg_color,
                bg_color,
            );
            baseline = baseline + Offset::x(icon.toif.width() + ICON_RIGHT_PADDING);
        }
        // Possibly shifting the baseline left, when there is a text bearing.
        // This is to center the text properly.
        baseline = baseline - Offset::x(self.text_x_bearing());
        if inverse {
            display_inverse(baseline, &self.text, self.font);
        } else {
            display(baseline, &self.text, self.font);
        }
    }

    /// Getting the overall width in pixels when displayed in center.
    /// That means both the icon and text will be shown.
    fn width_center(&self) -> i16 {
        let icon_width = if let Some(icon) = self.icon {
            icon.toif.width() + ICON_RIGHT_PADDING
        } else {
            0
        };
        icon_width + self.visible_text_width()
    }

    /// Painting item on the side if it fits, otherwise paint incomplete if
    /// allowed
    fn paint_left(&self, area: Rect, show_incomplete: bool) -> Option<i16> {
        // When the item does not fit, we stop.
        // Rendering the item anyway if the incomplete items are allowed.
        if !self.fits(area) {
            if show_incomplete {
                self.render_left(area);
            }
            return None;
        }

        // Rendering the item.
        self.render_left(area);

        Some(self.width_side())
    }

    /// Painting item on the side if it fits, otherwise paint incomplete if
    /// allowed
    fn paint_right(&self, area: Rect, show_incomplete: bool) -> Option<i16> {
        // When the item does not fit, we stop.
        // Rendering the item anyway if the incomplete items are allowed.
        if !self.fits(area) {
            if show_incomplete {
                self.render_right(area);
            }
            return None;
        }

        // Rendering the item.
        self.render_right(area);

        Some(self.width_side())
    }

    /// Getting current button layout.
    fn btn_layout(&self) -> ButtonLayout<T> {
        self.btn_layout.clone()
    }
}

// DEBUG-ONLY SECTION BELOW

#[cfg(feature = "ui_debug")]
impl<T> crate::trace::Trace for ChoiceItem<T>
where
    T: StringType,
{
    fn trace(&self, t: &mut dyn crate::trace::Tracer) {
        t.component("ChoiceItem");
        t.string("content", &self.text);
    }
}
