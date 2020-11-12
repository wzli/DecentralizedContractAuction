#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod decentralized_task_auction {
    use ink_prelude::string::String;
    use ink_storage::collections::Stash;
    use ink_storage::traits::{PackedLayout, SpreadLayout};

    #[derive(Debug, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct TaskAuction {
        task_id: u32,
        description: String,
        closing_time: Timestamp,
        extension_time: Timestamp,
        deposit: Balance,
        ask_price: Balance,
        bid_price: Balance,
        contractor: AccountId,
        client: AccountId,
        jury: AccountId,
    }

    #[ink(event)]
    pub struct CreateEvent {
        #[ink(topic)]
        key: u32,
        #[ink(topic)]
        id: u32,
    }

    impl TaskAuction {}

    #[ink(storage)]
    pub struct DecentralizedTaskAuction {
        service_fee: Balance,
        task_counter: u32,
        tasks: Stash<TaskAuction>,
    }

    impl DecentralizedTaskAuction {
        #[ink(constructor)]
        pub fn new(service_fee: Balance) -> Self {
            Self {
                service_fee,
                task_counter: 0,
                tasks: Stash::new(),
            }
        }

        #[ink(message)]
        pub fn create_task_auction(
            &mut self,
            description: String,
            jury: AccountId,
            ask_price: Balance,
            deposit: Balance,
            closing_time: Timestamp,
            extension_time: Timestamp,
        ) -> (u32, u32) {
            // TODO: add input checks and panic
            // compuet service fees based on duration until closing time
            // make sure the transfered balance covers fees, deposit, and ask_price
            let task_id = self.task_counter;
            let task_key = self.tasks.put(TaskAuction {
                task_id,
                description,
                client: self.env().caller(),
                jury,
                contractor: self.env().caller(),
                bid_price: ask_price,
                ask_price,
                deposit,
                closing_time,
                extension_time,
            });
            self.task_counter += 1;
            (task_id, task_key)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const DEFAULT_CALLEE_HASH: [u8; 32] = [0x07; 32];
        const DEFAULT_ENDOWMENT: Balance = 1_000_000;
        const DEFAULT_GAS_LIMIT: Balance = 1_000_000;

        fn set_next_caller(caller: AccountId) {
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                caller,
                AccountId::from(DEFAULT_CALLEE_HASH),
                DEFAULT_ENDOWMENT,
                DEFAULT_GAS_LIMIT,
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])),
            )
        }

        #[test]
        fn it_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
            let mut dca = DecentralizedTaskAuction::new(0);
            set_next_caller(accounts.alice);
            let id = dca.create_task_auction("test desc".into(), accounts.charlie, 0, 0, 0, 0);
            println!("{:?}", id);

            use ink_storage::collections::Stash;

            let mut stash = Stash::new();
            for i in 11..20 {
                println!("{}, ", stash.put(i));
            }
            for i in 0..5 {
                println!("{}, ", stash.take(i).unwrap());
            }
            for i in 0..10 {
                println!("{}, ", stash.put(i));
            }
        }
    }
}
