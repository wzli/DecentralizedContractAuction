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

    #[ink(event)]
    pub struct Confirm {
        #[ink(topic)]
        confirmation: bool,
        #[ink(topic)]
        source: AccountId,
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
            assert!(self.is_open());
            // bid must be within %50 - %99 of previous bid
            assert!(Self::env().transferred_balance() * 2 > self.current_bid);
            assert!(Self::env().transferred_balance() * 100 < self.current_bid * 99);
            // disallow bids from jury or previous bidder (to discourage spam)
            let caller = Self::env().caller();
            assert_ne!(caller, self.jury);
            assert_ne!(caller, self.contractor);
            // refund previous bidder and update current bid
            Self::transfer_or_terminate(self.current_bid, self.contractor);
            self.update_bid(Self::env().transferred_balance(), caller);
        }

        // TODO: add tests
        #[ink(message)]
        pub fn cancel(&mut self) {
            if Self::env().caller() == self.contractor {
                // contractor cancelled
                if self.is_open() {
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
                let refund = if self.is_open() {
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
            assert!(!self.is_open());
            let source = Self::env().caller();
            // parse confirmation
            if source == self.client {
                self.client_confirmation = Some(confirmation);
                Self::env().emit_event(Confirm {
                    confirmation,
                    source,
                });
            }
            if source == self.contractor {
                self.contractor_confirmation = Some(confirmation);
                Self::env().emit_event(Confirm {
                    confirmation,
                    source,
                });
            }
            // check if termination conditions are satisfied
            if let Some(true) = self.contractor_confirmation {
                if let Some(true) = self.client_confirmation {
                    // mutually confirmed, pay contractor and terminate
                    Self::transfer_or_terminate(
                        self.current_bid * Balance::from(self.pay_multiplier),
                        self.contractor,
                    );
                    Self::env().terminate_contract(self.client);
                } else if source == self.jury {
                    // let jury resolve dispute (for pay)
                    Self::env().emit_event(Confirm {
                        confirmation,
                        source,
                    });
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

        #[ink(message)]
        pub fn extend_deadline(&mut self, extension: Timestamp) -> Timestamp {
            assert!(self.is_open());
            assert_eq!(Self::env().caller(), self.client);
            self.deadline += extension;
            self.deadline
        }

        #[ink(message)]
        pub fn is_open(&self) -> bool {
            Self::env().block_timestamp() < self.deadline
        }

        #[ink(message)]
        pub fn get_blocktime(&self) -> Timestamp {
            Self::env().block_timestamp()
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
        use ink_env::{call, test};
        use ink_lang as ink;

        const BLOCK_DURATION: Timestamp = 5;

        #[ink::test]
        #[should_panic]
        fn pay_multiplier_overflow() {
            TaskAuction::new("test desc".into(), 255, AccountId::from([1; 32]), 0, 0);
        }

        #[ink::test]
        fn no_bidders() {
            let mut task_auction = new_task_auction(1000, 1, BLOCK_DURATION, 0);
            assert!(task_auction.is_open());
            advance_block();
            assert!(!task_auction.is_open());
        }

        #[ink::test]
        fn bid_extension() {}

        #[ink::test]
        fn bid_closed() {}

        // helper functions

        fn new_task_auction(
            endowment: Balance,
            pay_multiplier: u8,
            duration: Timestamp,
            extension: Timestamp,
        ) -> TaskAuction {
            // given
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), endowment);
            TaskAuction::new(
                "task descripton".into(),
                pay_multiplier,
                accounts.bob,
                duration,
                extension,
            )

            /*
                        description: String,
                        pay_multiplier: u8,
                        jury: AccountId,
                        duration: Timestamp,
                        extension: Timestamp,
            */

            /*
            // when
            call_payable(
                10,
                contract_id(),
                accounts.eve,
                ink_env::call::Selector::new([0xCA, 0xFE, 0xBA, 0xBE]),
                || {
                    give_me.accumulate_value();
                    ()
                },
            );
            */
        }

        fn advance_block() {
            ink_env::test::advance_block::<ink_env::DefaultEnvironment>()
                .expect("Cannot advance block");
        }

        fn contract_id() -> AccountId {
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id")
        }

        fn set_sender(sender: AccountId) {
            let callee =
                ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            test::push_execution_context::<Environment>(
                sender,
                callee,
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }

        fn default_accounts() -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Off-chain environment should have been initialized already")
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(account_id, balance)
                .expect("Cannot set account balance");
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot set account balance")
        }

        /// Calls a payable message, increases the contract balance before
        /// invoking `f`.
        fn call_payable<F>(
            amount: Balance,
            contract_id: AccountId,
            from: AccountId,
            selector: ink_env::call::Selector,
            f: F,
        ) where
            F: FnOnce() -> (),
        {
            set_sender(from);

            let mut data = ink_env::test::CallData::new(selector);
            data.push_arg(&from);

            // Push the new execution context which sets `from` as caller and
            // the `amount` as the value which the contract  will see as transferred
            // to it.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                from,
                contract_id,
                1000000,
                amount,
                data,
            );

            set_balance(contract_id, get_balance(contract_id) + amount);
            f();
        }
    }
}
