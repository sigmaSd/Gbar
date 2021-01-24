use gtk::prelude::Cast;
use gtk::{
    ContainerExt, EditableSignals, Entry, EntryExt, GtkWindowExt, Label, LabelExt, ListBox,
    ListBoxExt, ListBoxRow, WidgetExt, Window, WindowType,
};
use std::cell::RefCell;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::rc::Rc;
use std::sync::mpsc;
const MAX_VISIBLE: usize = 10;
const ADDRESS: &str = "127.0.0.1:38451";
fn main() {
    if TcpStream::connect(ADDRESS).is_ok() {
        // gbar is already running
        return;
    }

    gtk::init().unwrap();

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let (tx2, rx2) = mpsc::channel();
    start_server(tx, rx2);
    let _bar = Bar::new(rx, tx2);

    gtk::main();
}
fn start_server(tx: glib::Sender<Vec<String>>, rx2: mpsc::Receiver<String>) {
    std::thread::spawn(move || {
        use std::net::TcpListener;
        let listener = TcpListener::bind(ADDRESS).unwrap();
        let mut listener = listener.incoming();

        loop {
            let mut recv_stream = listener.next().unwrap().unwrap();
            let mut send_stream = listener.next().unwrap().unwrap();

            let mut args = String::new();
            recv_stream.read_to_string(&mut args).unwrap();
            let args = args.lines().map(ToOwned::to_owned).collect();

            tx.send(args).unwrap();
            let bin = rx2.recv().unwrap();

            send_stream.write_all(bin.as_bytes()).unwrap();
            send_stream.flush().unwrap();
        }
    });
}
struct Bar {}

impl Bar {
    fn new(rx: glib::Receiver<Vec<String>>, tx2: mpsc::Sender<String>) -> Self {
        let win = Window::new(WindowType::Toplevel);
        win.set_default_size(400, 300);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let fuzzy = ListBox::new();
        let fuzzy_c = fuzzy.clone();
        let fuzzy_c2 = fuzzy.clone();

        let args: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));
        let args_c = args.clone();

        let input = Entry::new();
        input.connect_changed(move |ent| {
            let text = ent.get_text().to_string();
            fuzzy_c.get_children().iter().for_each(|c| {
                fuzzy_c.remove(c);
            });
            args_c
                .borrow()
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

        let win_c = win.clone();
        let input_c = input.clone();
        let fuzzy_c = fuzzy.clone();
        let hide = move || {
            win_c.hide();
            input_c.set_text("");
            fuzzy_c.get_children().iter().for_each(|c| {
                fuzzy_c.remove(c);
            });
        };

        win.connect_key_press_event(move |_win, key| match key.get_keyval() {
            gdk::keys::constants::Escape => {
                tx2.send("".into()).unwrap();
                //gtk::main_quit();
                hide();
                gtk::Inhibit(false)
            }
            gdk::keys::constants::Return => {
                if !fuzzy_c2.get_children().is_empty() {
                    let bin: Label = fuzzy_c2.get_selected_row().unwrap().get_children()[0]
                        .clone()
                        .downcast()
                        .unwrap();
                    //let _ = std::process::Command::new(bin.get_text().to_string()).spawn();
                    //println!("{}", bin.get_text());
                    tx2.send(bin.get_text().to_string()).unwrap();
                }

                //gtk::main_quit();
                hide();
                gtk::Inhibit(false)
            }
            _ => gtk::Inhibit(false),
        });

        vbox.add(&input);
        vbox.add(&fuzzy);
        win.add(&vbox);
        rx.attach(None, move |new_args| {
            let mut args = args.borrow_mut();
            args.clear();
            *args = new_args;
            win.show_all();
            glib::Continue(true)
        });
        Self {}
    }
}
