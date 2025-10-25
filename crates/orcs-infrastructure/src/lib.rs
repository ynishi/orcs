pub mod dto;
// pub mod migration;  // Removed: migrated to version-migrate
pub mod repository;
pub mod toml_storage;
pub mod user_service;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
