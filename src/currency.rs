use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a currency
pub type CurrencyId = String;

/// Represents a currency with its exchange rate relative to the base currency.
///
/// The base currency has an exchange rate of 1.0. Other currencies are valued
/// relative to this base. For example, if currency "USD" is the base with rate 1.0,
/// and "EUR" has rate 1.2, then 1 EUR = 1.2 USD.
///
/// # Examples
///
/// ```
/// use community_simulation::currency::Currency;
///
/// let usd = Currency::new("USD".to_string(), 1.0);
/// assert_eq!(usd.id, "USD");
/// assert_eq!(usd.exchange_rate, 1.0);
///
/// let eur = Currency::new("EUR".to_string(), 1.2);
/// assert_eq!(eur.id, "EUR");
/// assert_eq!(eur.exchange_rate, 1.2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Currency {
    /// Unique identifier for this currency (e.g., "USD", "EUR", "JPY")
    pub id: CurrencyId,

    /// Exchange rate relative to the base currency.
    /// Base currency = 1.0, other currencies valued relative to base.
    /// Must be positive.
    pub exchange_rate: f64,
}

impl Currency {
    /// Creates a new currency with the given ID and exchange rate.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the currency
    /// * `exchange_rate` - Exchange rate relative to base currency (must be positive)
    ///
    /// # Panics
    ///
    /// Panics if exchange_rate is not positive.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::currency::Currency;
    ///
    /// let currency = Currency::new("USD".to_string(), 1.0);
    /// assert_eq!(currency.id, "USD");
    /// ```
    pub fn new(id: CurrencyId, exchange_rate: f64) -> Self {
        assert!(exchange_rate > 0.0, "Exchange rate must be positive");
        Currency { id, exchange_rate }
    }

    /// Converts an amount from this currency to another currency.
    ///
    /// The conversion is done through the base currency:
    /// 1. Convert from this currency to base (divide by this.exchange_rate)
    /// 2. Convert from base to target (multiply by target.exchange_rate)
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount in this currency
    /// * `target` - Target currency to convert to
    ///
    /// # Returns
    ///
    /// The equivalent amount in the target currency
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::currency::Currency;
    ///
    /// let usd = Currency::new("USD".to_string(), 1.0);
    /// let eur = Currency::new("EUR".to_string(), 1.2);
    ///
    /// // Convert 100 USD to EUR
    /// let eur_amount = usd.convert_to(100.0, &eur);
    /// assert!((eur_amount - 120.0).abs() < 0.001);
    ///
    /// // Convert back: 120 EUR to USD
    /// let usd_amount = eur.convert_to(120.0, &usd);
    /// assert!((usd_amount - 100.0).abs() < 0.001);
    /// ```
    pub fn convert_to(&self, amount: f64, target: &Currency) -> f64 {
        // Convert to base currency, then to target currency
        let base_amount = amount / self.exchange_rate;
        base_amount * target.exchange_rate
    }
}

/// Manages multiple currencies and their exchange rates.
///
/// This struct provides a central registry for all currencies in the simulation
/// and handles currency conversions. By default, it contains a single base currency.
///
/// # Examples
///
/// ```
/// use community_simulation::currency::{CurrencySystem, Currency};
///
/// let mut system = CurrencySystem::default();
/// assert_eq!(system.currencies.len(), 1);
/// assert!(system.currencies.contains_key("BASE"));
///
/// // Add a new currency
/// system.add_currency(Currency::new("EUR".to_string(), 1.2));
/// assert_eq!(system.currencies.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencySystem {
    /// Map of currency ID to Currency
    pub currencies: HashMap<CurrencyId, Currency>,

    /// ID of the base currency (typically "BASE" or "USD")
    pub base_currency_id: CurrencyId,
}

impl Default for CurrencySystem {
    /// Creates a default currency system with a single base currency.
    ///
    /// The base currency has ID "BASE" and exchange rate 1.0.
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::currency::CurrencySystem;
    ///
    /// let system = CurrencySystem::default();
    /// assert_eq!(system.base_currency_id, "BASE");
    /// assert_eq!(system.currencies.len(), 1);
    /// ```
    fn default() -> Self {
        let mut currencies = HashMap::new();
        let base = Currency::new("BASE".to_string(), 1.0);
        currencies.insert("BASE".to_string(), base);

        CurrencySystem { currencies, base_currency_id: "BASE".to_string() }
    }
}

