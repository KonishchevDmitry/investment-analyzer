use lazy_static::lazy_static;
use regex::Regex;

use crate::broker_statement::taxes::TaxId;
use crate::core::{GenericResult, EmptyResult};
use crate::util::DecimalRestrictions;

use super::StatementParser;
use super::common::{Record, RecordParser};

// Every year IB has to adjust the 1042 withholding (i.e. withholding on US dividends paid to non-US
// accounts) to reflect dividend reclassifications. This is typically done in February the following
// year. As such, the majority of these adjustments are refunds to customers. The typical case is
// when IB's best information at the time of paying a dividend indicates that the distribution is an
// ordinary dividend (and therefore subject to withholding), then later at year end, the dividend is
// reclassified as Return of Capital, proceeds, or capital gains (all of which are not subject to
// 1042 withholding).
//
// So withholding in previous year's statements should be reviewed against February statement's
// withholding adjustments. As it turns out, dates may not match.
//
// At this time we match dividends on taxes using (date, symbol) pair. Matching by description
// turned out to be too fragile.
pub struct WithholdingTaxParser {}

impl RecordParser for WithholdingTaxParser {
    fn skip_totals(&self) -> bool {
        true
    }

    fn parse(&mut self, parser: &mut StatementParser, record: &Record) -> EmptyResult {
        let currency = record.get_value("Currency")?;
        let description = record.get_value("Description")?;
        let date = parser.tax_remapping.map(record.parse_date("Date")?, description);

        let issuer = parse_tax_description(description)?;
        let tax_id = TaxId::new(date, &issuer);

        // Tax amount is represented as a negative number.
        //
        // Positive number is used to cancel a previous tax payment and usually followed by another
        // negative number.
        let tax = record.parse_cash("Amount", currency, DecimalRestrictions::NonZero)?;
        let accruals = parser.statement.tax_accruals.entry(tax_id).or_default();

        if tax.is_positive() {
            accruals.reverse(tax);
        } else {
            accruals.add(-tax);
        }

        Ok(())
    }
}

fn parse_tax_description(description: &str) -> GenericResult<String> {
    lazy_static! {
        static ref DESCRIPTION_REGEX: Regex = Regex::new(
            r"^(?P<issuer>[A-Z]+) ?\([A-Z0-9]+\) .+ - [A-Z]{2} Tax$").unwrap();
    }

    let captures = DESCRIPTION_REGEX.captures(description).ok_or_else(|| format!(
        "Unexpected tax description: {:?}", description))?;

    Ok(captures.name("issuer").unwrap().as_str().to_owned())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest(description, symbol,
        case("BND (US9219378356) Cash Dividend USD 0.181007 - US Tax", "BND"),
        case("BND(US9219378356) Cash Dividend USD 0.193413 per Share - US Tax", "BND"),
        case("BND(US9219378356) Cash Dividend 0.18366600 USD per Share - US Tax", "BND"),
        case("BND(43645828) Cash Dividend 0.19446400 USD per Share - US Tax", "BND"),
        case("UNIT(US91325V1089) Payment in Lieu of Dividend - US Tax", "UNIT"),
        case("ETN(IE00B8KQN827) Cash Dividend USD 0.73 per Share - IE Tax", "ETN"),
    )]
    fn tax_parsing(description: &str, symbol: &str) {
        assert_eq!(parse_tax_description(description).unwrap(), symbol.to_owned());
    }
}
