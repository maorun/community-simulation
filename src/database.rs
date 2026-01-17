//! SQLite database export functionality for simulation results.
//!
//! This module provides functionality to export simulation results to a SQLite database
//! for long-term storage and analysis. The export is optional and does not affect
//! the core simulation logic.
//!
//! # Examples
//!
//! ```ignore
//! use simulation_framework::result::SimulationResult;
//! use simulation_framework::database::export_to_sqlite;
//! use simulation_framework::{SimulationConfig, SimulationEngine};
//!
//! // Create and run a simulation
//! let config = SimulationConfig::default();
//! let mut engine = SimulationEngine::new(config);
//! let result = engine.run();
//!
//! // Export results to SQLite database
//! export_to_sqlite(&result, "simulation_results.db")?;
//! ```

use crate::result::SimulationResult;
use rusqlite::{Connection, Result};

/// Exports simulation results to a SQLite database.
///
/// Creates a new SQLite database file (or overwrites if it exists) with the following tables:
/// - `summary_statistics`: Overall simulation statistics
/// - `money_distribution`: Final money distribution per person
/// - `reputation_distribution`: Final reputation distribution per person  
/// - `skill_prices`: Final skill prices
///
/// # Arguments
///
/// * `result` - The simulation result to export
/// * `db_path` - Path to the SQLite database file
///
/// # Errors
///
/// Returns an error if database creation or data insertion fails.
///
/// # Examples
///
/// ```ignore
/// use simulation_framework::database::export_to_sqlite;
/// use simulation_framework::{SimulationConfig, SimulationEngine};
///
/// let config = SimulationConfig::default();
/// let mut engine = SimulationEngine::new(config);
/// let result = engine.run();
/// export_to_sqlite(&result, "results.db")?;
/// ```
pub fn export_to_sqlite(result: &SimulationResult, db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;

    // Create tables
    create_tables(&conn)?;

    // Insert data
    insert_summary_statistics(&conn, result)?;
    insert_money_distribution(&conn, result)?;
    insert_reputation_distribution(&conn, result)?;
    insert_skill_prices(&conn, result)?;

    Ok(())
}

