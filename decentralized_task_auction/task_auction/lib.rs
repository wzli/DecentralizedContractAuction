#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod task_auction {
    use ink_prelude::string::String;

    #[ink(storage)]
    pub struct TaskAuction {
        description: String,
        pay_multiplier: u8,
        current_bid: Balance,
        contractor: AccountId,
        client: AccountId,
        jury: AccountId,
        deadline: Timestamp,
        extension: Timestamp,

        contractor_confirmation: Option<bool>,
        client_confirmation: Option<bool>,
    }

    #[ink(event)]
    pub struct Bid {
        #[ink(topic)]
        bid: Balance,
        #[ink(topic)]
        contractor: AccountId,
    }

    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Source {
        Contractor,
        Client,
        Jury,
    }

    #[ink(event)]
    pub struct Confirm {
        #[ink(topic)]
        source: Source,
        #[ink(topic)]
        confirmation: bool,
    }

    #[ink(event)]
    pub struct Dispute;

    impl TaskAuction {
        #[ink(constructor)]
        pub fn new(
            description: String,
            pay_multiplier: u8,
            jury: AccountId,
            duration: Timestamp,
            extension: Timestamp,
        ) -> Self {
            let client = Self::env().caller();
            Self {
                description,
                pay_multiplier,
                current_bid: Self::env().transferred_balance() / Balance::from(pay_multiplier + 1),
                contractor: client,
                client,
                jury,
                deadline: Self::env().block_timestamp() + duration,
                extension,
                contractor_confirmation: None,
                client_confirmation: None,
            }
        }

        // TODO: add tests
        #[ink(message, payable)]
        pub fn bid(&mut self) {
            // only allow bids before deadline
            assert!(Self::env().block_timestamp() <= self.deadline);
            // bid must be within %50 - %99 of previous bid
            assert!(Self::env().transferred_balance() * 2 > self.current_bid);
            assert!(Self::env().transferred_balance() * 100 < self.current_bid * 99);
            // disallow bids from jury or previous bidder (to discourage spam)
            assert_ne!(Self::env().caller(), self.jury);
            assert_ne!(Self::env().caller(), self.contractor);
            // refund previous bidder and update current bid
            Self::transfer_or_terminate(self.current_bid, self.contractor);
            self.update_bid(Self::env().transferred_balance(), Self::env().caller());
        }

        // TODO: add tests
        #[ink(message)]
        pub fn cancel(&mut self) {
            if Self::env().caller() == self.contractor {
                // contractor cancelled
                if Self::env().block_timestamp() <= self.deadline {
                    // refund contractor if pre deadline
                    Self::transfer_or_terminate(self.current_bid, self.contractor);
                }
                // reset bid
                self.update_bid(
                    Self::env().balance() / Balance::from(self.pay_multiplier + 1),
                    self.client,
                );
            } else if Self::env().caller() == self.client {
                // client cancelled, refund contractor and terminate auction
                let refund = if Self::env().block_timestamp() <= self.deadline {
                    self.current_bid
                } else {
                    // full payment if past deadline
                    self.current_bid * Balance::from(self.pay_multiplier)
                };
                Self::transfer_or_terminate(refund, self.contractor);
                Self::env().terminate_contract(self.client);
            }
        }

        #[ink(message)]
        pub fn confirm(&mut self, confirmation: bool) {
            assert!(Self::env().block_timestamp() > self.deadline);
            // parse confirmation
            let source = if Self::env().caller() == self.contractor {
                self.contractor_confirmation = Some(confirmation);
                Source::Contractor
            } else if Self::env().caller() == self.client {
                self.client_confirmation = Some(confirmation);
                Source::Client
            } else if Self::env().caller() == self.jury {
                match (self.contractor_confirmation, self.client_confirmation) {
                    (Some(true), Some(false)) => Source::Jury,
                    _ => return,
                }
            } else {
                return;
            };
            // notify subscribers
            Self::env().emit_event(Confirm {
                confirmation,
                source,
            });
            // check if termination conditions are satisfied
            if let Some(true) = self.contractor_confirmation {
                if let Some(true) = self.client_confirmation {
                    // mutually confirmed, pay contractor and terminate
                    Self::transfer_or_terminate(
                        self.current_bid * Balance::from(self.pay_multiplier),
                        self.contractor,
                    );
                    Self::env().terminate_contract(self.client);
                } else if Self::env().caller() == self.jury {
                    // let jury resolve dispute (for pay)
                    Self::transfer_or_terminate(self.current_bid, self.jury);
                    if confirmation {
                        // pay contractor if task deemed to be fulfilled
                        Self::transfer_or_terminate(
                            self.current_bid * Balance::from(self.pay_multiplier),
                            self.contractor,
                        );
                    }
                    Self::env().terminate_contract(self.client);
                } else {
                    // dispute triggered
                    Self::env().emit_event(Dispute {});
                }
            }
        }

        #[ink(message, payable)]
        pub fn test_func(&self) {
            // println!("{:?}", Self::env().block_timestamp());
            // println!("rent {:?}", Self::env().rent_allowance());
        }

        // helper functions

        fn update_bid(&mut self, bid: Balance, contractor: AccountId) {
            self.current_bid = bid;
            self.contractor = contractor;
            self.contractor_confirmation = None;
            self.deadline = Timestamp::max(
                self.deadline,
                Self::env().block_timestamp() + self.extension,
            );
            Self::env().emit_event(Bid { bid, contractor });
        }

        fn transfer_or_terminate(balance: Balance, account: AccountId) {
            if let Err(_) = Self::env().transfer(account, balance) {
                Self::env().terminate_contract(account);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        fn set_balance(balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                    .unwrap(),
                balance,
            )
            .expect("Cannot set account balance");
        }

        fn set_sender(sender: AccountId, pay: Balance) {
            let callee =
                ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                    .unwrap();
            let data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // dummy
            ink_env::test::push_execution_context::<Environment>(
                sender, callee, 1000000, pay, data,
            );
        }

        #[ink::test]
        fn it_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
            println!(
                "a: {:?}",
                ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.alice)?
            );
            //println!("b: {:?}", ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.bob)?);
            //println!("c: {:?}", ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.charlie)?);

            set_balance(100);
            let task_auction = TaskAuction::new("test desc".into(), 2, accounts.bob, 10, 4);
            ink_env::test::advance_block::<ink_env::DefaultEnvironment>()?;
            task_auction.test_func();
            set_sender(accounts.bob, 10000);
            ink_env::test::advance_block::<ink_env::DefaultEnvironment>()?;
            task_auction.test_func();
            task_auction.test_func();
            println!(
                "a: {:?}",
                ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.alice)?
            );
        }

        #[ink::test]
        #[should_panic]
        fn pay_multiplier_overflow() {
            TaskAuction::new("test desc".into(), 255, AccountId::from([1; 32]), 0, 0);
        }
    }
}
