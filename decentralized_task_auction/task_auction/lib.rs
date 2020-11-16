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

        contractor_confirm: Option<bool>,
        client_confirm: Option<bool>,
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
        value: bool,
        #[ink(topic)]
        source: AccountId,
    }

    #[ink(event)]
    pub struct Dispute {}

    #[ink(event)]
    pub struct Extend {
        #[ink(topic)]
        deadline: Timestamp,
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
            Self {
                description,
                pay_multiplier,
                current_bid: Self::env().balance() / Balance::from(pay_multiplier + 1),
                contractor: Self::env().account_id(),
                client: Self::env().caller(),
                jury,
                deadline: Self::env().block_timestamp() + duration,
                extension,
                contractor_confirm: None,
                client_confirm: None,
            }
        }

        #[ink(message, payable, selector = "0xCAFEBABE")]
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
        pub fn confirm(&mut self, value: bool) {
            assert!(!self.is_open());
            let source = Self::env().caller();
            // parse confirmation
            if source == self.client {
                self.client_confirm = Some(value);
                Self::env().emit_event(Confirm { value, source });
                // represent no bidder case as well
                if self.contractor == Self::env().account_id() {
                    self.contractor_confirm = Some(value);
                }
            } else if source == self.contractor {
                self.contractor_confirm = Some(value);
                Self::env().emit_event(Confirm { value, source });
            }
            // check if termination conditions are satisfied
            if let Some(true) = self.contractor_confirm {
                if let Some(true) = self.client_confirm {
                    // mutually confirmed, pay contractor and terminate
                    Self::transfer_or_terminate(
                        self.current_bid * Balance::from(self.pay_multiplier),
                        self.contractor,
                    );
                    Self::env().terminate_contract(self.client);
                } else if source == self.jury {
                    // let jury resolve dispute (for pay)
                    Self::env().emit_event(Confirm { value, source });
                    Self::transfer_or_terminate(self.current_bid, self.jury);
                    if value {
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
            assert_eq!(Self::env().caller(), self.client);
            assert!(self.is_open() || self.contractor == Self::env().account_id());
            self.deadline += extension;
            Self::env().emit_event(Extend {
                deadline: self.deadline,
            });
            self.deadline
        }

        #[ink(message)]
        pub fn is_open(&self) -> bool {
            Self::env().block_timestamp() < self.deadline
        }

        #[ink(message)]
        pub fn get_current_bid(&self) -> Balance {
            self.current_bid
        }

        // helper functions

        fn update_bid(&mut self, bid: Balance, contractor: AccountId) {
            self.current_bid = bid;
            self.contractor = contractor;
            self.contractor_confirm = None;
            Self::env().emit_event(Bid { bid, contractor });
            let deadline = Self::env().block_timestamp() + self.extension;
            if deadline > self.deadline {
                self.deadline = deadline;
                Self::env().emit_event(Extend { deadline });
            }
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
        #[should_panic(expected = "attempt to add with overflow")]
        fn pay_multiplier_overflow() {
            TaskAuction::new("test desc".into(), 255, AccountId::from([1; 32]), 0, 0);
        }

        #[ink::test]
        #[should_panic(
            expected = "`(left != right)`\n  left: `AccountId([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2])`,\n right: `AccountId([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2])`"
        )]
        fn bid_jury_reject() {
            let mut task_auction = new_task_auction(100, 1, BLOCK_DURATION, 0);
            assert_eq!(task_auction.get_current_bid(), 50);
            let accounts = default_accounts();
            call_payable(49, accounts.bob, [0xCA, 0xFE, 0xBA, 0xBE], || {
                task_auction.bid();
                ()
            });
        }

        #[ink::test]
        #[should_panic(
            expected = "Self::env().transferred_balance() * 100 < self.current_bid * 99"
        )]
        fn bid_below_increment() {
            let mut task_auction = new_task_auction(100, 1, BLOCK_DURATION, 0);
            assert_eq!(task_auction.get_current_bid(), 50);
            let accounts = default_accounts();
            call_payable(50, accounts.charlie, [0xCA, 0xFE, 0xBA, 0xBE], || {
                task_auction.bid();
                ()
            });
        }

        #[ink::test]
        #[should_panic(expected = "Self::env().transferred_balance() * 2 > self.current_bid")]
        fn bid_devalue_reject() {
            let mut task_auction = new_task_auction(100, 1, BLOCK_DURATION, 0);
            assert_eq!(task_auction.get_current_bid(), 50);
            let accounts = default_accounts();
            call_payable(10, accounts.charlie, [0xCA, 0xFE, 0xBA, 0xBE], || {
                task_auction.bid();
                ()
            });
        }

        #[ink::test]
        #[should_panic(expected = "self.is_open()")]
        fn bid_closed() {
            let mut task_auction = new_task_auction(1000, 1, BLOCK_DURATION, 0);
            advance_block();
            advance_block();
            let accounts = default_accounts();
            set_sender(accounts.bob);
            task_auction.bid();
        }

        #[ink::test]
        fn bid_rally() {}

        #[ink::test]
        fn bid_extension() {}

        #[ink::test]
        fn no_bidders() {
            // create auction
            let endowment = 1000;
            let mut task_auction = new_task_auction(endowment, 1, BLOCK_DURATION, 0);
            // check that auction is closed before confirm
            assert!(task_auction.is_open());
            advance_block();
            assert!(!task_auction.is_open());
            // non-client are ignored
            let accounts = default_accounts();
            set_sender(accounts.bob);
            task_auction.confirm(true);
            task_auction.confirm(false);
            // if client terminate only if true
            set_sender(accounts.alice);
            task_auction.confirm(false);
            let alice_balance = get_balance(accounts.alice);
            // compensate for extra pay accumulation, due contract self transfer broken in off-chain tests
            set_balance(contract_id(), endowment / 2);
            // check that contract terminated after confirmation
            ink_env::test::assert_contract_termination::<ink_env::DefaultEnvironment, _>(
                move || task_auction.confirm(true),
                accounts.alice,
                endowment,
            );
            // ensure that original owner received full funds
            assert_eq!(alice_balance + endowment, get_balance(accounts.alice));
            // one event for each confirm
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(2, emitted_events.len());
        }

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
        fn call_payable<F>(amount: Balance, from: AccountId, selector: [u8; 4], f: F)
        where
            F: FnOnce() -> (),
        {
            let contract_id = contract_id();
            set_sender(from);

            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new(selector));
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
