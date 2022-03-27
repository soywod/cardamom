use std::collections::{HashMap, HashSet};

use crate::card::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HunkKind {
    PrevLeft(String),
    NextLeft(String),
    PrevRight(String),
    NextRight(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hunk {
    Add(Card),
    Set(Card),
    Del(Card),
}

impl Hunk {
    pub fn card(&self) -> &Card {
        match self {
            Self::Add(card) => card,
            Self::Set(card) => card,
            Self::Del(card) => card,
        }
    }
}

#[derive(Debug, Default)]
pub struct Patch {
    hunks: HashMap<HunkKind, Hunk>,
}

impl Patch {
    pub fn insert(&mut self, kind: HunkKind, next_hunk: Hunk) {
        if let Some(prev_hunk) = self.hunks.get_mut(&kind) {
            if next_hunk.card().date > prev_hunk.card().date {
                *prev_hunk = next_hunk
            }
        } else {
            self.hunks.insert(kind, next_hunk);
        }
    }
}

impl Patch {
    pub fn new(left: impl Cards, right: impl Cards) -> Self {
        let mut ids = HashSet::new();
        let mut patch = Patch::default();

        // gather all existing ids found in all cards maps
        ids.extend(left.prev().iter().map(|(id, _)| id.as_str()));
        ids.extend(left.next().iter().map(|(id, _)| id.as_str()));
        ids.extend(right.prev().iter().map(|(id, _)| id.as_str()));
        ids.extend(right.next().iter().map(|(id, _)| id.as_str()));

        // given the matrice left.prev × left.next × right.prev × right.next,
        // check every 2⁴ = 16 possibilities:
        for id in ids {
            let lp = left.prev().contains_key(id);
            let ln = left.next().contains_key(id);
            let rp = right.prev().contains_key(id);
            let rn = right.next().contains_key(id);

            // 0 (0000): id nowhere
            if !lp && !ln && !rp && !rn {
                // nothing to do
            }

            // 1 (0001): id only in next right cards, which means the
            // card was added right
            if !lp && !ln && !rp && rn {
                let card = right.next().get(id).unwrap();

                patch.insert(
                    HunkKind::PrevLeft(id.to_owned()),
                    Hunk::Add(card.to_owned()),
                );
                patch.insert(
                    HunkKind::NextLeft(id.to_owned()),
                    Hunk::Add(card.to_owned()),
                );
                patch.insert(
                    HunkKind::PrevRight(id.to_owned()),
                    Hunk::Add(card.to_owned()),
                );
            }

            // 2 (0010): id only in prev right cards, which means prev
            // right card is obsolete
            if !lp && !ln && rp && !rn {
                let card = right.prev().get(id).unwrap();

                patch.insert(
                    HunkKind::PrevRight(id.to_owned()),
                    Hunk::Del(card.to_owned()),
                );
            }

            // 3 (0011): id in right cards, which means the previous
            // synchro failed and left is not in phase with right
            // anymore
            if !lp && !ln && rp && rn {
                let prev_card = right.prev().get(id).unwrap();
                let next_card = right.next().get(id).unwrap();

                if next_card.date >= prev_card.date {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Add(next_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextLeft(id.to_owned()),
                        Hunk::Add(next_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Set(next_card.to_owned()),
                    );
                } else {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Add(prev_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextLeft(id.to_owned()),
                        Hunk::Add(prev_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextRight(id.to_owned()),
                        Hunk::Set(prev_card.to_owned()),
                    );
                }
            }

            // 4 (0100): id only in next left cards, which means a
            // card was added left
            if !lp && ln && !rp && !rn {
                let card = left.next().get(id).unwrap();

                patch.insert(
                    HunkKind::PrevLeft(id.to_owned()),
                    Hunk::Add(card.to_owned()),
                );
                patch.insert(
                    HunkKind::PrevRight(id.to_owned()),
                    Hunk::Add(card.to_owned()),
                );
                patch.insert(
                    HunkKind::NextRight(id.to_owned()),
                    Hunk::Add(card.to_owned()),
                );
            }

            // 5 (0101): id in next left and next right cards, which
            // means the same card was added both left and right at
            // the same time — unlikely to happen
            if !lp && ln && !rp && rn {
                let left_card = left.next().get(id).unwrap();
                let right_card = right.next().get(id).unwrap();

                if right_card.date >= left_card.date {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Add(right_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextLeft(id.to_owned()),
                        Hunk::Set(right_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Add(right_card.to_owned()),
                    );
                } else {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Add(left_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Add(left_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextRight(id.to_owned()),
                        Hunk::Set(left_card.to_owned()),
                    );
                }
            }

            // 6 (0110): id in next left and prev right cards, which
            // means the card was added left and deleted right
            if !lp && ln && rp && !rn {
                let left_card = left.next().get(id).unwrap();
                let right_card = right.prev().get(id).unwrap();

                if right_card.date >= left_card.date {
                    patch.insert(
                        HunkKind::NextLeft(id.to_owned()),
                        Hunk::Del(left_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Del(right_card.to_owned()),
                    );
                } else {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Add(left_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Set(left_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextRight(id.to_owned()),
                        Hunk::Add(left_card.to_owned()),
                    );
                }
            }

            // 7 (0111): id in next left and right cards, which means
            // the card was added left and potentially modified right
            if !lp && ln && rp && rn {
                let left_card = left.next().get(id).unwrap();
                let prev_right_card = right.prev().get(id).unwrap();
                let next_right_card = right.next().get(id).unwrap();

                if next_right_card.date >= left_card.date {
                    if next_right_card.date >= prev_right_card.date {
                        patch.insert(
                            HunkKind::PrevLeft(id.to_owned()),
                            Hunk::Add(next_right_card.to_owned()),
                        );
                        patch.insert(
                            HunkKind::NextLeft(id.to_owned()),
                            Hunk::Set(next_right_card.to_owned()),
                        );
                        if next_right_card.date != prev_right_card.date {
                            patch.insert(
                                HunkKind::PrevRight(id.to_owned()),
                                Hunk::Set(next_right_card.to_owned()),
                            );
                        }
                    } else {
                        patch.insert(
                            HunkKind::PrevLeft(id.to_owned()),
                            Hunk::Add(prev_right_card.to_owned()),
                        );
                        patch.insert(
                            HunkKind::NextLeft(id.to_owned()),
                            Hunk::Set(prev_right_card.to_owned()),
                        );
                        patch.insert(
                            HunkKind::NextRight(id.to_owned()),
                            Hunk::Set(prev_right_card.to_owned()),
                        );
                    }
                } else {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Add(left_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Set(left_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextRight(id.to_owned()),
                        Hunk::Set(left_card.to_owned()),
                    );
                }
            }

            // 8 (1000): id only in prev left cards, which means the
            // prev left card is obsolete
            if lp && !ln && !rp && !rn {
                let card = left.prev().get(id).unwrap();

                patch.insert(
                    HunkKind::PrevLeft(id.to_owned()),
                    Hunk::Del(card.to_owned()),
                );
            }

            // 9 (1001): id in prev left and next right cards, which
            // means a card was deleted left and added right
            if lp && !ln && !rp && rn {
                let left_card = left.prev().get(id).unwrap();
                let right_card = right.next().get(id).unwrap();

                if right_card.date >= left_card.date {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Set(right_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextLeft(id.to_owned()),
                        Hunk::Add(right_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Add(right_card.to_owned()),
                    );
                } else {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Del(left_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextRight(id.to_owned()),
                        Hunk::Del(right_card.to_owned()),
                    );
                }
            }

            // 10 (1010): id in prev left and prev right cards, which
            // means both prev are obsolete
            if lp && !ln && rp && !rn {
                let left_card = left.prev().get(id).unwrap();
                let right_card = right.prev().get(id).unwrap();

                patch.insert(
                    HunkKind::PrevLeft(id.to_owned()),
                    Hunk::Del(left_card.to_owned()),
                );
                patch.insert(
                    HunkKind::PrevRight(id.to_owned()),
                    Hunk::Del(right_card.to_owned()),
                );
            }

            // 11 (1011): id in prev left and right cards, which means
            // a card was deleted left and potentially modified right
            if lp && !ln && rp && rn {
                let prev_left_card = left.prev().get(id).unwrap();
                let prev_right_card = right.prev().get(id).unwrap();
                let next_right_card = right.next().get(id).unwrap();

                patch.insert(
                    HunkKind::PrevLeft(id.to_owned()),
                    Hunk::Del(prev_left_card.to_owned()),
                );
                patch.insert(
                    HunkKind::PrevRight(id.to_owned()),
                    Hunk::Del(prev_right_card.to_owned()),
                );
                patch.insert(
                    HunkKind::NextRight(id.to_owned()),
                    Hunk::Del(next_right_card.to_owned()),
                );
            }

            // 12 (1100): id in left cards, which means the previous
            // synchro failed and left is not in phase with right
            // anymore
            if lp && ln && !rp && !rn {
                let prev_card = left.prev().get(id).unwrap();
                let next_card = left.next().get(id).unwrap();

                if next_card.date >= prev_card.date {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Set(next_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Add(next_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextRight(id.to_owned()),
                        Hunk::Add(next_card.to_owned()),
                    );
                } else {
                    patch.insert(
                        HunkKind::NextLeft(id.to_owned()),
                        Hunk::Set(prev_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Add(prev_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextRight(id.to_owned()),
                        Hunk::Add(prev_card.to_owned()),
                    );
                }
            }

            // 13 (1101): id in left and next right cards, which means
            // the card was potentially modified left and added right
            if lp && ln && !rp && rn {
                let prev_left_card = left.prev().get(id).unwrap();
                let next_left_card = left.next().get(id).unwrap();
                let right_card = right.next().get(id).unwrap();

                if next_left_card.date >= right_card.date {
                    if next_left_card.date >= prev_left_card.date {
                        if next_left_card.date != prev_left_card.date {
                            patch.insert(
                                HunkKind::PrevLeft(id.to_owned()),
                                Hunk::Set(next_left_card.to_owned()),
                            );
                        }
                        patch.insert(
                            HunkKind::PrevRight(id.to_owned()),
                            Hunk::Add(next_left_card.to_owned()),
                        );
                        patch.insert(
                            HunkKind::NextRight(id.to_owned()),
                            Hunk::Set(next_left_card.to_owned()),
                        );
                    } else {
                        patch.insert(
                            HunkKind::NextLeft(id.to_owned()),
                            Hunk::Set(prev_left_card.to_owned()),
                        );
                        patch.insert(
                            HunkKind::PrevRight(id.to_owned()),
                            Hunk::Add(prev_left_card.to_owned()),
                        );
                        patch.insert(
                            HunkKind::NextRight(id.to_owned()),
                            Hunk::Set(prev_left_card.to_owned()),
                        );
                    }
                } else {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Set(right_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::NextLeft(id.to_owned()),
                        Hunk::Set(right_card.to_owned()),
                    );
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Add(right_card.to_owned()),
                    );
                }
            }

            // 14 (1110): id in left and prev right cards, which means
            // the card was deleted right
            if lp && ln && rp && !rn {
                let prev_left_card = left.prev().get(id).unwrap();
                let next_left_card = left.next().get(id).unwrap();
                let prev_right_card = right.prev().get(id).unwrap();

                patch.insert(
                    HunkKind::PrevLeft(id.to_owned()),
                    Hunk::Del(prev_left_card.to_owned()),
                );
                patch.insert(
                    HunkKind::NextLeft(id.to_owned()),
                    Hunk::Del(next_left_card.to_owned()),
                );
                patch.insert(
                    HunkKind::PrevRight(id.to_owned()),
                    Hunk::Del(prev_right_card.to_owned()),
                );
            }

            // 15 (1111): id everywhere
            if lp && ln && rp && rn {
                let prev_left_card = left.prev().get(id).unwrap();
                let next_left_card = left.next().get(id).unwrap();
                let prev_right_card = right.prev().get(id).unwrap();
                let next_right_card = right.next().get(id).unwrap();
                let mut cards = [
                    prev_left_card,
                    next_left_card,
                    prev_right_card,
                    next_right_card,
                ];
                cards.sort_by(|a, b| b.date.partial_cmp(&a.date).unwrap());
                let card = *cards.first().unwrap();
                if card != prev_left_card {
                    patch.insert(
                        HunkKind::PrevLeft(id.to_owned()),
                        Hunk::Set(card.to_owned()),
                    );
                }
                if card != next_left_card {
                    patch.insert(
                        HunkKind::NextLeft(id.to_owned()),
                        Hunk::Set(card.to_owned()),
                    );
                }
                if card != prev_right_card {
                    patch.insert(
                        HunkKind::PrevRight(id.to_owned()),
                        Hunk::Set(card.to_owned()),
                    );
                }
                if card != next_right_card {
                    patch.insert(
                        HunkKind::NextRight(id.to_owned()),
                        Hunk::Set(card.to_owned()),
                    );
                }
            }
        }

        patch
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local};
    use std::{collections::HashMap, iter::FromIterator};

    use super::*;

    struct TestCards {
        prev: CardsMap,
        next: CardsMap,
    }

    impl TestCards {
        pub fn new(prev: Vec<Card>, next: Vec<Card>) -> Self {
            Self {
                prev: HashMap::from_iter(
                    prev.iter()
                        .map(|card| (card.id.to_owned(), card.to_owned())),
                ),
                next: HashMap::from_iter(
                    next.iter()
                        .map(|card| (card.id.to_owned(), card.to_owned())),
                ),
            }
        }
    }

    impl Cards for TestCards {
        fn prev(&self) -> &CardsMap {
            &self.prev
        }

        fn next(&self) -> &CardsMap {
            &self.next
        }
    }

    macro_rules! card {
        ($id: literal, $date: literal) => {
            Card {
                id: format!("{}", $id),
                date: DateTime::parse_from_rfc3339(&format!("{}T00:00:00+00:00", $date))
                    .unwrap()
                    .with_timezone(&Local),
                content: String::new(),
            }
        };
    }

    #[test]
    fn test_patch_0000() {
        let left = TestCards::new(vec![], vec![]);
        let right = TestCards::new(vec![], vec![]);
        let patch = Patch::new(left, right);

        assert!(patch.hunks.is_empty());
    }

    #[test]
    fn test_patch_0001() {
        let left = TestCards::new(vec![], vec![]);
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-19")]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
    }

    #[test]
    fn test_patch_0010() {
        let left = TestCards::new(vec![], vec![]);
        let right = TestCards::new(vec![card!("id", "2020-01-19")], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(1, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
    }

    #[test]
    fn test_patch_0011() {
        // when next right date is before prev right date
        let left = TestCards::new(vec![], vec![]);
        let right = TestCards::new(
            vec![card!("id", "2020-01-18")],
            vec![card!("id", "2020-01-19")],
        );
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );

        // when next right date is after prev right date
        let left = TestCards::new(vec![], vec![]);
        let right = TestCards::new(
            vec![card!("id", "2020-01-20")],
            vec![card!("id", "2020-01-19")],
        );
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );
    }

    #[test]
    fn test_patch_0100() {
        let left = TestCards::new(vec![], vec![card!("id", "2020-01-19")]);
        let right = TestCards::new(vec![], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );
    }

    #[test]
    fn test_patch_0101() {
        // when left date is before right date
        let left = TestCards::new(vec![], vec![card!("id", "2020-01-18")]);
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-19")]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );

        // when left date is after right date
        let left = TestCards::new(vec![], vec![card!("id", "2020-01-20")]);
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-19")]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );
    }

    #[test]
    fn test_patch_0110() {
        let left = TestCards::new(vec![], vec![card!("id", "2020-01-18")]);
        let right = TestCards::new(vec![card!("id", "2020-01-19")], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(2, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-18"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
    }

    #[test]
    fn test_patch_0111() {
        // when left date is before right date and prev right date is
        // before next right date
        let left = TestCards::new(vec![], vec![card!("id", "2020-01-18")]);
        let right = TestCards::new(
            vec![card!("id", "2020-01-19")],
            vec![card!("id", "2020-01-20")],
        );
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );

        // when left date is before right date and prev right date is
        // equal to next right date
        let left = TestCards::new(vec![], vec![card!("id", "2020-01-18")]);
        let right = TestCards::new(
            vec![card!("id", "2020-01-19")],
            vec![card!("id", "2020-01-19")],
        );
        let patch = Patch::new(left, right);

        assert_eq!(2, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );

        // when left date is before right date and prev right date is
        // after next right date
        let left = TestCards::new(vec![], vec![card!("id", "2020-01-18")]);
        let right = TestCards::new(
            vec![card!("id", "2020-01-20")],
            vec![card!("id", "2020-01-19")],
        );
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );

        // when left date is after right date
        let left = TestCards::new(vec![], vec![card!("id", "2020-01-20")]);
        let right = TestCards::new(
            vec![card!("id", "2020-01-19")],
            vec![card!("id", "2020-01-19")],
        );
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );
    }

    #[test]
    fn test_patch_1000() {
        let left = TestCards::new(vec![card!("id", "2020-01-19")], vec![]);
        let right = TestCards::new(vec![], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(1, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
    }

    #[test]
    fn test_patch_1001() {
        // when left date is before right date
        let left = TestCards::new(vec![card!("id", "2020-01-18")], vec![]);
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-19")]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );

        // when left date is equal to right date
        let left = TestCards::new(vec![card!("id", "2020-01-19")], vec![]);
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-19")]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );

        // when left date is after to right date
        let left = TestCards::new(vec![card!("id", "2020-01-20")], vec![]);
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-19")]);
        let patch = Patch::new(left, right);

        assert_eq!(2, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );
    }

    #[test]
    fn test_patch_1010() {
        let left = TestCards::new(vec![card!("id", "2020-01-18")], vec![]);
        let right = TestCards::new(vec![card!("id", "2020-01-19")], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(2, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-18"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
    }

    #[test]
    fn test_patch_1011() {
        let left = TestCards::new(vec![card!("id", "2020-01-18")], vec![]);
        let right = TestCards::new(
            vec![card!("id", "2020-01-19")],
            vec![card!("id", "2020-01-20")],
        );
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-18"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );
    }

    #[test]
    fn test_patch_1100() {
        // when next left date is before prev left date
        let left = TestCards::new(
            vec![card!("id", "2020-01-18")],
            vec![card!("id", "2020-01-19")],
        );
        let right = TestCards::new(vec![], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );

        // when next right date is after prev right date
        let left = TestCards::new(
            vec![card!("id", "2020-01-20")],
            vec![card!("id", "2020-01-19")],
        );
        let right = TestCards::new(vec![], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );
    }

    #[test]
    fn test_patch_1101() {
        // when left date is before right date and prev right date is
        // before next right date
        let left = TestCards::new(
            vec![card!("id", "2020-01-19")],
            vec![card!("id", "2020-01-20")],
        );
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-18")]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );

        // when right date is before left date and prev left date is
        // equal to next left date
        let left = TestCards::new(
            vec![card!("id", "2020-01-19")],
            vec![card!("id", "2020-01-19")],
        );
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-18")]);
        let patch = Patch::new(left, right);

        assert_eq!(2, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );

        // when right date is before left date and prev left date is
        // after next left date
        let left = TestCards::new(
            vec![card!("id", "2020-01-20")],
            vec![card!("id", "2020-01-19")],
        );
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-18")]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );

        // when right date is after left date
        let left = TestCards::new(
            vec![card!("id", "2020-01-19")],
            vec![card!("id", "2020-01-19")],
        );
        let right = TestCards::new(vec![], vec![card!("id", "2020-01-20")]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Add(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Set(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
    }

    #[test]
    fn test_patch_1110() {
        let left = TestCards::new(
            vec![card!("id", "2020-01-18")],
            vec![card!("id", "2020-01-19")],
        );
        let right = TestCards::new(vec![card!("id", "2020-01-20")], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(3, patch.hunks.len(), "{:?}", patch);
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-18"))),
            patch.hunks.get(&HunkKind::PrevLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-19"))),
            patch.hunks.get(&HunkKind::NextLeft("id".into())),
        );
        assert_eq!(
            Some(&Hunk::Del(card!("id", "2020-01-20"))),
            patch.hunks.get(&HunkKind::PrevRight("id".into())),
        );
    }

    #[test]
    fn test_patch_1111() {
        // TODO
    }
}
