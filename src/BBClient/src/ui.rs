use gtk::{
    ApplicationWindow, Builder, Button, ButtonExt, ContainerExt, GtkWindowExt, HeaderBar, Inhibit,
    WidgetExt, Window, WindowType, TextView, Application,
};

use std::{rc::Rc, cell::RefCell};

pub(crate) struct GuiWindow {
    window: ApplicationWindow,
    header_bar: HeaderBar,
    asm_textview: TextView,
}

impl GuiWindow {
    pub fn new(app: &Application) -> Option<Rc<RefCell<Self>>> {
        let builder = Builder::new_from_string(include_str!("bbclient.ui"));
        None
    }
}
