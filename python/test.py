from regicide import actions, card, players
import regicide
import random

class CustomPlayer:
    def __init__(self) -> None:
        pass

    def play(self, state):
        c = card.Card(card.CardSuit.Diamonds, card.CardValue.Six)
        special_action = actions.ActionPlay(c)

        legal_actions = state.action_space()

        if special_action in legal_actions:
            action = special_action
        else:
            action = random.choice(legal_actions)

        print(f"{state.reward()}: {state.current_enemy()}")
        print([x.__str__() for x in state.current_hand()])
            
        print(action)
        print()
        
        return action
            
class CustomMCTSPlayer:
    def __init__(self) -> None:
        self.base = regicide.players.MCTSPlayer(n_playouts=1000, use_heuristics=True)

    def play(self, state):
        action = self.base.play(state)
        print(f"{state.reward()} -> {action}")
        return action
    
players = [
    CustomMCTSPlayer(),
    # CustomPlayer(),
    # players.MCTSPlayer(n_playouts=10_000, use_heuristics=False),
    # players.InputPlayer(),
]

game = regicide.RegicideGame(players, 1337)
result = game.playout()
print(f"{result = }, reward = {game.reward()}")

