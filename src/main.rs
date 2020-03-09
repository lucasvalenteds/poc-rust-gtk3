#![windows_subsystem = "windows"]
extern crate gio;
extern crate gtk;
extern crate gdk;

use gio::prelude::*;
use gtk::prelude::*;

use gtk::*;
use gdk::*;
use sqlx::*;
use std::rc::*;
use std::cell::*;
use async_std::{task,io};

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

async fn soft_delete_account(
    mut pool: &MySqlPool,
    account_code: &String
) -> Result<u64, sqlx::error::Error> {
    sqlx::query("update ACCOUNT set REMOVED = 1 where CODE = ?")
        .bind(account_code)
        .execute(&mut pool)
        .await
}

fn create_account_revoked_dialog(account_code: &String) -> MessageDialog {
    let dialog_builder: MessageDialogBuilder = MessageDialogBuilder::new();

    let dialog: MessageDialog = dialog_builder
        .gravity(Gravity::Center)
        .title("Account revoked successfully")
        .text(&format!("Account with code {} does not have access to the system anymore.", account_code))
        .buttons(ButtonsType::Ok)
        .build();

    dialog
}

fn create_account_revoked_failure_dialog(account_code: &String) -> MessageDialog {
    let dialog_builder: MessageDialogBuilder = MessageDialogBuilder::new();

    let dialog: MessageDialog = dialog_builder
        .gravity(Gravity::Center)
        .title("Account revoked failure")
        .text(&format!("Could not revoke access from account with code {} ", account_code))
        .buttons(ButtonsType::Ok)
        .build();

    dialog
}

fn create_revoke_account_window(
    pool_ref: &MySqlPool
) -> gtk::Box {
    const ACCOUNT_CODE_LENGTH: i32 = 12;
    let box_builder: BoxBuilder = BoxBuilder::new();
    let label_builder: LabelBuilder = LabelBuilder::new();
    let entry_box_builder: BoxBuilder = BoxBuilder::new();
    let entry_label_builder: LabelBuilder = LabelBuilder::new();
    let entry_builder: EntryBuilder = EntryBuilder::new();
    let button_builder: ButtonBuilder = ButtonBuilder::new();
    let text_field_buffer = EntryBuffer::new(None);

    let vbox: gtk::Box = box_builder
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin(16)
        .build();

    let label: Label = label_builder
        .label("Use the form below to revoke account access.")
        .lines(1)
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let entry_box = entry_box_builder
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();

    let entry_label: Label = entry_label_builder
        .label("Account code")
        .lines(1)
        .justify(Justification::Left)
        .build();

    let entry: Entry = entry_builder
        .buffer(&text_field_buffer)
        .valign(Align::Fill)
        .hexpand(true)
        .max_length(ACCOUNT_CODE_LENGTH)
        .build();

    let button: Button = button_builder
        .label("Revoke")
        .sensitive(false)
        .build();

    let button_clone: Button = button.clone();
    entry.connect_changed(move |entry_ref| {
        if let Some(text) = entry_ref.get_text() {
            if text.as_str().len() == ACCOUNT_CODE_LENGTH as usize {
                &button_clone.set_sensitive(true);
            } else {
                &button_clone.set_sensitive(false);
            }
        }
    });

    let pool_clone: MySqlPool = pool_ref.clone();
    let pool: Rc<RefCell<MySqlPool>> = Rc::new(RefCell::new(pool_clone));
    button.connect_clicked(clone!(pool => move |_| {
        task::block_on(async {
            let account_code = &text_field_buffer.get_text();
            let result = soft_delete_account(&pool.borrow_mut(), &account_code)
                .await;

            let dialog: MessageDialog = match result {
                Ok(_) => create_account_revoked_dialog(&account_code),
                Err(_) => create_account_revoked_failure_dialog(&account_code)
            };
            dialog.connect_response(|d, _| {
                d.destroy();
            });
            dialog.show_all();

            ()
        })
    }));

    entry_box.add(&entry_label);
    entry_box.add(&entry);

    vbox.add(&label);
    vbox.add(&entry_box);
    vbox.add(&button);

    vbox
}

fn create_application_window(
    application: &Application
) -> ApplicationWindow {
    let window_builder: ApplicationWindowBuilder = ApplicationWindowBuilder::new();

    let window: ApplicationWindow = window_builder.application(application)
        .title("Support app")
        .default_width(480)
        .default_height(320)
        .resizable(false)
        .window_position(WindowPosition::CenterAlways)
        .build();

    window
}

fn main() -> io::Result<()> {
    task::block_on(async {
        let pool = MySqlPool::new(env!("DATABASE_URL")).await
            .unwrap();

        let application_id: Option<&str> = Some(env!("APPLICATION_ID"));
        let application_flags: gio::ApplicationFlags = Default::default();
        let application: Application = Application::new(application_id, application_flags)
            .unwrap();

        application.connect_activate(move |application: &gtk::Application| {
            let window = create_application_window(application);
            window.add(&create_revoke_account_window(&pool));
            window.show_all();
        });

        application.run(&[]);

        Ok(())
    })
}
