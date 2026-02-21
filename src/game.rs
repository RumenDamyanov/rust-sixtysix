//! Sixty-six game rules implementation.
//!
//! Card encoding: `suit * 100 + rank_value`
//!
//! Suits: Clubs=0, Diamonds=1, Hearts=2, Spades=3
//! Rank values: A=11, 10=10, K=4, Q=3, J=2, 9=0

use crate::engine::{Action, Game, GameError};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const CLUBS: i32 = 0;
pub const DIAMONDS: i32 = 1;
pub const HEARTS: i32 = 2;
pub const SPADES: i32 = 3;

const SUITS: [i32; 4] = [CLUBS, DIAMONDS, HEARTS, SPADES];
const RANK_ORDER: [i32; 6] = [11, 10, 4, 3, 2, 0];

pub const ACTION_PLAY: &str = "play";
pub const ACTION_CLOSE_STOCK: &str = "closeStock";
pub const ACTION_DECLARE: &str = "declare";
pub const ACTION_EXCHANGE: &str = "exchangeTrump";

// ---------------------------------------------------------------------------
// Card helpers
// ---------------------------------------------------------------------------

fn card(suit: i32, rank_val: i32) -> i32 {
    suit * 100 + rank_val
}
fn card_suit(c: i32) -> i32 {
    c / 100
}
fn card_val(c: i32) -> i32 {
    c % 100
}
fn trick_points(c: i32) -> i32 {
    card_val(c)
}

fn has_suit(hand: &[i32], suit: i32) -> bool {
    hand.iter().any(|&c| card_suit(c) == suit)
}

fn remove(hand: &[i32], card: i32) -> Vec<i32> {
    let mut result = Vec::with_capacity(hand.len());
    let mut removed = false;
    for &c in hand {
        if c == card && !removed {
            removed = true;
        } else {
            result.push(c);
        }
    }
    result
}

fn trick_winner(lead: i32, follow: i32, trump: i32) -> usize {
    let ls = card_suit(lead);
    let fs = card_suit(follow);
    if fs == ls {
        if card_val(follow) > card_val(lead) {
            return 1;
        }
        return 0;
    }
    if fs == trump && ls != trump {
        return 1;
    }
    0
}

fn new_deck() -> Vec<i32> {
    let mut deck = Vec::with_capacity(24);
    for &suit in &SUITS {
        for &rv in &RANK_ORDER {
            deck.push(card(suit, rv));
        }
    }
    deck
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub current: usize,
    pub scores: [i32; 2],
    pub hands: [Vec<i32>; 2],
    pub stock: Vec<i32>,
    pub closed: bool,
    pub trump_suit: i32,
    pub trump_card: i32,
    pub trick: Vec<i32>,
    pub winner: i32,
}

// ---------------------------------------------------------------------------
// Payload helpers
// ---------------------------------------------------------------------------

fn get_int(action: &Action, key: &str) -> Option<i32> {
    action
        .payload
        .as_ref()?
        .get(key)?
        .as_i64()
        .map(|v| v as i32)
}

// ---------------------------------------------------------------------------
// SixtySix game
// ---------------------------------------------------------------------------

/// The Sixty-six game implementing the [`Game`] trait.
pub struct SixtySix;

impl Game for SixtySix {
    fn name(&self) -> &str {
        "sixtysix"
    }

    fn initial_state(&self, seed: i64) -> serde_json::Value {
        let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
        let mut deck = new_deck();
        deck.shuffle(&mut rng);

        let trump_card = *deck.last().unwrap();
        let trump_suit = card_suit(trump_card);

        let mut hands: [Vec<i32>; 2] = [vec![], vec![]];

        // Deal 3 to player 0, 3 to player 1, 3 to player 0, 3 to player 1
        for (pos, &count) in [3, 3, 3, 3].iter().enumerate() {
            let player = if pos % 2 == 0 { 0 } else { 1 };
            for _ in 0..count {
                hands[player].push(deck[0]);
                deck = deck[1..].to_vec();
            }
        }

        // Stock is all remaining cards minus the trump card (last)
        let stock: Vec<i32> = deck[..deck.len() - 1].to_vec();

        // Sort hands
        for hand in &mut hands {
            hand.sort_by(|a, b| {
                let suit_cmp = card_suit(*a).cmp(&card_suit(*b));
                if suit_cmp != std::cmp::Ordering::Equal {
                    return suit_cmp;
                }
                card_val(*a).cmp(&card_val(*b))
            });
        }

        let state = State {
            current: 0,
            scores: [0, 0],
            hands,
            stock,
            closed: false,
            trump_suit,
            trump_card,
            trick: vec![],
            winner: -1,
        };

        serde_json::to_value(state).unwrap()
    }

