use crate::core::GenericResult;
use crate::currency::Cash;
use crate::currency::converter::CurrencyConverter;
use crate::localities::Country;
use crate::taxes::IncomeType;
use crate::time::Date;
use chrono::Datelike;

pub struct IdleCashInterest {
    pub date: Date,
    pub amount: Cash, // May be negative
}

impl IdleCashInterest {
    pub fn new(date: Date, amount: Cash) -> IdleCashInterest {
        IdleCashInterest {
            date, amount
        }
    }

    pub fn tax_to_pay(&self, country: &Country, converter: &CurrencyConverter) -> GenericResult<Cash> {
        let amount = converter.convert_to_cash_rounding(self.date, self.amount, country.currency)?;
        Ok(country.tax_to_pay(IncomeType::Interest, self.date.year(), amount, None))
    }
}