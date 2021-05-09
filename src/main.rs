use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize};
use std::env;
use std::io;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize)]
enum TransactionType {
    #[serde(rename  = "deposit")]
    Deposit,
    #[serde(rename  = "withdrawal")]
    Withdrawal,
    #[serde(rename  = "dispute")]
    Dispute,
    #[serde(rename  = "resolve")]
    Resolve,
    #[serde(rename  = "chargeback")]
    Chargeback,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
struct TransactionRequset {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

#[derive(Eq, PartialEq)]
enum TransactionState {
    Normal,
    Disputed,
    Chargedback,
}
struct Transatcion {
    pub tx_type: TransactionType,
    pub amount: f64,
    pub state: TransactionState,
}

#[derive(Default)]
struct Account {
    pub total: f64,
    pub held: f64,
}

impl Account {
    pub fn avalible(&self) -> f64 {
        self.total - self.held
    }
}

#[derive(Default)]
struct User {
    pub id: u16,
    pub account: Account,
    pub tx_history: HashMap<u32,Transatcion>,
    pub frozen: bool,
}

impl User {
    pub fn process_tx(&mut self, tx : TransactionRequset) -> Result<(),String> {

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

    fn process_deposit(&mut self, tx: TransactionRequset) -> Result<(),String> {
        if self.tx_history.contains_key(&tx.tx) {
            return Err(format!("Doubled transaction id. Ignored.\n{:?}",tx));
        }

        if tx.amount.is_none() || *tx.amount.as_ref().unwrap() == 0.0 {
            return Err(format!("Invalid transaction data. Ignored.\n{:?}",tx));
        }

        self.tx_history.insert(tx.tx, Transatcion{
            tx_type: tx.r#type,
            amount: tx.amount.clone().unwrap(),
            state: TransactionState::Normal,
        });

        self.account.total += tx.amount.unwrap();

        Ok(())
    }

    fn process_withdrawal(&mut self, tx: TransactionRequset) -> Result<(),String> {
        if self.tx_history.contains_key(&tx.tx) {
            return Err(format!("Doubled transaction id. Ignored.\n{:?}",tx));
        }

        let amount = if tx.amount.is_none() || *tx.amount.as_ref().unwrap() == 0.0 {
            return Err(format!("Invalid transaction data. Ignored.\n{:?}",tx));
        } else {
            tx.amount.unwrap()
        };

        if amount > self.account.avalible() {
            return Err("Insufficient funds.".to_string());
        }

        self.tx_history.insert(tx.tx, Transatcion{
            tx_type: tx.r#type,
            amount: tx.amount.clone().unwrap(),
            state: TransactionState::Normal,
        });

        self.account.total -= tx.amount.unwrap();

        Ok(())
    }

    fn process_dispute(&mut self, tx: TransactionRequset) -> Result<(),String> {
        let old_tx = match self.tx_history.get_mut(&tx.tx) {
            None => return Err(format!("Invalid tx id. Ignored.\n{:?}",tx)),
            Some(v) => v,
        };

        if old_tx.state != TransactionState::Normal {
            return Err(format!("Transaction can't be dispputed. Ignored.\n{:?}",tx))
        } else {
            old_tx.state = TransactionState::Disputed;
            self.account.held += old_tx.amount;
            Ok(())
        }
    }

    fn process_resolve(&mut self, tx: TransactionRequset) -> Result<(),String> {
        let old_tx = match self.tx_history.get_mut(&tx.tx) {
            None => return Err(format!("Invalid tx id. Ignored.\n{:?}",tx)),
            Some(v) => v,
        };

        if old_tx.state != TransactionState::Disputed {
            return Err(format!("Transaction can't be resolved. Ignored.\n{:?}",tx))
        } else {
            old_tx.state = TransactionState::Normal;
            self.account.held -= old_tx.amount;
            Ok(())
        }
    }

    fn process_chargeback(&mut self, tx: TransactionRequset) -> Result<(),String> {
        let old_tx = match self.tx_history.get_mut(&tx.tx) {
            None => return Err(format!("Invalid tx id. Ignored.\n{:?}",tx)),
            Some(v) => v,
        };

        if old_tx.state != TransactionState::Disputed {
            return Err(format!("Transaction can't be resolved. Ignored.\n{:?}",tx))
        } else {
            old_tx.state = TransactionState::Chargedback;
            self.account.held -= old_tx.amount;
            self.account.total -= old_tx.amount;
            self.frozen = true;
            Ok(())
        }
    }
}

struct Engine {
    pub users: HashMap<u16,User>,
}

impl Engine {
    pub fn process_tx(&mut self, tx: TransactionRequset) -> Result<(),String> {
        let user = match self.users.get_mut(&tx.client) {
            Some(v) => v,
            None => {
                self.users.insert(tx.client, User {
                    id: tx.client,
                    account: Account {
                        total: 0.0,
                        held: 0.0,
                    },
                    tx_history: HashMap::new(),
                    frozen: false,
                });
                self.users.get_mut(&tx.client).unwrap()
            }
        };

        user.process_tx(tx)
    }

    pub fn process_data(&mut self, file_name: String) -> Result<(),io::Error> {
        let path_buff = PathBuf::from(file_name);
        println!("{:?}",path_buff.as_path().as_os_str());
        let mut rdr = csv::ReaderBuilder::new().trim(csv::Trim::All).from_path(&path_buff)?;
        for record in rdr.records() {
            match record.unwrap().deserialize::<TransactionRequset>(None) {
                Err(e) => {
                    eprintln!("Request parse error: {:?}", e);
                }
                Ok(request) => {
                    if let Err(e) = self.process_tx(request) {
                        eprintln!("{}",e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn print_users(&self) {
        println!("client,available,held,total,locked");
        for user in self.users.values() {
            println!("{}\t{:.4}\t{:.4}\t{:.4}\t{}",user.id,user.account.avalible(),user.account.held,user.account.total,user.frozen);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }

    println!("{:?}", &args);

    let mut engine = Engine { 
        users: HashMap::new(),
    };
    let _ = engine.process_data(args[1].clone());
    engine.print_users();
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_process_deposit() {
    //     let mut user = User::default();
    //     let tx = TransactionRequset {
    //         r#type: TransactionType::Deposit,
    //         client: 0,
    //         tx: 1,
    //         amount: Some(1.23),
    //     };

    //     let res = user.process_deposit(tx).unwrap();

    //     assert!(user.)
    // }
}