impl CurrencySystem {
    /// Creates a new currency system with the given base currency.
    ///
    /// # Arguments
    ///
    /// * `base_currency` - The base currency for the system
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::currency::{CurrencySystem, Currency};
    ///
    /// let usd = Currency::new("USD".to_string(), 1.0);
    /// let system = CurrencySystem::new(usd.clone());
    /// assert_eq!(system.base_currency_id, "USD");
    /// ```
    pub fn new(base_currency: Currency) -> Self {
        let mut currencies = HashMap::new();
        let base_id = base_currency.id.clone();
        currencies.insert(base_id.clone(), base_currency);

        CurrencySystem { currencies, base_currency_id: base_id }
    }

    /// Adds a currency to the system.
    ///
    /// If a currency with the same ID already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `currency` - The currency to add
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::currency::{CurrencySystem, Currency};
    ///
    /// let mut system = CurrencySystem::default();
    /// let eur = Currency::new("EUR".to_string(), 1.2);
    /// system.add_currency(eur);
    /// assert!(system.currencies.contains_key("EUR"));
    /// ```
    pub fn add_currency(&mut self, currency: Currency) {
        self.currencies.insert(currency.id.clone(), currency);
    }

    /// Gets a currency by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The currency ID to look up
    ///
    /// # Returns
    ///
    /// Some reference to the currency if found, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::currency::CurrencySystem;
    ///
    /// let system = CurrencySystem::default();
    /// let base = system.get_currency("BASE");
    /// assert!(base.is_some());
    /// assert_eq!(base.unwrap().exchange_rate, 1.0);
    /// ```
    pub fn get_currency(&self, id: &str) -> Option<&Currency> {
        self.currencies.get(id)
    }

    /// Converts an amount from one currency to another.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to convert
    /// * `from_currency_id` - Source currency ID
    /// * `to_currency_id` - Target currency ID
    ///
    /// # Returns
    ///
    /// Some converted amount if both currencies exist, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::currency::{CurrencySystem, Currency};
    ///
    /// let mut system = CurrencySystem::default();
    /// system.add_currency(Currency::new("EUR".to_string(), 1.2));
    ///
    /// let result = system.convert(100.0, "BASE", "EUR");
    /// assert!(result.is_some());
    /// assert!((result.unwrap() - 120.0).abs() < 0.001);
    /// ```
    pub fn convert(
        &self,
        amount: f64,
        from_currency_id: &str,
        to_currency_id: &str,
    ) -> Option<f64> {
        let from = self.get_currency(from_currency_id)?;
        let to = self.get_currency(to_currency_id)?;
        Some(from.convert_to(amount, to))
    }

