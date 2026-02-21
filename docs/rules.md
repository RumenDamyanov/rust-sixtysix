# Detailed Rules (Sixty-six)

This document expands on the brief rules summary in the root README.

## Deck & Values

24 cards: A (11), 10 (10), K (4), Q (3), J (2), 9 (0) in four suits.

## Objective

Capture card points to reach 66 first. If both fail before all tricks, last trick bonus may swing outcome (implementation awards +10 to last trick winner when hands empty).

## Deal

3 cards each, 3 cards each (total 6), remaining stock face-down, last card face-up (trump card) determining trump suit.

## Turn / Trick

Leader plays a card. While stock is open (not closed and not empty) follower may play any card. After stock closed or empty, follower must follow suit if possible; otherwise may play any; trump may be used to win.

## Trick Resolution

Higher card of suit led wins unless trump played versus non-trump. Winner scores points of both cards, leads next, and (if stock open) draws first, then opponent, until stock empties.

## Marriages

Declaring K+Q of same suit at the lead: 20 points (non-trump) or 40 (trump). Must declare before playing first card of the trick.

## Closing the Stock

Leader may close stock before playing a card (action `closeStock`). No further drawing; follow-suit rule enforced for remainder of deal.

## Trump Exchange

Leader holding the 9 of trump may exchange it with the face-up trump card while stock open and before playing a card (action `exchangeTrump`).

## End & Scoring

Deal ends immediately when a player reaches 66 or (implementation detail) when the final trick resolves; last trick winner gains +10.

Future extensions (not yet implemented): match scoring (schneider/schwarz), multi-deal tally.
