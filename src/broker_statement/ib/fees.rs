use crate::core::EmptyResult;
use crate::broker_statement::fees::Fee;
use crate::util::DecimalRestrictions;

use super::StatementParser;
use super::common::{Record, RecordParser};

pub struct FeesParser {}

impl RecordParser for FeesParser {
    fn skip_totals(&self) -> bool {
        true
    }

    fn parse(&mut self, parser: &mut StatementParser, record: &Record) -> EmptyResult {
        let currency = record.get_value("Currency")?;
        let date = record.parse_date("Date")?;
        let amount = record.parse_cash("Amount", currency, DecimalRestrictions::NonZero)?;
        Ok(parser.statement.fees.push(Fee::new(date, -amount, None)))
    }
}