    /// Gets the base currency.
    ///
    /// # Returns
    ///
    /// Reference to the base currency
    ///
    /// # Panics
    ///
    /// Panics if the base currency is not in the system (should never happen if constructed properly)
    ///
    /// # Examples
    ///
    /// ```
    /// use community_simulation::currency::CurrencySystem;
    ///
    /// let system = CurrencySystem::default();
    /// let base = system.get_base_currency();
    /// assert_eq!(base.id, "BASE");
    /// ```
    pub fn get_base_currency(&self) -> &Currency {
        self.currencies
            .get(&self.base_currency_id)
            .expect("Base currency must exist in the system")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_creation() {
        let currency = Currency::new("USD".to_string(), 1.0);
        assert_eq!(currency.id, "USD");
        assert_eq!(currency.exchange_rate, 1.0);
    }

    #[test]
    #[should_panic(expected = "Exchange rate must be positive")]
    fn test_currency_negative_rate() {
        Currency::new("INVALID".to_string(), -1.0);
    }

    #[test]
    #[should_panic(expected = "Exchange rate must be positive")]
    fn test_currency_zero_rate() {
        Currency::new("INVALID".to_string(), 0.0);
    }

    #[test]
    fn test_currency_conversion() {
        let usd = Currency::new("USD".to_string(), 1.0);
        let eur = Currency::new("EUR".to_string(), 1.2);

        // Convert 100 USD to EUR (should be 120 EUR)
        let eur_amount = usd.convert_to(100.0, &eur);
        assert!((eur_amount - 120.0).abs() < 0.001);

        // Convert 120 EUR to USD (should be 100 USD)
        let usd_amount = eur.convert_to(120.0, &usd);
        assert!((usd_amount - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_currency_conversion_same_currency() {
        let usd = Currency::new("USD".to_string(), 1.0);
        let amount = usd.convert_to(100.0, &usd);
        assert_eq!(amount, 100.0);
    }

    #[test]
    fn test_currency_system_default() {
        let system = CurrencySystem::default();
        assert_eq!(system.base_currency_id, "BASE");
        assert_eq!(system.currencies.len(), 1);
        assert!(system.currencies.contains_key("BASE"));
    }

    #[test]
    fn test_currency_system_new() {
        let usd = Currency::new("USD".to_string(), 1.0);
        let system = CurrencySystem::new(usd);
        assert_eq!(system.base_currency_id, "USD");
        assert_eq!(system.currencies.len(), 1);
    }

    #[test]
    fn test_currency_system_add_currency() {
        let mut system = CurrencySystem::default();
        system.add_currency(Currency::new("EUR".to_string(), 1.2));
        system.add_currency(Currency::new("JPY".to_string(), 0.01));

        assert_eq!(system.currencies.len(), 3);
        assert!(system.currencies.contains_key("EUR"));
        assert!(system.currencies.contains_key("JPY"));
    }

    #[test]
    fn test_currency_system_get_currency() {
        let mut system = CurrencySystem::default();
        system.add_currency(Currency::new("EUR".to_string(), 1.2));

        let eur = system.get_currency("EUR");
        assert!(eur.is_some());
        assert_eq!(eur.unwrap().exchange_rate, 1.2);

        let missing = system.get_currency("NONEXISTENT");
        assert!(missing.is_none());
    }

    #[test]
    fn test_currency_system_convert() {
        let mut system = CurrencySystem::default();
        system.add_currency(Currency::new("EUR".to_string(), 1.2));
        system.add_currency(Currency::new("JPY".to_string(), 0.01));

        // BASE to EUR
        let result = system.convert(100.0, "BASE", "EUR");
        assert!(result.is_some());
        assert!((result.unwrap() - 120.0).abs() < 0.001);

        // EUR to JPY
        let result = system.convert(100.0, "EUR", "JPY");
        assert!(result.is_some());
        // 100 EUR -> 100/1.2 = 83.33 BASE -> 83.33 * 0.01 = 0.833 JPY
        assert!((result.unwrap() - 0.833).abs() < 0.01);

        // Invalid conversion
        let result = system.convert(100.0, "INVALID", "EUR");
        assert!(result.is_none());
    }

    #[test]
    fn test_currency_system_get_base_currency() {
        let system = CurrencySystem::default();
        let base = system.get_base_currency();
        assert_eq!(base.id, "BASE");
        assert_eq!(base.exchange_rate, 1.0);
    }

    #[test]
    fn test_currency_serialization() {
        let currency = Currency::new("USD".to_string(), 1.5);
        let json = serde_json::to_string(&currency).unwrap();
        let deserialized: Currency = serde_json::from_str(&json).unwrap();
        assert_eq!(currency, deserialized);
    }

    #[test]
    fn test_currency_system_serialization() {
        let mut system = CurrencySystem::default();
        system.add_currency(Currency::new("EUR".to_string(), 1.2));

        let json = serde_json::to_string(&system).unwrap();
        let deserialized: CurrencySystem = serde_json::from_str(&json).unwrap();

        assert_eq!(system.base_currency_id, deserialized.base_currency_id);
        assert_eq!(system.currencies.len(), deserialized.currencies.len());
    }
}
