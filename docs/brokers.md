# Broker specific

<a name="interactive-brokers"></a>
## Interactive Brokers

The program expects Activity Statements in `*.csv` format for broker statements (`Reports -> Statements -> Activity`).

<a name="ib-trade-settle-date"></a>
### T+2 trading mode

Activity statements don't provide trade settle date. So by default all calculations will be made in T+0 mode and
`simulate-sell` and `tax-statement` commands will complain on this via warning message because it affects correctness of
tax calculations.

Trade settle date may be obtained from Trade Confirmation Report. To do this, create a Trade Confirmation Flex Query in
the IB `Reports -> Flex Queries` tab with the following parameters:

![Trade Confirmation Flex Query Parameters](images/trade-confirmation-parameters.png?raw=true "Trade Confirmation Flex Query Parameters")

and download the statements for all periods where you have any trades. Investments will catch these statements and use
information from them for calculations in T+2 mode.

<a name="ib-dividend-reclassifications"></a>
### Dividend reclassifications

Every year IB has to adjust the 1042 withholding (i.e. withholding on US dividends paid to non-US accounts) to reflect
dividend reclassifications. This is typically done in February the following year. As such, the majority of these
adjustments are refunds to customers. The typical case is when IB's best information at the time of paying a dividend
indicates that the distribution is an ordinary dividend (and therefore subject to withholding), then later at year end,
the dividend is reclassified as Return of Capital, proceeds, or capital gains (all of which are not subject to 1042
withholding).

<a name="ib-tax-remapping"></a>
Investments finds such reclassifications and handles them properly, but at this time it matches dividends to taxes using
(date, symbol) pair, because matching by description turned out to be too fragile. As it turns out sometimes dates of
reclassified taxes don't match dividend dates. To workaround such cases there is `tax_remapping` configuration option
using which you can manually map reclassified tax to date of its origin dividend.


<a name="tinkoff"></a>
## Тинькофф

Dividends are parsed out from the broker statements, but without withheld tax information. See
[#26](https://github.com/KonishchevDmitry/investments/issues/26) (I need an example of broker statement + foreign income
report).


<a name="bcs"></a>
<a name="open-broker"></a>
## БКС and Открытие Брокер

Dividends aren't parsed out from broker statements yet. I use FinEx ETF which don't pay dividends, so I don't have an
example of how they are look like in the broker statements (see [#19](https://github.com/KonishchevDmitry/investments/issues/19)).