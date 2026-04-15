use crate::app::enumeration::Gender;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: String,
    pub name: String,
    pub gender: Gender,
    pub account: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub mobile_phone: String,
    pub birthday: NaiveDate,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
