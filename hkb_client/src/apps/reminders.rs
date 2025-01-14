use hkb_core::database::services;
use hkb_core::database::services::reminders::CreateReminderData;
use hkb_core::logger::{debug, error, info};
use hkb_daemon_core::frame::Event as FrameEvent;
use ratatui::prelude::{Frame, Rect};

use self::reminders_create::RemindersCreate;
use self::reminders_list::RemindersList;

mod reminders_create;
mod reminders_list;

trait RemindersView {
    fn init(&mut self);
    fn update(&mut self) -> Option<Message>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

enum View {
    List,
    Create,
}

impl From<View> for Box<dyn RemindersView> {
    fn from(val: View) -> Self {
        match val {
            View::List => Box::new(RemindersList::default()),
            View::Create => Box::new(RemindersCreate::default()),
        }
    }
}

enum Message {
    ChangeView(View),
    DeleteReminder(i64),
    CreateReminder(CreateReminderData),
}

pub struct RemindersApp {
    current_view: Box<dyn RemindersView>,
}

impl RemindersApp {
    pub fn new() -> Self {
        let mut current_view: Box<dyn RemindersView> = View::List.into();
        current_view.init();

        Self { current_view }
    }
}

impl RemindersApp {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(m) = self.current_view.update() {
            match m {
                Message::ChangeView(view) => {
                    self.current_view = view.into();
                    self.current_view.init();
                }
                Message::CreateReminder(reminder) => {
                    info!(target: "CLIENT_REMINDERS", "Creating a reminder.");
                    debug!(target: "CLIENT_REMINDERS", "Received a message to create a reminder with {reminder:?}");

                    if let Ok(reminder) = services::reminders::create_reminder(reminder) {
                        crate::singleton::send_server_msg(FrameEvent::ReminderCreated(reminder));
                    }

                    self.current_view = View::List.into();
                    self.current_view.init();
                }
                Message::DeleteReminder(reminder_id) => {
                    info!(target: "CLIENT_REMINDERS", "Deleting a reminder.");
                    debug!(target: "CLIENT_REMINDERS", "Received a message to delete a reminder with id {reminder_id}");

                    if services::reminders::delete_reminder(reminder_id).is_ok() {
                        crate::singleton::send_server_msg(FrameEvent::ReminderDeleted(reminder_id));

                        // reinitialize view, as we just deleted a reminder
                        self.current_view.init();
                    } else {
                        error!(target: "CLIENT_REMINDERS", "Failed to delete a reminder with id {reminder_id}!");
                    }
                }
            }
        };

        self.current_view.render(frame, area);
    }
}
