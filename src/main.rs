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

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}
fn main() {
    if TcpStream::connect(ADDRESS).is_ok() {
        // gbar is already running
        return;
    }

    gtk::init().unwrap();

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let (tx2, rx2) = mpsc::channel();
    start_server(tx, rx2);
    start_gui(rx, tx2);

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
            if args.is_empty() {
                continue;
            }
            let args = args.lines().map(ToOwned::to_owned).collect();

            tx.send(args).unwrap();
            let bin = rx2.recv().unwrap();

            send_stream.write_all(bin.as_bytes()).unwrap();
            send_stream.flush().unwrap();
        }
    });
}

fn start_gui(rx: glib::Receiver<Vec<String>>, tx2: mpsc::Sender<String>) {
    let args: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));

    let win = Window::new(WindowType::Toplevel);
    win.set_default_size(400, 300);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let input = Entry::new();
    let fuzzy = ListBox::new();

    input.connect_changed(clone!(fuzzy,args => move |ent| {
        let text = ent.get_text();
        let text = text.as_str();
        fuzzy.get_children().iter().for_each(|c| {
            fuzzy.remove(c);
        });
        args.borrow()
            .iter()
            .filter(|a| a.contains(text))
            .take(MAX_VISIBLE)
            .for_each(|a| {
                fuzzy.add(&Label::new(Some(&a)));
            });
        fuzzy.show_all();
        if fuzzy.get_children().is_empty() {
            return;
        }
        fuzzy.select_row(Some(
            &fuzzy.get_children()[0]
                .clone()
                .downcast::<ListBoxRow>()
                .unwrap(),
        ));
    }));

    let hide = clone!(win,input,fuzzy =>
    move || {
        win.hide();
        input.set_text("");
        fuzzy.get_children().iter().for_each(|c| {
            fuzzy.remove(c);
        });
    });

    win.connect_delete_event(clone!(tx2, hide => move |_, _| {
        tx2.send("".into()).unwrap();
        hide();
        gtk::Inhibit(true)
    }));

    win.connect_key_press_event(clone!(fuzzy => move |_win, key| match key.get_keyval() {
        gdk::keys::constants::Escape => {
            tx2.send("".into()).unwrap();
            hide();
            gtk::Inhibit(false)
        }
        gdk::keys::constants::Return => {
            if !fuzzy.get_children().is_empty() {
                let bin: Label = fuzzy.get_selected_row().unwrap().get_children()[0]
                    .clone()
                    .downcast()
                    .unwrap();
                tx2.send(bin.get_text().to_string()).unwrap();
            } else {
                tx2.send("".into()).unwrap();
            }
            hide();
            gtk::Inhibit(false)
        }
        _ => gtk::Inhibit(false),
    }));

    vbox.add(&input);
    vbox.add(&fuzzy);
    win.add(&vbox);

    rx.attach(None, move |new_args| {
        let mut args = args.borrow_mut();
        *args = new_args;
        args.iter().take(MAX_VISIBLE).for_each(|a| {
            fuzzy.add(&Label::new(Some(&a)));
        });
        fuzzy.select_row(Some(
            &fuzzy.get_children()[0]
                .clone()
                .downcast::<ListBoxRow>()
                .unwrap(),
        ));
        win.show_all();
        glib::Continue(true)
    });
}
