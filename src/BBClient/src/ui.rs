use gtk::{
    Application, ApplicationWindow, Builder, Button, ButtonExt, ContainerExt, GtkWindowExt,
    HeaderBar, Inhibit, TextView, WidgetExt, Window, WindowType,
};

use std::{cell::RefCell, rc::Rc};

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
