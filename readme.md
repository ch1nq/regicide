# Regicide
A framework for playing the card game Regicide.

## Requirements
The package requires python version 3.7+ and should be compatible with Linux, Windows and macOS.

## Installation
The Regicide framework can be installed in python using pip.
```shell
$ pip install regicide
```

## Usage

## Example
The following is an example of implementing an agent with custom behavior. In this example the agent will always choose to play the six of diamonds if available. Otherwise, it will choose randomly from the available actions.
```python
import random

import regicide
from regicide import actions, card, players


class CustomPlayer:
    """
    A player that always plays the Six of Diamonds if possible.
    Otherwise it will choose a random action.
    """

    def play(self, state):
        legal_actions = state.action_space()

        c = card.Card(card.CardSuit.Diamonds, card.CardValue.Six)
        special_action = actions.ActionPlay(c)

        if special_action in legal_actions:
            return special_action
        else:
            return random.choice(legal_actions)


players = [CustomPlayer(), CustomPlayer(), CustomPlayer()]
game = regicide.RegicideGame(players, seed=1337)
result = game.playout()

print(result, game.reward())
```

## API
The framework has three submodules: \cd{actions}, \cd{card} and \cd{players}. Each module contains python classes that can be instantiated and used in the framework.

### Actions
Available actions are:
- `Play(card: Card)`
- `AnimalCombo(cards: List[Card])`
- `Combo(cards: List[Card])`
- `Yield()`
- `Discard(cards: List[Card])`
- `ChangePlayer(id: int)`
- `RefillHand`

### Cards
Available cards are:
- `Card(suit: CardSuit, value: CardValue)`
- `CardSuit` that can be: `Spades`, `Hearts`, `Diamonds`, `Clubs` or `None`.
- `CardValue` that can be `Ace`...`King` or the special value `Jester`.

### Players
- `InputPlayer()` that allows user input for taking actions.
- `RandomPlayer(seed: int)`
- `MCTSPlayer(playouts: int, num_threads: int, uct_variation: int, use_heuristics: bool)`

## Caveats
Currently, the python package does not support code suggestions in IDE's, making it more difficult to work with the package. This is a result of the method used to generate the python bindings and has no implication on actual performance or correctness of the program.
