use std::collections::HashSet;

use crate::card::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hunk {
    PrevLeft(HunkKind),
    NextLeft(HunkKind),
    PrevRight(HunkKind),
    NextRight(HunkKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HunkKind {
    Add(Card),
    Set(Card),
    Del(String),
}

#[derive(Debug)]
pub struct Patch {
    hunks: Vec<Hunk>,
}

impl Patch {
    pub fn new(left: impl Cards, right: impl Cards) -> Self {
        let mut ids = HashSet::new();
        //ids.extend(left.prev().iter().map(|(id, _)| id.as_str()));
        ids.extend(left.next().iter().map(|(id, _)| id.as_str()));
        //ids.extend(right.prev().iter().map(|(id, _)| id.as_str()));
        ids.extend(right.next().iter().map(|(id, _)| id.as_str()));

        let mut hunks = Vec::new();

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

                hunks.push(Hunk::PrevLeft(HunkKind::Add(card.to_owned())));
                hunks.push(Hunk::NextLeft(HunkKind::Add(card.to_owned())));
                hunks.push(Hunk::PrevRight(HunkKind::Add(card.to_owned())));
            }

            // 2 (0010): id only in prev right cards, which means prev
            // right card is obsolete
            if !lp && !ln && rp && !rn {
                hunks.push(Hunk::PrevRight(HunkKind::Del(id.to_owned())));
            }

            // 3 (0011): id in right cards, which means the previous
            // synchro failed and left is not in phase with right
            // anymore
            if !lp && !ln && rp && rn {
                let card = right.next().get(id).unwrap();

                hunks.push(Hunk::PrevLeft(HunkKind::Add(card.to_owned())));
                hunks.push(Hunk::NextLeft(HunkKind::Add(card.to_owned())));
                hunks.push(Hunk::PrevRight(HunkKind::Set(card.to_owned())));
            }

            // 4 (0100): id only in next left cards, which means a
            // card was added left
            if !lp && ln && !rp && !rn {
                let card = left.next().get(id).unwrap();

                hunks.push(Hunk::PrevRight(HunkKind::Add(card.to_owned())));
                hunks.push(Hunk::NextRight(HunkKind::Add(card.to_owned())));
                hunks.push(Hunk::PrevLeft(HunkKind::Add(card.to_owned())));
            }

            // 5 (0101): id in next left and next right cards, which
            // means the same card was added both left and right at
            // the same time — unlikely to happen
            if !lp && ln && !rp && rn {
                let left_card = left.next().get(id).unwrap();
                let right_card = left.next().get(id).unwrap();

                if right_card.date > left_card.date {
                    hunks.push(Hunk::PrevLeft(HunkKind::Add(right_card.to_owned())));
                    hunks.push(Hunk::NextLeft(HunkKind::Set(right_card.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Add(right_card.to_owned())));
                } else {
                    hunks.push(Hunk::PrevLeft(HunkKind::Add(left_card.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Add(left_card.to_owned())));
                    hunks.push(Hunk::NextRight(HunkKind::Set(left_card.to_owned())));
                }
            }

            // 6 (0110): id in next left and prev right cards, which
            // means the card was added left and deleted right
            if !lp && ln && rp && !rn {
                let left_card = left.next().get(id).unwrap();
                let right_card = right.prev().get(id).unwrap();

                if right_card.date > left_card.date {
                    hunks.push(Hunk::NextLeft(HunkKind::Del(id.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Del(id.to_owned())));
                } else {
                    hunks.push(Hunk::PrevLeft(HunkKind::Add(left_card.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Set(left_card.to_owned())));
                    hunks.push(Hunk::NextRight(HunkKind::Add(left_card.to_owned())));
                }
            }

            // 7 (0111): id in next left and right cards, which means
            // the card was added left and potentially modified right
            if !lp && ln && rp && rn {
                let left_card = left.next().get(id).unwrap();
                let prev_right_card = right.prev().get(id).unwrap();
                let next_right_card = right.next().get(id).unwrap();

                if next_right_card.date > left_card.date {
                    hunks.push(Hunk::PrevLeft(HunkKind::Add(next_right_card.to_owned())));
                    hunks.push(Hunk::NextLeft(HunkKind::Set(next_right_card.to_owned())));
                    if next_right_card.date > prev_right_card.date {
                        hunks.push(Hunk::PrevRight(HunkKind::Set(next_right_card.to_owned())));
                    }
                } else {
                    hunks.push(Hunk::PrevLeft(HunkKind::Add(left_card.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Set(left_card.to_owned())));
                    hunks.push(Hunk::NextRight(HunkKind::Set(left_card.to_owned())));
                }
            }

            // 8 (1000): id only in prev left cards, which means the
            // prev left card is obsolete
            if lp && !ln && !rp && !rn {
                hunks.push(Hunk::PrevLeft(HunkKind::Del(id.to_owned())));
            }

            // 9 (1001): id in prev left and next right cards, which
            // means a card was deleted left and added right
            if lp && !ln && !rp && rn {
                let left_card = left.prev().get(id).unwrap();
                let right_card = right.next().get(id).unwrap();

                if right_card.date > left_card.date {
                    hunks.push(Hunk::PrevLeft(HunkKind::Set(right_card.to_owned())));
                    hunks.push(Hunk::NextLeft(HunkKind::Add(right_card.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Add(right_card.to_owned())));
                } else {
                    hunks.push(Hunk::PrevLeft(HunkKind::Del(id.to_owned())));
                    hunks.push(Hunk::NextRight(HunkKind::Del(id.to_owned())));
                }
            }

            // 10 (1010): id in prev left and prev right cards, which
            // means both prev are obsolete
            if lp && !ln && rp && !rn {
                hunks.push(Hunk::PrevLeft(HunkKind::Del(id.to_owned())));
                hunks.push(Hunk::PrevRight(HunkKind::Del(id.to_owned())));
            }

            // 11 (1011): id in prev left and right cards, which means
            // a card was deleted left and potentially deleted right
            if lp && !ln && rp && rn {
                let left_card = left.prev().get(id).unwrap();
                let prev_right_card = right.prev().get(id).unwrap();
                let next_right_card = right.next().get(id).unwrap();

                if next_right_card.date > left_card.date {
                    hunks.push(Hunk::PrevLeft(HunkKind::Set(next_right_card.to_owned())));
                    hunks.push(Hunk::NextLeft(HunkKind::Add(next_right_card.to_owned())));
                    if next_right_card.date > prev_right_card.date {
                        hunks.push(Hunk::PrevRight(HunkKind::Set(next_right_card.to_owned())));
                    }
                } else {
                    hunks.push(Hunk::PrevLeft(HunkKind::Del(id.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Del(id.to_owned())));
                    hunks.push(Hunk::NextRight(HunkKind::Del(id.to_owned())));
                }
            }

            // 12 (1100): id in left cards, which means the previous
            // synchro failed and left is not in phase with right
            // anymore
            if lp && ln && !rp && !rn {
                let prev_card = left.prev().get(id).unwrap();
                let next_card = left.next().get(id).unwrap();

                if next_card.date > prev_card.date {
                    hunks.push(Hunk::PrevLeft(HunkKind::Set(next_card.to_owned())));
                }
                hunks.push(Hunk::PrevRight(HunkKind::Add(next_card.to_owned())));
                hunks.push(Hunk::NextRight(HunkKind::Add(next_card.to_owned())));
            }

            // 13 (1101): id in left and next right cards, which means
            // the card was potentially modified left and added right
            if lp && ln && !rp && rn {
                let prev_left_card = left.prev().get(id).unwrap();
                let next_left_card = left.next().get(id).unwrap();
                let right_card = right.next().get(id).unwrap();

                if right_card.date > next_left_card.date {
                    hunks.push(Hunk::PrevLeft(HunkKind::Set(right_card.to_owned())));
                    hunks.push(Hunk::NextLeft(HunkKind::Set(right_card.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Add(right_card.to_owned())));
                } else {
                    if next_left_card.date > prev_left_card.date {
                        hunks.push(Hunk::PrevLeft(HunkKind::Set(next_left_card.to_owned())));
                    }
                    hunks.push(Hunk::PrevRight(HunkKind::Add(next_left_card.to_owned())));
                    hunks.push(Hunk::NextRight(HunkKind::Set(next_left_card.to_owned())));
                }
            }

            // 14 (1110): id in left and prev right cards, which means
            // the card was potentially modified left and deleted
            // right
            if lp && ln && rp && !rn {
                let prev_left_card = left.prev().get(id).unwrap();
                let next_left_card = left.next().get(id).unwrap();
                let right_card = right.prev().get(id).unwrap();

                if right_card.date > next_left_card.date {
                    hunks.push(Hunk::PrevLeft(HunkKind::Del(id.to_owned())));
                    hunks.push(Hunk::NextLeft(HunkKind::Del(id.to_owned())));
                    hunks.push(Hunk::PrevRight(HunkKind::Del(id.to_owned())));
                } else {
                    if next_left_card.date > prev_left_card.date {
                        hunks.push(Hunk::PrevLeft(HunkKind::Set(next_left_card.to_owned())));
                    }
                    hunks.push(Hunk::PrevRight(HunkKind::Add(next_left_card.to_owned())));
                    hunks.push(Hunk::NextRight(HunkKind::Set(next_left_card.to_owned())));
                }
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
                    hunks.push(Hunk::PrevLeft(HunkKind::Set(card.to_owned())));
                }
                if card != next_left_card {
                    hunks.push(Hunk::NextLeft(HunkKind::Set(card.to_owned())));
                }
                if card != prev_right_card {
                    hunks.push(Hunk::PrevRight(HunkKind::Set(card.to_owned())));
                }
                if card != next_right_card {
                    hunks.push(Hunk::NextRight(HunkKind::Set(card.to_owned())));
                }
            }
        }

        Self { hunks }
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

        assert_eq!(vec![] as Vec<Hunk>, patch.hunks);
    }

    #[test]
    fn test_patch_0001() {
        let left = TestCards::new(vec![], vec![]);
        let right = TestCards::new(vec![], vec![card!("new", "2020-01-19")]);
        let patch = Patch::new(left, right);

        assert_eq!(
            vec![
                Hunk::PrevLeft(HunkKind::Add(card!("new", "2020-01-19"))),
                Hunk::NextLeft(HunkKind::Add(card!("new", "2020-01-19"))),
                Hunk::PrevRight(HunkKind::Add(card!("new", "2020-01-19"))),
            ],
            patch.hunks
        );
    }

    #[test]
    fn test_patch_0010() {
        let left = TestCards::new(vec![], vec![]);
        let right = TestCards::new(vec![card!("old", "2020-01-19")], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(
            vec![Hunk::PrevRight(HunkKind::Del("old".into()))],
            patch.hunks
        );
    }

    #[test]
    fn test_patch_0011() {
        let left = TestCards::new(vec![], vec![]);
        let right = TestCards::new(
            vec![card!("old", "2020-01-19")],
            vec![card!("new", "2020-01-20")],
        );
        let patch = Patch::new(left, right);

        assert_eq!(
            vec![
                Hunk::PrevLeft(HunkKind::Add(card!("new", "2020-01-20"))),
                Hunk::NextLeft(HunkKind::Add(card!("new", "2020-01-20"))),
                Hunk::PrevRight(HunkKind::Set(card!("new", "2020-01-20"))),
            ],
            patch.hunks
        );
    }

    #[test]
    fn test_patch_0100() {
        let left = TestCards::new(vec![], vec![card!("new", "2020-01-19")]);
        let right = TestCards::new(vec![], vec![]);
        let patch = Patch::new(left, right);

        assert_eq!(
            vec![
                Hunk::PrevRight(HunkKind::Set(card!("new", "2020-01-20"))),
                Hunk::NextLeft(HunkKind::Add(card!("new", "2020-01-20"))),
                Hunk::PrevLeft(HunkKind::Add(card!("new", "2020-01-20"))),
            ],
            patch.hunks
        );
    }
}