    fn validate(&self, state: &serde_json::Value, action: &Action) -> Result<(), GameError> {
        let st: State =
            serde_json::from_value(state.clone()).map_err(|e| GameError(e.to_string()))?;

        if st.winner != -1 {
            return Err(GameError("game over".to_string()));
        }

        match action.action_type.as_str() {
            ACTION_PLAY => {
                let c = get_int(action, "card").ok_or(GameError("missing card".to_string()))?;
                if !st.hands[st.current].contains(&c) {
                    return Err(GameError("card not in hand".to_string()));
                }
                if st.trick.len() == 1 && (st.closed || st.stock.is_empty()) {
                    let lead = st.trick[0];
                    let ls = card_suit(lead);
                    if card_suit(c) != ls && has_suit(&st.hands[st.current], ls) {
                        return Err(GameError("must follow suit".to_string()));
                    }
                }
                Ok(())
            }
            ACTION_CLOSE_STOCK => {
                if st.closed || st.stock.is_empty() {
                    return Err(GameError("cannot close".to_string()));
                }
                if !st.trick.is_empty() {
                    return Err(GameError("cannot close mid-trick".to_string()));
                }
                Ok(())
            }
            ACTION_DECLARE => {
                let suit = get_int(action, "suit").ok_or(GameError("missing suit".to_string()))?;
                let k = card(suit, 4);
                let q = card(suit, 3);
                if !(st.hands[st.current].contains(&k) && st.hands[st.current].contains(&q)) {
                    return Err(GameError("no marriage".to_string()));
                }
                if !st.trick.is_empty() {
                    return Err(GameError("declare only on lead".to_string()));
                }
                Ok(())
            }
            ACTION_EXCHANGE => {
                if st.closed || st.stock.is_empty() {
                    return Err(GameError(
                        "cannot exchange when stock closed or empty".to_string(),
                    ));
                }
                if !st.trick.is_empty() {
                    return Err(GameError("exchange only at lead".to_string()));
                }
                let nine = card(st.trump_suit, 0);
                if !st.hands[st.current].contains(&nine) {
                    return Err(GameError("no nine of trump to exchange".to_string()));
                }
                Ok(())
            }
            _ => Err(GameError("unknown action".to_string())),
        }
    }

