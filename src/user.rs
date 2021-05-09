use serde::Deserialize;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TransactionRequset {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

#[derive(Eq, PartialEq)]
pub enum TransactionState {
    Normal,
    Disputed,
    Chargedback,
}
pub struct Transatcion {
    pub tx_type: TransactionType,
    pub amount: f64,
    pub state: TransactionState,
}

#[derive(Default)]
pub struct Account {
    pub total: f64,
    pub held: f64,
}

impl Account {
    pub fn avalible(&self) -> f64 {
        self.total - self.held
    }
}

#[derive(Default)]
pub struct User {
    pub id: u16,
    pub account: Account,
    pub tx_history: HashMap<u32, Transatcion>,
    pub frozen: bool,
}

impl User {
    pub fn process_tx(&mut self, tx: TransactionRequset) -> Result<(), String> {
        if self.frozen {
            return Err("Account frozen".to_string());
        }

        match tx.r#type {
            TransactionType::Deposit => self.process_deposit(tx)?,
            TransactionType::Withdrawal => self.process_withdrawal(tx)?,
            TransactionType::Dispute => self.process_dispute(tx)?,
            TransactionType::Resolve => self.process_resolve(tx)?,
            TransactionType::Chargeback => self.process_chargeback(tx)?,
        }

        Ok(())
    }

    fn process_deposit(&mut self, tx: TransactionRequset) -> Result<(), String> {
        if self.tx_history.contains_key(&tx.tx) {
            return Err(format!("Doubled transaction id. Ignored.\n{:?}", tx));
        }

        if tx.amount.is_none() || *tx.amount.as_ref().unwrap() == 0.0 {
            return Err(format!("Invalid transaction data. Ignored.\n{:?}", tx));
        }

        self.tx_history.insert(
            tx.tx,
            Transatcion {
                tx_type: tx.r#type,
                amount: tx.amount.clone().unwrap(),
                state: TransactionState::Normal,
            },
        );

        self.account.total += tx.amount.unwrap();

        Ok(())
    }

    fn process_withdrawal(&mut self, tx: TransactionRequset) -> Result<(), String> {
        if self.tx_history.contains_key(&tx.tx) {
            return Err(format!("Doubled transaction id. Ignored.\n{:?}", tx));
        }

        let amount = if tx.amount.is_none() || *tx.amount.as_ref().unwrap() == 0.0 {
            return Err(format!("Invalid transaction data. Ignored.\n{:?}", tx));
        } else {
            tx.amount.unwrap()
        };

        if amount > self.account.avalible() {
            return Err("Insufficient funds.".to_string());
        }

        self.tx_history.insert(
            tx.tx,
            Transatcion {
                tx_type: tx.r#type,
                amount: tx.amount.clone().unwrap(),
                state: TransactionState::Normal,
            },
        );

        self.account.total -= tx.amount.unwrap();

        Ok(())
    }

    fn process_dispute(&mut self, tx: TransactionRequset) -> Result<(), String> {
        let old_tx = match self.tx_history.get_mut(&tx.tx) {
            None => return Err(format!("Invalid tx id. Ignored.\n{:?}", tx)),
            Some(v) => v,
        };

        if old_tx.state != TransactionState::Normal {
            return Err(format!(
                "Transaction can't be dispputed. Ignored.\n{:?}",
                tx
            ));
        } else {
            old_tx.state = TransactionState::Disputed;
            self.account.held += old_tx.amount;
            Ok(())
        }
    }

    fn process_resolve(&mut self, tx: TransactionRequset) -> Result<(), String> {
        let old_tx = match self.tx_history.get_mut(&tx.tx) {
            None => return Err(format!("Invalid tx id. Ignored.\n{:?}", tx)),
            Some(v) => v,
        };

        if old_tx.state != TransactionState::Disputed {
            return Err(format!("Transaction can't be resolved. Ignored.\n{:?}", tx));
        } else {
            old_tx.state = TransactionState::Normal;
            self.account.held -= old_tx.amount;
            Ok(())
        }
    }

    fn process_chargeback(&mut self, tx: TransactionRequset) -> Result<(), String> {
        let old_tx = match self.tx_history.get_mut(&tx.tx) {
            None => return Err(format!("Invalid tx id. Ignored.\n{:?}", tx)),
            Some(v) => v,
        };

        if old_tx.state != TransactionState::Disputed {
            return Err(format!("Transaction can't be resolved. Ignored.\n{:?}", tx));
        } else {
            old_tx.state = TransactionState::Chargedback;
            self.account.held -= old_tx.amount;
            self.account.total -= old_tx.amount;
            self.frozen = true;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_process_deposit() {
        let mut user = User::default();
        let mut tx = TransactionRequset {
            r#type: TransactionType::Deposit,
            client: 0,
            tx: 1,
            amount: Some(1.23),
        };

        assert!(user.process_deposit(tx.clone()).is_ok());

        assert_eq!(user.account.total, 1.23);
        assert_eq!(user.account.avalible(), 1.23);
        assert_eq!(user.account.held, 0.0);

        // Doubled tx id
        assert!(user.process_deposit(tx.clone()).is_err());

        // Missing amount
        tx.tx = 2;
        tx.amount = None;
        assert!(user.process_deposit(tx).is_err());

        assert_eq!(user.account.total, 1.23);
        assert_eq!(user.account.avalible(), 1.23);
        assert_eq!(user.account.held, 0.0);
    }

    #[test]
    fn test_process_withdrawal() {
        let mut user = User {
            account: Account {
                total: 15.0,
                held: 5.0,
            },
            ..Default::default()
        };
        let mut tx = TransactionRequset {
            r#type: TransactionType::Withdrawal,
            client: 0,
            tx: 1,
            amount: Some(5.0),
        };

        assert!(user.process_withdrawal(tx.clone()).is_ok());

        assert_eq!(user.account.total, 10.0);
        assert_eq!(user.account.avalible(), 5.0);
        assert_eq!(user.account.held, 5.0);

        // Doubled tx id
        assert!(user.process_withdrawal(tx.clone()).is_err());

        // Missing amount
        tx.tx = 2;
        tx.amount = None;
        assert!(user.process_withdrawal(tx.clone()).is_err());

        // Out of avalible funds
        tx.tx = 3;
        tx.amount = Some(7.0);
        assert!(user.process_withdrawal(tx).is_err());

        assert_eq!(user.account.total, 10.0);
        assert_eq!(user.account.avalible(), 5.0);
        assert_eq!(user.account.held, 5.0);
    }

    #[test]
    fn test_process_dispute() {
        let mut user = User::default();
        let mut tx = TransactionRequset {
            r#type: TransactionType::Deposit,
            client: 0,
            tx: 1,
            amount: Some(5.0),
        };

        assert!(user.process_deposit(tx.clone()).is_ok());
        tx.tx = 2;
        assert!(user.process_deposit(tx.clone()).is_ok());

        tx.r#type = TransactionType::Dispute;
        tx.amount = Some(20.22); // Amount should be ignored anyway
        assert_eq!(user.account.total, 10.0);
        assert_eq!(user.account.avalible(), 10.0);
        assert_eq!(user.account.held, 0.0);

        assert!(user.process_dispute(tx.clone()).is_ok());

        assert_eq!(user.account.total, 10.0);
        assert_eq!(user.account.avalible(), 5.0);
        assert_eq!(user.account.held, 5.0);

        // Doubled tx id
        assert!(user.process_dispute(tx).is_err());

        assert_eq!(user.account.total, 10.0);
        assert_eq!(user.account.avalible(), 5.0);
        assert_eq!(user.account.held, 5.0);
    }

    #[test]
    fn test_process_resolve() {
        let mut user = User::default();
        let mut tx = TransactionRequset {
            r#type: TransactionType::Deposit,
            client: 0,
            tx: 1,
            amount: Some(5.0),
        };

        assert!(user.process_deposit(tx.clone()).is_ok());
        tx.tx = 2;
        assert!(user.process_deposit(tx.clone()).is_ok());

        tx.r#type = TransactionType::Dispute;
        assert!(user.process_dispute(tx.clone()).is_ok());
        assert!(user.process_resolve(tx.clone()).is_ok());

        assert_eq!(user.account.total, 10.0);
        assert_eq!(user.account.avalible(), 10.0);
        assert_eq!(user.account.held, 0.0);

        // Not in despute anymore
        assert!(user.process_resolve(tx.clone()).is_err());

        assert_eq!(user.account.total, 10.0);
        assert_eq!(user.account.avalible(), 10.0);
        assert_eq!(user.account.held, 0.0);
    }

    #[test]
    fn test_process_chargeback() {
        let mut user = User::default();
        let mut tx = TransactionRequset {
            r#type: TransactionType::Deposit,
            client: 0,
            tx: 1,
            amount: Some(5.0),
        };

        assert!(user.process_deposit(tx.clone()).is_ok());
        tx.tx = 2;
        assert!(user.process_deposit(tx.clone()).is_ok());

        tx.r#type = TransactionType::Dispute;
        assert!(user.process_dispute(tx.clone()).is_ok());
        assert!(user.process_chargeback(tx.clone()).is_ok());

        assert_eq!(user.account.total, 5.0);
        assert_eq!(user.account.avalible(), 5.0);
        assert_eq!(user.account.held, 0.0);
        assert!(user.frozen);

        // Not in despute anymore
        assert!(user.process_chargeback(tx.clone()).is_err());

        // Account is locked
        tx.r#type = TransactionType::Withdrawal;
        tx.tx = 10;
        assert!(user.process_tx(tx.clone()).is_err());

        assert_eq!(user.account.total, 5.0);
        assert_eq!(user.account.avalible(), 5.0);
        assert_eq!(user.account.held, 0.0);
    }
}
