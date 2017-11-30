
// TODO Gems and Keys

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Item {
    Gem,
    PowerGem,
    HealingGem,
    Key(usize),
}
