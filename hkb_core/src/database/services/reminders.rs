pub use crate::dtos::reminders::*;
use diesel::{
    sql_types::Date as SqlDateType, ExpressionMethods, IntoSql, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use hkb_date::date::SimpleDate;
use log::debug;

use crate::database::{
    self,
    models::reminders::{CreateReminder, Reminder, UpdateReminder},
    schema::reminders::{self, dsl as reminders_dsl},
    DatabaseResult,
};

impl From<Reminder> for ReminderData {
    fn from(val: Reminder) -> Self {
        ReminderData {
            id: val.id,
            note: val.note,
            remind_at: SimpleDate::parse_from_rfc3339(val.remind_at).unwrap(),
            created_at: SimpleDate::parse_from_rfc3339(val.created_at).unwrap(),
        }
    }
}

impl From<ReminderData> for Reminder {
    fn from(val: ReminderData) -> Self {
        Reminder {
            id: val.id,
            note: val.note,
            remind_at: val.remind_at.to_string(),
            created_at: val.created_at.to_string(),
        }
    }
}

impl From<CreateReminderData> for CreateReminder {
    fn from(val: CreateReminderData) -> Self {
        CreateReminder {
            note: val.note,
            remind_at: val.remind_at.to_string(),
            created_at: SimpleDate::local().to_string(),
        }
    }
}

impl From<UpdateReminderData> for UpdateReminder {
    fn from(val: UpdateReminderData) -> Self {
        UpdateReminder {
            note: val.note,
            remind_at: val.remind_at.map(|date| date.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum ReminderQueryOptions<'a> {
    RemindAtGe {
        date: SimpleDate,
    },
    RemindAtLe {
        date: SimpleDate,
    },
    RemindAtBetween {
        end_date: SimpleDate,
        start_date: SimpleDate,
    },
    WithIds {
        ids: &'a Vec<i64>,
    },
    WithoutIds {
        ids: &'a Vec<i64>,
    },
}

pub fn fetch_reminders(
    options: Option<Vec<ReminderQueryOptions>>,
) -> DatabaseResult<Vec<ReminderData>> {
    database::within_database(|conn| {
        debug!(target: "CORE_REMINDERS_SERVICE", "Fetching reminders with options: {options:?}");

        let mut query = reminders_dsl::reminders
            .select(Reminder::as_select())
            .order_by(reminders_dsl::id.asc())
            .into_boxed();

        if let Some(options) = options {
            for option in options {
                match option {
                    ReminderQueryOptions::RemindAtBetween {
                        end_date,
                        start_date,
                    } => {
                        query = query.filter(reminders_dsl::remind_at.between(
                            start_date.to_string().into_sql::<SqlDateType>(),
                            end_date.to_string().into_sql::<SqlDateType>(),
                        ));
                    }
                    ReminderQueryOptions::RemindAtGe { date } => {
                        query = query.filter(
                            reminders_dsl::remind_at.ge(date.to_string().into_sql::<SqlDateType>()),
                        );
                    }
                    ReminderQueryOptions::RemindAtLe { date } => {
                        query = query.filter(
                            reminders_dsl::remind_at.le(date.to_string().into_sql::<SqlDateType>()),
                        );
                    }
                    ReminderQueryOptions::WithIds { ids } => {
                        query = query.filter(reminders_dsl::id.eq_any(ids));
                    }
                    ReminderQueryOptions::WithoutIds { ids } => {
                        query = query.filter(diesel::dsl::not(reminders_dsl::id.eq_any(ids)));
                    }
                }
            }
        }

        let reminders: Vec<ReminderData> = query
            .get_results(conn)?
            .into_iter()
            .map(|reminder| reminder.into())
            .collect();

        debug!(target: "CORE_REMINDERS_SERVICE", "Reminders fetched: {}", reminders.len());

        Ok(reminders)
    })
}

pub fn fetch_reminder(id: i64) -> DatabaseResult<ReminderData> {
    database::within_database(|conn| {
        debug!(target: "CORE_REMINDERS_SERVICE", "Fetching reminder with id {id}");

        let reminder = reminders_dsl::reminders
            .find(id)
            .select(Reminder::as_select())
            .first(conn)?;

        debug!(target: "CORE_REMINDERS_SERVICE", "Found reminder {reminder:?}");

        Ok(reminder.into())
    })
}

pub fn create_reminder(reminder: CreateReminderData) -> DatabaseResult<ReminderData> {
    database::within_database(|conn| {
        debug!(target: "CORE_REMINDERS_SERVICE", "Creating reminder: {reminder:?}");

        let create_reminder: CreateReminder = reminder.into();
        let created_reminder = diesel::insert_into(reminders::table)
            .values(&create_reminder)
            .returning(Reminder::as_returning())
            .get_result(conn)?;

        debug!(target: "CORE_REMINDERS_SERVICE", "Reminder created. ID is: : {}", created_reminder.id);

        Ok(created_reminder.into())
    })
}

pub fn update_reminder(reminder: UpdateReminderData) -> DatabaseResult<ReminderData> {
    database::within_database(|conn| {
        debug!(target: "CORE_REMINDERS_SERVICE", "Updating reminder: {reminder:?}");

        let id = reminder.id;
        let update_reminder: UpdateReminder = reminder.into();
        let updated_reminder = diesel::update(reminders_dsl::reminders.find(id))
            .set(&update_reminder)
            .returning(Reminder::as_returning())
            .get_result(conn)?;

        debug!(target: "CORE_REMINDERS_SERVICE", "Reminder {id} updated!");

        Ok(updated_reminder.into())
    })
}

pub fn delete_reminders(option: ReminderQueryOptions) -> DatabaseResult<()> {
    database::within_database(|conn| {
        debug!(target: "CORE_REMINDERS_SERVICE", "Deleting reminders: {option:?}");

        match option {
            ReminderQueryOptions::RemindAtBetween {
                end_date,
                start_date,
            } => {
                diesel::delete(
                    reminders_dsl::reminders.filter(reminders_dsl::remind_at.between(
                        start_date.to_string().into_sql::<SqlDateType>(),
                        end_date.to_string().into_sql::<SqlDateType>(),
                    )),
                )
                .execute(conn)?;
            }
            ReminderQueryOptions::RemindAtGe { date } => {
                diesel::delete(reminders_dsl::reminders.filter(
                    reminders_dsl::remind_at.ge(date.to_string().into_sql::<SqlDateType>()),
                ))
                .execute(conn)?;
            }
            ReminderQueryOptions::RemindAtLe { date } => {
                diesel::delete(reminders_dsl::reminders.filter(
                    reminders_dsl::remind_at.le(date.to_string().into_sql::<SqlDateType>()),
                ))
                .execute(conn)?;
            }
            ReminderQueryOptions::WithIds { ids } => {
                diesel::delete(reminders_dsl::reminders.filter(reminders_dsl::id.eq_any(ids)))
                    .execute(conn)?;
            }
            ReminderQueryOptions::WithoutIds { ids } => {
                diesel::delete(
                    reminders_dsl::reminders
                        .filter(diesel::dsl::not(reminders_dsl::id.eq_any(ids))),
                )
                .execute(conn)?;
            }
        };

        debug!(target: "CORE_REMINDERS_SERVICE", "Deleted Reminders.");

        Ok(())
    })
}

pub fn delete_reminder(id: i64) -> DatabaseResult<()> {
    database::within_database(|conn| {
        debug!(target: "CORE_REMINDERS_SERVICE", "Deleting reminder: {id}");

        diesel::delete(reminders_dsl::reminders.find(id)).execute(conn)?;

        debug!(target: "CORE_REMINDERS_SERVICE", "Deleted Reminder: {id}");

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use self::database::{init_database, within_database};
    use ctor::ctor;
    use diesel::sql_query;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations};
    use hkb_date::date::SimpleDate;
    use hkb_date::duration::Duration;
    use serial_test::serial;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    use super::*;

    macro_rules! create_a_reminder {
        () => {{
            let date =
                SimpleDate::parse_from_str("2024-04-05 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
            let reminder_data = CreateReminderData {
                remind_at: date,
                note: "Testing".to_owned(),
            };

            create_reminder(reminder_data).unwrap()
        }};

        ($date:expr) => {{
            let reminder_data = CreateReminderData {
                remind_at: $date,
                note: "Testing".to_owned(),
            };

            create_reminder(reminder_data).unwrap()
        }};
    }

    macro_rules! truncate_table {
        () => {
            within_database(|conn| {
                sql_query("DELETE from reminders where 1=1")
                    .execute(conn)
                    .unwrap();

                Ok(())
            })
            .unwrap();
        };
    }

    #[test]
    #[ctor]
    fn init() {
        init_database(":memory:", vec![MIGRATIONS]).unwrap();
    }

    #[test]
    #[serial]
    fn it_can_fetch_a_reminder() {
        let reminder = create_a_reminder!();
        let fetched_reminder = fetch_reminder(reminder.id).unwrap();

        assert_eq!(reminder.id, fetched_reminder.id);
        assert_eq!(reminder.note, fetched_reminder.note);
        assert_eq!(
            reminder.remind_at.to_string(),
            fetched_reminder.remind_at.to_string()
        );
    }

    #[test]
    #[serial]
    fn it_can_fetch_reminders() {
        truncate_table!();

        let reminders = [create_a_reminder!(),
            create_a_reminder!(),
            create_a_reminder!()];
        let fetched_reminders = fetch_reminders(None).unwrap();

        assert_eq!(reminders.len(), fetched_reminders.len());

        for i in 0..fetched_reminders.len() {
            let reminder = fetched_reminders.get(i).unwrap();
            let expected_reminder = reminders.get(i).unwrap();

            assert_eq!(expected_reminder, reminder);
        }
    }

    #[test]
    #[serial]
    fn it_can_fetch_reminders_in_between() {
        truncate_table!();

        let d1 = SimpleDate::parse_from_str("2024-03-11 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let d2 = SimpleDate::parse_from_str("2024-03-12 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let start_date =
            SimpleDate::parse_from_str("2024-03-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end_date =
            SimpleDate::parse_from_str("2024-04-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let reminders = vec![
            create_a_reminder!(d1),
            create_a_reminder!(d2),
            create_a_reminder!(),
            create_a_reminder!(),
        ];
        let fetched_reminders =
            fetch_reminders(Some(vec![ReminderQueryOptions::RemindAtBetween {
                end_date,
                start_date,
            }]))
            .unwrap();

        assert_eq!(2, fetched_reminders.len());

        assert_eq!(reminders.first().unwrap(), fetched_reminders.first().unwrap());
        assert_eq!(reminders.get(1).unwrap(), fetched_reminders.get(1).unwrap());

        let start_date =
            SimpleDate::parse_from_str("2024-04-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end_date =
            SimpleDate::parse_from_str("2024-05-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let fetched_reminders =
            fetch_reminders(Some(vec![ReminderQueryOptions::RemindAtBetween {
                end_date,
                start_date,
            }]))
            .unwrap();

        assert_eq!(2, fetched_reminders.len());

        assert_eq!(reminders.get(2).unwrap(), fetched_reminders.first().unwrap());
        assert_eq!(reminders.get(3).unwrap(), fetched_reminders.get(1).unwrap());
    }

    #[test]
    #[serial]
    fn it_can_fetch_reminders_by_filtering_out_some_ids() {
        truncate_table!();

        let reminders = vec![
            create_a_reminder!(),
            create_a_reminder!(),
            create_a_reminder!(),
            create_a_reminder!(),
            create_a_reminder!(),
        ];
        let ids_to_exclude = vec![reminders[2].id, reminders[4].id];
        let fetched_reminders = fetch_reminders(Some(vec![ReminderQueryOptions::WithoutIds {
            ids: &ids_to_exclude,
        }]))
        .unwrap();

        assert_eq!(
            reminders.len() - ids_to_exclude.len(),
            fetched_reminders.len()
        );

        for id in ids_to_exclude.iter() {
            for reminder in fetched_reminders.iter() {
                assert_ne!(*id, reminder.id);
            }
        }
    }

    #[test]
    #[serial]
    fn it_can_create_a_reminder() {
        let date = SimpleDate::local();
        let reminder_data = CreateReminderData {
            remind_at: date,
            note: "Testing".to_owned(),
        };
        let reminder = create_reminder(reminder_data).unwrap();

        assert_eq!("Testing", reminder.note);
        assert_eq!(date.to_string(), reminder.remind_at.to_string());
    }

    #[test]
    #[serial]
    fn it_can_update_a_reminder() {
        let reminder = create_a_reminder!();
        let updated_reminder = update_reminder(UpdateReminderData {
            id: reminder.id,
            note: Some("Testing a new".to_owned()),
            remind_at: None,
        })
        .unwrap();

        assert_eq!("Testing a new", updated_reminder.note);
        assert_ne!(reminder.note, updated_reminder.note);
        assert_eq!(
            reminder.remind_at.to_string(),
            updated_reminder.remind_at.to_string()
        );
    }

    #[test]
    #[serial]
    fn it_can_update_date_of_a_reminder() {
        let reminder = create_a_reminder!();
        let date = SimpleDate::local()
            .add_duration(Duration::Month(1))
            .unwrap();

        let expected_date = date.to_string();
        let updated_reminder = update_reminder(UpdateReminderData {
            id: reminder.id,
            note: None,
            remind_at: Some(date),
        })
        .unwrap();

        assert_eq!(reminder.note, updated_reminder.note);
        assert_ne!(reminder.remind_at.to_string(), expected_date);
        assert_eq!(expected_date, updated_reminder.remind_at.to_string());
    }

    #[test]
    #[serial]
    fn it_can_delete_a_reminder() {
        let reminder = create_a_reminder!();
        let reminder2 = create_a_reminder!();

        assert!(fetch_reminder(reminder.id).is_ok());
        assert!(delete_reminder(reminder.id).is_ok());
        assert!(fetch_reminder(reminder.id).is_err());
        assert!(fetch_reminder(reminder2.id).is_ok());
    }

    #[test]
    #[serial]
    fn it_can_delete_multiple_reminders_at_once() {
        let reminder = create_a_reminder!();
        let reminder2 = create_a_reminder!();
        let reminder3 = create_a_reminder!();

        delete_reminders(ReminderQueryOptions::WithIds {
            ids: &vec![reminder.id, reminder2.id],
        })
        .unwrap();

        assert!(fetch_reminder(reminder.id).is_err());
        assert!(fetch_reminder(reminder2.id).is_err());
        assert!(fetch_reminder(reminder3.id).is_ok());
    }
}
