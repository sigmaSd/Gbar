use std::cell::RefCell;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::rc::Rc;
use std::sync::mpsc;

const MAX_VISIBLE: usize = 10;
const ADDRESS: &str = "127.0.0.1:38451";

use gio::{ApplicationExt, ListModelExt};
use glib::signal::Inhibit;
use glib::Cast;
use gtk::{
    prelude::ApplicationExtManual, Application, BoxExt, EditableExt, Entry, EntryExt,
    EventControllerKey, GtkApplicationExt, GtkWindowExt, Label, ListBox, ListBoxRow, ListBoxRowExt,
    WidgetExt, Window,
};

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
    let ev = EventControllerKey::new();

    let win = Window::new();
    win.set_decorated(false);
    win.set_default_size(400, 300);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let input = Entry::new();
    let fuzzy = ListBox::new();

    input.connect_changed(clone!(fuzzy,args => move |ent| {
        let text = ent.get_text().unwrap();
        let text = text.as_str();

        remove_all_children(&fuzzy);

        let mut fst = None;
        args.borrow().iter().filter(|a| a.contains(text)).take(MAX_VISIBLE).enumerate().for_each(|(idx,a)| {
            if idx == 0 {
                let lbox = ListBoxRow::new();
                lbox.set_child(Some(&Label::new(Some(&a))));
            fst = Some(lbox);
            fuzzy.append(fst.as_ref().unwrap());
            } else {
            fuzzy.append(&Label::new(Some(&a)));
            }
        });
        fuzzy.select_row(fst.as_ref());
    }));

    let hide = clone!(win,input,fuzzy =>
    move || {
        win.hide();
        input.set_text("");
        remove_all_children(&fuzzy);
    }
    );

    input.connect_activate(clone!(fuzzy, tx2, hide => move|_|{
            if let Some(row) = fuzzy.get_selected_row() {
                let bin: Label = row.get_child().unwrap()
                    .downcast()
                    .unwrap();
                tx2.send(bin.get_text().to_string()).unwrap();
            } else {
                tx2.send("".into()).unwrap();
            }
            hide();
    }));

    ev.connect_key_pressed(clone!(fuzzy => move |_ev, key,_keycode, _m| match key {
        gdk::keys::constants::Escape => {
            tx2.send("".into()).unwrap();
            hide();
            Inhibit(false)
        }
        gdk::keys::constants::Return => {
            if let Some(row) = fuzzy.get_selected_row() {
                let bin: Label = row.get_child().unwrap()
                    .downcast()
                    .unwrap();
                tx2.send(bin.get_text().to_string()).unwrap();
            } else {
                tx2.send("".into()).unwrap();
            }
            hide();
            Inhibit(false)
        }
        _ => Inhibit(false),
    }));

    vbox.append(&input);
    vbox.append(&fuzzy);
    win.set_child(Some(&vbox));
    win.add_controller(&ev);

    rx.attach(
        None,
        clone!(win => move |new_args| {
            let mut args = args.borrow_mut();
            *args = new_args;

            let mut fst = None;
            args.iter().take(MAX_VISIBLE).enumerate().for_each(|(idx,a)| {
                if idx == 0 {
                    let lbox = ListBoxRow::new();
                    lbox.set_child(Some(&Label::new(Some(&a))));
                fst = Some(lbox);
                fuzzy.append(fst.as_ref().unwrap());
                } else {
                fuzzy.append(&Label::new(Some(&a)));
                }
            });
            fuzzy.select_row(fst.as_ref());

            win.present();
            glib::Continue(true)
        }),
    );

    let app = Application::new(None, gio::ApplicationFlags::empty()).unwrap();
    app.connect_startup(move |app| {
        app.add_window(&win);
    });
    app.connect_activate(|_| {});
    app.run(&[]);
}

// helper
fn remove_all_children(fuzzy: &ListBox) {
    let listmodel = fuzzy.observe_children();
    while let Some(o) = listmodel.get_object(0) {
        let widget = o.clone().downcast::<gtk::Widget>().unwrap();
        fuzzy.remove(&widget);
    }
}
