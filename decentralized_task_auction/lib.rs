#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod decentralized_task_auction {
    use ink_prelude::{string::String, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};

    #[derive(Debug, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Task {
        id: u32,
        description: String,
        client: AccountId,
        arbitrator: AccountId,

        contractor: AccountId,
        bid_price: Balance,

        ask_price: Balance,
        deposit: Balance,

        closing_time: Timestamp,
        extension_time: Timestamp,
    }

    impl Task {}

    #[ink(storage)]
    pub struct DecentralizedTaskAuction {
        service_fee: Balance,
        task_counter: u32,
        tasks: ink_storage::Vec<Task>,
    }

    impl DecentralizedTaskAuction {
        #[ink(constructor)]
        pub fn new(service_fee: Balance) -> Self {
            Self {
                service_fee,
                task_counter: 0,
                tasks: ink_storage::Vec::new(),
            }
        }

        #[ink(message)]
        pub fn create_task(
            &mut self,
            description: String,
            arbitrator: AccountId,
            ask_price: Balance,
            deposit: Balance,
            closing_time: Timestamp,
            extension_time: Timestamp,
        ) {
            // TODO: add input checks and panic
            // compuet service fees based on duration until closing time
            // make sure the transfered balance covers fees, deposit, and ask_price
            self.tasks.push(Task {
                id: self.task_counter,
                description,
                client: self.env().caller(),
                arbitrator,
                contractor: self.env().caller(),
                bid_price: ask_price,
                ask_price,
                deposit,
                closing_time,
                extension_time,
            });
            self.task_counter += 1;
        }

        #[ink(message)]
        pub fn get(&self) -> u32 {
            self.task_counter
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let decentralized_task_auction = DecentralizedTaskAuction::new(0);
            assert_eq!(decentralized_task_auction.get(), 0);
        }
    }
}
