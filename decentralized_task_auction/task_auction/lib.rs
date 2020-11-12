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
    }

    #[ink(event)]
    pub struct Bid {
        #[ink(topic)]
        bid: Balance,
        #[ink(topic)]
        contractor: AccountId,
    }

    impl TaskAuction {
        #[ink(constructor)]
        pub fn new(
            description: String,
            pay_multiplier: u8,
            jury: AccountId,
            duration: Timestamp,
            extension: Timestamp,
        ) -> Self {
            // TODO: add input checks
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
            }
        }

        #[ink(message, payable)]
        pub fn bid(&mut self) {
            // verify bid
            assert!(Self::env().block_timestamp() <= self.deadline);
            assert!(Self::env().transferred_balance() * 1000 < self.current_bid * 995);
            // refund previous bidder
            Self::env().transfer(self.contractor, self.current_bid);
            // update lowest bid
            self.contractor = Self::env().caller();
            self.current_bid = Self::env().transferred_balance();
            // extend deadline
            self.deadline = Timestamp::max(
                self.deadline,
                Self::env().block_timestamp() + self.extension,
            );
            // notify subscribers
            Self::env().emit_event(Bid {
                bid: self.current_bid,
                contractor: self.contractor,
            });
        }

        // TODO: add tests
        #[ink(message)]
        pub fn cancel(&mut self) {
            let caller = Self::env().caller();
            if caller == self.client {
                // client cancelled, refund contractor and terminate auction
                let refund = if Self::env().block_timestamp() > self.deadline {
                    // full payment if past deadline
                    self.current_bid * Balance::from(self.pay_multiplier)
                } else {
                    self.current_bid
                };
                Self::env().terminate_contract(
                    match Self::env().transfer(self.contractor, refund) {
                        Ok(_) => self.client,
                        Err(_) => self.contractor,
                    },
                );
            } else if caller == self.contractor {
                // contractor cancelled, reset winning bid
                self.contractor = self.client;
                // refund contractor if pre deadline
                if Self::env().block_timestamp() <= self.deadline {
                    if let Err(_) = Self::env().transfer(caller, self.current_bid) {
                        Self::env().terminate_contract(caller);
                    }
                }
                self.current_bid = Self::env().balance() / Balance::from(self.pay_multiplier + 1);
            }
        }

        #[ink(message, payable)]
        pub fn test_func(&self) {
            // println!("{:?}", Self::env().block_timestamp());
            // println!("rent {:?}", Self::env().rent_allowance());
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
