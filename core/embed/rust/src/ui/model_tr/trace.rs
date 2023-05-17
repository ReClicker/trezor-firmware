use crate::trace::Tracer;

use super::component::ButtonPos;

use heapless::String;

pub trait ButtonTrace {
    /// Describes what happens when a certain button is triggered.
    fn get_btn_action(&self, _pos: ButtonPos) -> String<25> {
        "Default".into()
    }

    /// Report actions for all three buttons in easy-to-parse format.
    fn report_btn_actions(&self, t: &mut dyn Tracer) {
        t.string("left_action", &self.get_btn_action(ButtonPos::Left));
        t.string("middle_action", &self.get_btn_action(ButtonPos::Middle));
        t.string("right_action", &self.get_btn_action(ButtonPos::Right));
    }
}
