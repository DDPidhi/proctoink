#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod proctoink {
    use ink::storage::Mapping;
    use scale::{Decode, Encode};
    use scale_info::TypeInfo;

    #[cfg_attr(feature = "std", derive(TypeInfo, ink::storage::traits::StorageLayout))]
    #[derive(Encode, Decode, Default, Clone, Debug, PartialEq, Eq)]
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
            let mut meta = self.exam_metadata.get(user).unwrap_or_default();
            meta.start_time = Some(start_time);
            self.exam_metadata.insert(user, &meta);
        }

        #[ink(message)]
        pub fn add_violation(&mut self, user: AccountId, violation_time: u64) {
            let mut meta = self.exam_metadata.get(user).unwrap_or_default();

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
            let mut meta = self.exam_metadata.get(user).unwrap_or_default();

            // only update if start_time exists and end_time >= start_time
            if let Some(start) = meta.start_time {
                if end_time > start {
                    meta.end_time = Some(end_time);
                    self.exam_metadata.insert(user, &meta);
                }
            }
        }

        #[ink(message)]
        pub fn get_start_time(&self, user: AccountId) -> Option<u64> {
            self.exam_metadata.get(user).and_then(|m| m.start_time)
        }

        #[ink(message)]
        pub fn get_end_time(&self, user: AccountId) -> Option<u64> {
            self.exam_metadata.get(user).and_then(|m| m.end_time)
        }

        #[ink(message)]
        pub fn get_violation_times(&self, user: AccountId) -> [Option<u64>; 3] {
            self.exam_metadata.get(user).map(|m| m.violations).unwrap_or_default()
        }

        #[ink(message)]
        pub fn is_kicked(&self, user: AccountId) -> bool {
            self.exam_metadata.get(user).map(|m| m.kicked).unwrap_or(false)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn test_start_and_end_times() {
            let mut contract = Proctoink::new();
            let user = AccountId::from([0x1; 32]);
            contract.set_start(user, 1000);
            assert_eq!(contract.get_start_time(user), Some(1000));

            contract.set_end(user, 2000);
            assert_eq!(contract.get_end_time(user), Some(2000));
        }

        #[ink::test]
        fn test_add_violations_and_kick() {
            let mut contract = Proctoink::new();
            let user = AccountId::from([0x3; 32]);

            contract.add_violation(user, 1100);
            assert_eq!(contract.is_kicked(user), false);

            contract.add_violation(user, 1200);
            contract.add_violation(user, 1300);
            assert_eq!(contract.is_kicked(user), true);

            let violations = contract.get_violation_times(user);
            assert_eq!(violations, [Some(1100), Some(1200), Some(1300)]);
        }

        #[ink::test]
        fn test_metadata_defaults() {
            let contract = Proctoink::new();
            let user = AccountId::from([0x4; 32]);
            assert_eq!(contract.get_start_time(user), None);
            assert_eq!(contract.get_end_time(user), None);
            assert_eq!(contract.get_violation_times(user), [None, None, None]);
            assert_eq!(contract.is_kicked(user), false);
        }
    }
}