/// Creates the database schema.
fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS summary_statistics (
            id INTEGER PRIMARY KEY,
            total_steps INTEGER NOT NULL,
            total_duration REAL NOT NULL,
            active_persons INTEGER NOT NULL,
            avg_money REAL NOT NULL,
            median_money REAL NOT NULL,
            std_dev_money REAL NOT NULL,
            min_money REAL NOT NULL,
            max_money REAL NOT NULL,
            gini_coefficient REAL NOT NULL,
            herfindahl_index REAL NOT NULL,
            avg_reputation REAL NOT NULL,
            median_reputation REAL NOT NULL,
            total_trades INTEGER NOT NULL,
            total_volume REAL NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS money_distribution (
            id INTEGER PRIMARY KEY,
            person_index INTEGER NOT NULL,
            money REAL NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS reputation_distribution (
            id INTEGER PRIMARY KEY,
            person_index INTEGER NOT NULL,
            reputation REAL NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS skill_prices (
            id INTEGER PRIMARY KEY,
            skill_id TEXT NOT NULL,
            price REAL NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// Inserts summary statistics into the database.
fn insert_summary_statistics(conn: &Connection, result: &SimulationResult) -> Result<()> {
    conn.execute(
        "INSERT INTO summary_statistics (
            total_steps, total_duration, active_persons,
            avg_money, median_money, std_dev_money, min_money, max_money,
            gini_coefficient, herfindahl_index,
            avg_reputation, median_reputation,
            total_trades, total_volume
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            result.total_steps as i64,
            result.total_duration,
            result.active_persons as i64,
            result.money_statistics.average,
            result.money_statistics.median,
            result.money_statistics.std_dev,
            result.money_statistics.min_money,
            result.money_statistics.max_money,
            result.money_statistics.gini_coefficient,
            result.money_statistics.herfindahl_index,
            result.reputation_statistics.average,
            result.reputation_statistics.median,
            result.trade_volume_statistics.total_trades as i64,
            result.trade_volume_statistics.total_volume,
        ],
    )?;

    Ok(())
}

/// Inserts money distribution data into the database.
fn insert_money_distribution(conn: &Connection, result: &SimulationResult) -> Result<()> {
    let mut stmt =
        conn.prepare("INSERT INTO money_distribution (person_index, money) VALUES (?1, ?2)")?;

    for (index, money) in result.final_money_distribution.iter().enumerate() {
        stmt.execute(rusqlite::params![index as i64, money])?;
    }

    Ok(())
}

/// Inserts reputation distribution data into the database.
fn insert_reputation_distribution(conn: &Connection, result: &SimulationResult) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO reputation_distribution (person_index, reputation) VALUES (?1, ?2)",
    )?;

    for (index, reputation) in result.final_reputation_distribution.iter().enumerate() {
        stmt.execute(rusqlite::params![index as i64, reputation])?;
    }

    Ok(())
}

/// Inserts skill prices into the database.
fn insert_skill_prices(conn: &Connection, result: &SimulationResult) -> Result<()> {
    let mut stmt = conn.prepare("INSERT INTO skill_prices (skill_id, price) VALUES (?1, ?2)")?;

    for skill_price in &result.final_skill_prices {
        stmt.execute(rusqlite::params![skill_price.id, skill_price.price])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::{
        MoneyStats, ReputationStats, SavingsStats, SkillPriceInfo, TradeVolumeStats,
    };
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_to_sqlite_creates_database() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let result = create_test_result();
        export_to_sqlite(&result, db_path).unwrap();

        // Verify database was created
        let conn = Connection::open(db_path).unwrap();

        // Check that tables exist
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 4);
    }

    #[test]
    fn test_export_to_sqlite_inserts_summary_stats() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let result = create_test_result();
        export_to_sqlite(&result, db_path).unwrap();

        let conn = Connection::open(db_path).unwrap();

        // Verify summary statistics were inserted
        let (total_steps, active_persons): (i64, i64) = conn
            .query_row(
                "SELECT total_steps, active_persons FROM summary_statistics WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(total_steps as usize, 100);
        assert_eq!(active_persons as usize, 10);
    }

    #[test]
    fn test_export_to_sqlite_inserts_money_distribution() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let result = create_test_result();
        export_to_sqlite(&result, db_path).unwrap();

        let conn = Connection::open(db_path).unwrap();

        // Verify money distribution was inserted
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM money_distribution", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(count, 10);
    }

    #[test]
    fn test_export_to_sqlite_inserts_reputation_distribution() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let result = create_test_result();
        export_to_sqlite(&result, db_path).unwrap();

        let conn = Connection::open(db_path).unwrap();

        // Verify reputation distribution was inserted
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM reputation_distribution", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(count, 10);
    }

    #[test]
    fn test_export_to_sqlite_inserts_skill_prices() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let result = create_test_result();
        export_to_sqlite(&result, db_path).unwrap();

        let conn = Connection::open(db_path).unwrap();

        // Verify skill prices were inserted
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM skill_prices", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 3);
    }

    fn create_test_result() -> SimulationResult {
        SimulationResult {
            total_steps: 100,
            total_duration: 1.5,
            step_times: vec![],
            active_persons: 10,
            failed_steps: 0,
            final_money_distribution: vec![
                100.0, 120.0, 80.0, 150.0, 90.0, 110.0, 95.0, 105.0, 130.0, 85.0,
            ],
            money_statistics: MoneyStats {
                average: 106.5,
                median: 102.5,
                std_dev: 20.5,
                min_money: 80.0,
                max_money: 150.0,
                gini_coefficient: 0.15,
                herfindahl_index: 1050.0,
                top_10_percent_share: 0.25,
                top_1_percent_share: 0.15,
                bottom_50_percent_share: 0.45,
            },
            final_reputation_distribution: vec![
                1.0, 1.1, 0.9, 1.2, 1.0, 1.1, 0.95, 1.05, 1.15, 0.85,
            ],
            reputation_statistics: ReputationStats {
                average: 1.025,
                median: 1.025,
                std_dev: 0.12,
                min_reputation: 0.85,
                max_reputation: 1.2,
            },
            final_savings_distribution: vec![],
            savings_statistics: SavingsStats {
                total_savings: 0.0,
                average_savings: 0.0,
                median_savings: 0.0,
                min_savings: 0.0,
                max_savings: 0.0,
            },
            credit_score_statistics: None,
            trade_volume_statistics: TradeVolumeStats {
                total_trades: 500,
                total_volume: 5000.0,
                avg_trades_per_step: 5.0,
                avg_volume_per_step: 50.0,
                avg_transaction_value: 10.0,
                min_trades_per_step: 2,
                max_trades_per_step: 8,
            },
            trades_per_step: vec![],
            volume_per_step: vec![],
            final_skill_prices: vec![
                SkillPriceInfo {
                    id: "Skill1".to_string(),
                    price: 15.0,
                },
                SkillPriceInfo {
                    id: "Skill2".to_string(),
                    price: 12.0,
                },
                SkillPriceInfo {
                    id: "Skill3".to_string(),
                    price: 18.0,
                },
            ],
            most_valuable_skill: Some(SkillPriceInfo {
                id: "Skill3".to_string(),
                price: 18.0,
            }),
            least_valuable_skill: Some(SkillPriceInfo {
                id: "Skill2".to_string(),
                price: 12.0,
            }),
            skill_price_history: std::collections::HashMap::new(),
            final_persons_data: vec![],
            total_fees_collected: 0.0,
            per_skill_trade_stats: vec![],
            skill_market_concentration: None,
            wealth_stats_history: vec![],
            trading_partner_statistics: crate::result::TradingPartnerStats {
                per_person: vec![],
                network_metrics: crate::result::NetworkMetrics {
                    avg_unique_partners: 0.0,
                    network_density: 0.0,
                    most_active_pair: None,
                },
            },
            centrality_analysis: None,
            mobility_statistics: None,
            failed_trade_statistics: crate::result::FailedTradeStats {
                total_failed_attempts: 0,
                failure_rate: 0.0,
                avg_failed_per_step: 0.0,
                min_failed_per_step: 0,
                max_failed_per_step: 0,
            },
            failed_attempts_per_step: vec![],
            black_market_statistics: None,
            total_taxes_collected: None,
            total_taxes_redistributed: None,
            loan_statistics: None,
            contract_statistics: None,
            education_statistics: None,
            mentorship_statistics: None,
            certification_statistics: None,
            environment_statistics: None,
            friendship_statistics: None,
            trade_agreement_statistics: None,
            group_statistics: None,
            quality_statistics: None,
            events: None,
        }
    }
}