    fn apply(
        &self,
        state: &serde_json::Value,
        action: &Action,
    ) -> Result<serde_json::Value, GameError> {
        let mut st: State =
            serde_json::from_value(state.clone()).map_err(|e| GameError(e.to_string()))?;

        match action.action_type.as_str() {
            ACTION_PLAY => {
                let c = get_int(action, "card").unwrap();
                let actor = st.current;
                st.hands[actor] = remove(&st.hands[actor], c);
                st.trick.push(c);

                if st.trick.len() == 2 {
                    let winner_idx = trick_winner(st.trick[0], st.trick[1], st.trump_suit);
                    // winner_idx is 0 for leader, 1 for follower
                    // Map to actual player: leader was the player who played first
                    // The leader is (1 - actor) since actor is the follower who just played
                    let leader = 1 - actor;
                    let actual_winner = if winner_idx == 0 { leader } else { actor };

                    let pts = trick_points(st.trick[0]) + trick_points(st.trick[1]);
                    st.scores[actual_winner] += pts;
                    st.trick.clear();

                    if !st.closed && !st.stock.is_empty() {
                        if st.stock.len() >= 2 {
                            st.hands[actual_winner].push(st.stock[0]);
                            st.hands[1 - actual_winner].push(st.stock[1]);
                            st.stock = st.stock[2..].to_vec();
                        } else {
                            st.hands[actual_winner].push(st.stock[0]);
                            st.stock.clear();
                        }
                    }

                    st.current = actual_winner;

                    if st.hands[0].is_empty() && st.hands[1].is_empty() {
                        st.scores[actual_winner] += 10;
                    }

                    if st.scores[actual_winner] >= 66 {
                        st.winner = actual_winner as i32;
                    }
                } else {
                    st.current = 1 - actor;
                }

                Ok(serde_json::to_value(st).unwrap())
            }
            ACTION_CLOSE_STOCK => {
                st.closed = true;
                Ok(serde_json::to_value(st).unwrap())
            }
            ACTION_DECLARE => {
                let suit = get_int(action, "suit").unwrap();
                let pts = if suit == st.trump_suit { 40 } else { 20 };
                st.scores[st.current] += pts;
                if st.scores[st.current] >= 66 {
                    st.winner = st.current as i32;
                }
                Ok(serde_json::to_value(st).unwrap())
            }
            ACTION_EXCHANGE => {
                let nine = card(st.trump_suit, 0);
                st.hands[st.current] = remove(&st.hands[st.current], nine);
                st.hands[st.current].push(st.trump_card);
                st.trump_card = nine;
                Ok(serde_json::to_value(st).unwrap())
            }
            _ => Err(GameError("unknown action".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action_play(card: i32) -> Action {
        let mut payload = std::collections::HashMap::new();
        payload.insert("card".to_string(), serde_json::json!(card));
        Action {
            action_type: ACTION_PLAY.to_string(),
            actor: None,
            payload: Some(payload),
            idempotency_key: None,
        }
    }

    fn parse_state(val: &serde_json::Value) -> State {
        serde_json::from_value(val.clone()).unwrap()
    }

    #[test]
    fn initial_deal_deterministic() {
        let g = SixtySix;
        let a = parse_state(&g.initial_state(42));
        let b = parse_state(&g.initial_state(42));
        assert_eq!(a.trump_suit, b.trump_suit);
        assert_eq!(a.trump_card, b.trump_card);
        assert_eq!(a.hands[0].len(), 6);
        assert_eq!(a.hands[1].len(), 6);
        // deck=24, trump=1, dealt=12 => stock=11
        assert_eq!(a.stock.len(), 11);
    }

    #[test]
    fn play_and_trick_resolution() {
        let g = SixtySix;
        let state = g.initial_state(1);
        let st = parse_state(&state);

        let lead = st.hands[st.current][0];
        g.validate(&state, &action_play(lead)).unwrap();
        let ns = g.apply(&state, &action_play(lead)).unwrap();
        let st2 = parse_state(&ns);

        let follow = st2.hands[st2.current][0];
        g.validate(&ns, &action_play(follow)).unwrap();
        let ns2 = g.apply(&ns, &action_play(follow)).unwrap();
        let st3 = parse_state(&ns2);

        assert!(st3.trick.is_empty(), "trick should be resolved");
    }

    #[test]
    fn close_stock_enforces_follow_suit() {
        let g = SixtySix;
        let state = g.initial_state(7);
        let mut st = parse_state(&state);
        st.closed = true;
        let state = serde_json::to_value(&st).unwrap();

        let lead = st.hands[st.current][0];
        let ns = g.apply(&state, &action_play(lead)).unwrap();
        let st2 = parse_state(&ns);

        let ls = card_suit(lead);
        let follower = st2.current;
        // Find an off-suit card in follower's hand
        if let Some(&off) = st2.hands[follower].iter().find(|&&c| card_suit(c) != ls) {
            let result = g.validate(&ns, &action_play(off));
            // Should fail if they also have a same-suit card
            if has_suit(&st2.hands[follower], ls) {
                assert!(
                    result.is_err(),
                    "expected follow-suit requirement when closed"
                );
            }
        }
    }

    #[test]
    fn declare_marriage_and_exchange() {
        let g = SixtySix;
        let state = g.initial_state(99);
        let st = parse_state(&state);

        // Try to declare trump marriage
        let mut payload = std::collections::HashMap::new();
        payload.insert("suit".to_string(), serde_json::json!(st.trump_suit));
        let declare_action = Action {
            action_type: ACTION_DECLARE.to_string(),
            actor: None,
            payload: Some(payload),
            idempotency_key: None,
        };

        let state = if g.validate(&state, &declare_action).is_ok() {
            g.apply(&state, &declare_action).unwrap()
        } else {
            state
        };

        // Try exchange
        let exchange_action = Action {
            action_type: ACTION_EXCHANGE.to_string(),
            actor: None,
            payload: None,
            idempotency_key: None,
        };

        if g.validate(&state, &exchange_action).is_ok() {
            let ns = g.apply(&state, &exchange_action).unwrap();
            let st2 = parse_state(&ns);
            assert_eq!(card_suit(st2.trump_card), st2.trump_suit);
            assert_eq!(card_val(st2.trump_card), 0);
        }
    }

    #[test]
    fn last_trick_bonus() {
        let g = SixtySix;
        let st = State {
            current: 0,
            scores: [0, 0],
            hands: [vec![card(HEARTS, 0)], vec![card(HEARTS, 11)]],
            stock: vec![],
            closed: true,
            trump_suit: SPADES,
            trump_card: card(SPADES, 0),
            trick: vec![],
            winner: -1,
        };
        let state = serde_json::to_value(&st).unwrap();

        // Player 0 leads 9 of hearts
        let ns = g.apply(&state, &action_play(card(HEARTS, 0))).unwrap();
        // Player 1 follows with ace of hearts
        let ns2 = g.apply(&ns, &action_play(card(HEARTS, 11))).unwrap();
        let final_st = parse_state(&ns2);

        assert!(final_st.hands[0].is_empty() && final_st.hands[1].is_empty());
        // Player 1 wins the trick (ace > 9), gets 11+0+10(bonus)=21
        assert!(
            final_st.scores[1] >= 21,
            "expected last trick bonus, scores={:?}",
            final_st.scores
        );
    }
}
