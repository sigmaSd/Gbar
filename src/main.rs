use gtk::prelude::Cast;
use gtk::{
    ContainerExt, EditableSignals, Entry, EntryExt, GtkWindowExt, Label, LabelExt, ListBox,
    ListBoxExt, ListBoxRow, WidgetExt, Window, WindowType,
};
use std::io::BufRead;
use std::rc::Rc;
const MAX_VISIBLE: usize = 10;
fn main() {
    gtk::init().unwrap();
    let args: Vec<String> = std::io::stdin()
        .lock()
        .lines()
        .filter_map(Result::ok)
        .collect();
    let _bar = Bar::new(args);

    gtk::main();
}

struct Bar {}

impl Bar {
    fn new(args: Vec<String>) -> Self {
        let win = Window::new(WindowType::Toplevel);
        win.set_default_size(400, 300);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let fuzzy = ListBox::new();
        let fuzzy_c = fuzzy.clone();
        let fuzzy_c2 = fuzzy.clone();

        let args = Rc::new(args);
        let args_c = args.clone();
        args.iter().take(MAX_VISIBLE).for_each(|a| {
            fuzzy.add(&Label::new(Some(&a)));
        });

        let input = Entry::new();
        input.connect_changed(move |ent| {
            let text = ent.get_text().to_string();
            fuzzy_c.get_children().iter().for_each(|c| {
                fuzzy_c.remove(c);
            });
            args_c
                .iter()
                .filter(|a| a.contains(&text))
                .take(MAX_VISIBLE)
                .for_each(|a| {
                    fuzzy_c.add(&Label::new(Some(&a)));
                });
            fuzzy_c.show_all();
            if fuzzy_c.get_children().is_empty() {
                return;
            }
            fuzzy_c.select_row(Some(
                &fuzzy_c.get_children()[0]
                    .clone()
                    .downcast::<ListBoxRow>()
                    .unwrap(),
            ));
        });
        win.connect_key_press_event(move |_, key| match key.get_keyval() {
            gdk::keys::constants::Escape => {
                gtk::main_quit();
                gtk::Inhibit(false)
            }
            gdk::keys::constants::Return => {
                let bin: Label = fuzzy_c2.get_selected_row().unwrap().get_children()[0]
                    .clone()
                    .downcast()
                    .unwrap();
                //let _ = std::process::Command::new(bin.get_text().to_string()).spawn();
                println!("{}", bin.get_text());

                gtk::main_quit();
                gtk::Inhibit(false)
            }
            _ => gtk::Inhibit(false),
        });

        vbox.add(&input);
        vbox.add(&fuzzy);
        win.add(&vbox);
        win.show_all();
        fuzzy.select_row(Some(
            &fuzzy.get_children()[0]
                .clone()
                .downcast::<ListBoxRow>()
                .unwrap(),
        ));
        Self {}
    }
}
