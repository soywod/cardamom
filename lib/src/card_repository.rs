use crate::{card::Card, error::*};

pub trait CardRepository {
    fn create(&self, card: &mut Card) -> Result<()>;
    fn read(&self, id: &str) -> Result<Card>;
    fn update(&self, card: &mut Card) -> Result<()>;
    fn delete(&self, card: &Card) -> Result<()>;
}
