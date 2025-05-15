#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod proctoink {
    use ink::storage::Mapping;
    use ink::storage::traits::StorageLayout;
    use scale::{Decode, Encode};
    use scale_info::TypeInfo;

    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    #[derive(scale::Encode, scale::Decode, Default, Clone, Debug, PartialEq, Eq)]
    pub struct ExamMetadata {
        pub start_time: Option<u64>,
        pub end_time: Option<u64>,
        pub violations: [Option<u64>; 3],
        pub kicked: bool,
    }

    #[ink(storage)]
    pub struct Proctoink {
        exam_metadata: Mapping<AccountId, ExamMetadata>,
    }

    impl Proctoink {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                exam_metadata: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn set_start(&mut self, user: AccountId, start_time: u64) {
            let mut meta = self.exam_metadata.get(&user).unwrap_or_default();
            meta.start_time = Some(start_time);
            self.exam_metadata.insert(user, &meta);
        }

        #[ink(message)]
        pub fn add_violation(&mut self, user: AccountId, violation_time: u64) {
            let mut meta = self.exam_metadata.get(&user).unwrap_or_default();

            for slot in meta.violations.iter_mut() {
                if slot.is_none() {
                    *slot = Some(violation_time);
                    break;
                }
            }

            if meta.violations.iter().all(|v| v.is_some()) {
                meta.kicked = true;
            }

            self.exam_metadata.insert(user, &meta);
        }

        #[ink(message)]
        pub fn set_end(&mut self, user: AccountId, end_time: u64) {
            let mut meta = self.exam_metadata.get(&user).unwrap_or_default();
            meta.end_time = Some(end_time);
            self.exam_metadata.insert(user, &meta);
        }

        #[ink(message)]
        pub fn get_metadata(&self, user: AccountId) -> Option<ExamMetadata> {
            self.exam_metadata.get(&user)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn it_works() {
            let mut contract = Proctoink::new();
            let user = AccountId::from([0x01; 32]);

            contract.set_start(user, 1000);
            let meta = contract.get_metadata(user).unwrap();
            assert_eq!(meta.start_time, Some(1000));
            assert_eq!(meta.kicked, false);

            contract.add_violation(user, 1100);
            contract.add_violation(user, 1200);
            contract.add_violation(user, 1300);
            let meta = contract.get_metadata(user).unwrap();
            assert_eq!(meta.kicked, true);

            contract.set_end(user, 2000);
            let meta = contract.get_metadata(user).unwrap();
            assert_eq!(meta.end_time, Some(2000));
        }
    }
}
