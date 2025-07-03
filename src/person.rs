use serde::{Deserialize, Serialize};
use crate::skill::{Skill, SkillId};

pub type PersonId = usize;
pub type UrgencyLevel = u8; // Define UrgencyLevel (e.g., 1-3, higher is more urgent)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub step: usize,
    pub skill_id: SkillId,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub counterparty_id: Option<PersonId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeededSkillItem {
    pub id: SkillId,
    pub urgency: UrgencyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: PersonId,
    pub money: f64,
    pub own_skill: Skill,
    // Now stores tuples of (SkillId, UrgencyLevel)
    pub needed_skills: Vec<NeededSkillItem>,
    pub transaction_history: Vec<Transaction>,
    // Stores SkillIds that have been satisfied in the current step
    pub satisfied_needs_current_step: Vec<SkillId>,
}

impl Person {
    pub fn new(id: PersonId, initial_money: f64, own_skill: Skill) -> Self {
        Person {
            id,
            money: initial_money,
            own_skill,
            needed_skills: Vec::new(),
            transaction_history: Vec::new(),
            satisfied_needs_current_step: Vec::new(),
        }
    }

    pub fn can_afford(&self, amount: f64) -> bool {
        self.money >= amount
    }

    pub fn record_transaction(
        &mut self,
        step: usize,
        skill_id: SkillId,
        transaction_type: TransactionType,
        amount: f64,
        counterparty_id: Option<PersonId>,
    ) {
        let transaction = Transaction {
            step,
            skill_id,
            transaction_type,
            amount,
            counterparty_id,
        };
        self.transaction_history.push(transaction);
    }
}
