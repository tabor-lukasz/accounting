use std::collections::HashMap;
use std::io;

use crate::user::*;

pub struct Engine {
    pub users: HashMap<u16, User>,
}

impl Engine {
    /// Process single transaction
    pub fn process_tx(&mut self, tx: TransactionRequset) -> Result<(), String> {
        let user = match self.users.get_mut(&tx.client) {
            Some(v) => v,
            None => {
                self.users.insert(
                    tx.client,
                    User {
                        id: tx.client,
                        account: Account {
                            total: 0.0,
                            held: 0.0,
                        },
                        tx_history: HashMap::new(),
                        frozen: false,
                    },
                );
                self.users.get_mut(&tx.client).unwrap()
            }
        };

        user.process_tx(tx)
    }

    /// Processes file with pending transactions
    pub fn process_data(&mut self, path: &std::path::Path) -> Result<(), io::Error> {
        let mut rdr = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_path(path)?;
        for record in rdr.records() {
            match record.unwrap().deserialize::<TransactionRequset>(None) {
                Err(e) => {
                    eprintln!("Request parse error: {:?}", e);
                }
                Ok(request) => {
                    if let Err(e) = self.process_tx(request) {
                        eprintln!("{}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Prints all users data.
    pub fn print_users(&self) {
        println!("client,available,held,total,locked");
        for user in self.users.values() {
            println!(
                "{}\t{:.4}\t{:.4}\t{:.4}\t{}",
                user.id,
                user.account.avalible(),
                user.account.held,
                user.account.total,
                user.frozen
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_process_tx() {
        let mut engine = Engine {
            users: HashMap::new(),
        };

        let mut tx = TransactionRequset {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(10.0),
        };

        assert!(engine.process_tx(tx.clone()).is_ok());
        tx.tx = 2;
        assert!(engine.process_tx(tx.clone()).is_ok());
        tx.client = 2;
        tx.tx = 3;
        assert!(engine.process_tx(tx.clone()).is_ok());
        assert_eq!(engine.users.len(), 2);

        tx.client = 1;
        tx.tx = 4;
        tx.r#type = TransactionType::Withdrawal;
        tx.amount = Some(5.0);
        assert!(engine.process_tx(tx.clone()).is_ok());
        assert_eq!(engine.users.get(&1).unwrap().account.total, 15.0);
        assert_eq!(engine.users.get(&1).unwrap().account.avalible(), 15.0);

        // Not in despute
        tx.tx = 1;
        tx.r#type = TransactionType::Resolve;
        assert!(engine.process_tx(tx.clone()).is_err());

        tx.r#type = TransactionType::Dispute;
        assert!(engine.process_tx(tx.clone()).is_ok());
        assert_eq!(engine.users.get(&1).unwrap().account.total, 15.0);
        assert_eq!(engine.users.get(&1).unwrap().account.avalible(), 5.0);

        tx.r#type = TransactionType::Resolve;
        assert!(engine.process_tx(tx.clone()).is_ok());
        assert_eq!(engine.users.get(&1).unwrap().account.total, 15.0);
        assert_eq!(engine.users.get(&1).unwrap().account.avalible(), 15.0);

        tx.r#type = TransactionType::Dispute;
        assert!(engine.process_tx(tx.clone()).is_ok());
        assert_eq!(engine.users.get(&1).unwrap().account.total, 15.0);
        assert_eq!(engine.users.get(&1).unwrap().account.avalible(), 5.0);

        tx.r#type = TransactionType::Chargeback;
        assert!(engine.process_tx(tx.clone()).is_ok());
        assert_eq!(engine.users.get(&1).unwrap().account.total, 5.0);
        assert_eq!(engine.users.get(&1).unwrap().account.avalible(), 5.0);

        // User locked
        tx.r#type = TransactionType::Deposit;
        assert!(engine.process_tx(tx.clone()).is_err());
    }

    #[test]
    fn test_process_data() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("input.csv");
        let mut file = std::fs::File::create(&path).unwrap();

        let data = r#"type,client,tx,amount
        deposit,    1,      1,  1.0
        deposit,    2,      2,  2.0
        deposit,    1,      3,  2.0
        withdrawal, 1,      4,  1.5
        withdrawal, 2,      5,  3.0"#;
        write!(file, "{}", data).unwrap();

        let mut engine = Engine {
            users: HashMap::new(),
        };

        assert!(engine.process_data(&path).is_ok());
        assert_eq!(engine.users.get(&1).unwrap().account.avalible(), 1.5);
        assert_eq!(engine.users.get(&1).unwrap().account.held, 0.0);
        assert_eq!(engine.users.get(&1).unwrap().account.total, 1.5);

        assert_eq!(engine.users.get(&2).unwrap().account.avalible(), 2.0);
        assert_eq!(engine.users.get(&2).unwrap().account.held, 0.0);
        assert_eq!(engine.users.get(&2).unwrap().account.total, 2.0);
    }
}
