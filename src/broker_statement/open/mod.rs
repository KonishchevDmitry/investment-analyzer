mod assets;
mod cash_flows;
mod common;
mod report;
mod trades;

#[cfg(test)] use crate::brokers::Broker;
#[cfg(test)] use crate::config::Config;
use crate::core::GenericResult;
#[cfg(test)] use crate::taxes::TaxRemapping;

#[cfg(test)] use super::{BrokerStatement};
use super::{BrokerStatementReader, PartialBrokerStatement};

use report::BrokerReport;

pub struct StatementReader {
}

impl StatementReader {
    pub fn new() -> GenericResult<Box<dyn BrokerStatementReader>> {
        Ok(Box::new(StatementReader{}))
    }
}

impl BrokerStatementReader for StatementReader {
    fn is_statement(&self, path: &str) -> GenericResult<bool> {
        Ok(path.ends_with(".xml"))
    }

    fn read(&mut self, path: &str, _is_last: bool) -> GenericResult<PartialBrokerStatement> {
        let mut statement = PartialBrokerStatement::new();
        read_statement(path)?.parse(&mut statement)?;
        statement.validate()
    }
}

fn read_statement(path: &str) -> GenericResult<BrokerReport> {
    let data = std::fs::read(path)?;

    let (data, _, errors) = encoding_rs::WINDOWS_1251.decode(data.as_slice());
    if errors {
        return Err!("Got an invalid Windows-1251 encoded data");
    }

    Ok(serde_xml_rs::from_str(&data).map_err(|e| e.to_string())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_real() {
        let broker = Broker::Open.get_info(&Config::mock(), None).unwrap();

        let statement = BrokerStatement::read(
            broker, "testdata/open-broker", &hashmap!{}, &hashmap!{}, TaxRemapping::new(), true).unwrap();

        assert!(!statement.cash_flows.is_empty());
        assert!(!statement.cash_assets.is_empty());

        assert!(!statement.fees.is_empty());
        assert!(statement.idle_cash_interest.is_empty());

        assert!(statement.forex_trades.is_empty());
        assert!(!statement.stock_buys.is_empty());
        assert!(!statement.stock_sells.is_empty());
        assert!(statement.dividends.is_empty());

        assert!(!statement.open_positions.is_empty());
        assert!(!statement.instrument_names.is_empty());
    }
}