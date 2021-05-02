use crate::db::schema::{AssetType, assets, currency_rates, quotes, telemetry};
use crate::types::{Date, DateTime};

#[derive(Insertable, Queryable)]
#[table_name="assets"]
pub struct Asset {
    pub portfolio: String,
    pub asset_type: AssetType,
    pub symbol: String,
    pub quantity: String,
}

#[derive(Insertable)]
#[table_name="currency_rates"]
pub struct NewCurrencyRate<'a> {
    pub currency: &'a str,
    pub date: Date,
    pub price: Option<String>,
}

#[derive(Insertable)]
#[table_name="quotes"]
pub struct NewQuote<'a> {
    pub symbol: &'a str,
    pub time: DateTime,
    pub currency: &'a str,
    pub price: String,
}

#[derive(Insertable)]
#[table_name="telemetry"]
pub struct NewTelemetryRecord {
    pub payload: String,
}