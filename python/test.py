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
        self.base = regicide.players.MCTSPlayer(
            playouts=10000, use_heuristics=False, num_threads=7, policy_variation=2,
        )

    def play(self, state):
        print(state)
        print("Top actions: ")
        best_action = self.base.play(state)

        
        for i, action_info in enumerate(self.base.ranked_actions()[:3]): 
            prefix = "->" if i == 0 else "  "
            print(f"{prefix} {i}: {action_info}")
        
        print()
        print("-" * 80)
        print()
        return best_action


players = [
    # CustomMCTSPlayer(),
    # CustomMCTSPlayer(),
    CustomMCTSPlayer(),
    CustomMCTSPlayer(),
    CustomMCTSPlayer(),
    # CustomPlayer(),
    # players.MCTSPlayer(playouts=1_000, num_threads=4, use_heuristics=False),
    # players.InputPlayer(),
] 

game = regicide.RegicideGame(players, seed=None)
result = game.playout()
print(f"{result = }, reward = {game.reward()}")
