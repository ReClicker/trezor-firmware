use crate::{strutil::ShortString, trace::Tracer};

pub trait ButtonTrace {
    fn get_left_action(&self) -> ShortString {
        "Prev".into()
    }

    fn get_middle_action(&self) -> ShortString {
        "Default".into()
    }

    fn get_right_action(&self) -> ShortString {
        "Next".into()
    }

    /// Report actions for all three buttons in easy-to-parse format.
    fn report_btn_actions(&self, t: &mut dyn Tracer) {
        t.string("left_action", &self.get_left_action());
        t.string("middle_action", &self.get_middle_action());
        t.string("right_action", &self.get_right_action());
    }
}
