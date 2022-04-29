from regicide import actions, card, players
import regicide
import random

class CustomPlayer():
    def __init__(self) -> None:
        pass

    def play(self, state):
        c = card.Card(card.CardSuit.Spades, card.CardValue.Eight)
        special_action = actions.ActionPlay(c)

        legal_actions = state.action_space()

        if special_action in legal_actions:
            action = special_action
        else:
            action = random.choice(legal_actions)

        print(f"{state.reward()}: {state.current_enemy()}")
        print(action)
        print()
        
        return action
            

            
    
players = [
    CustomPlayer
    # players.MCTSPlayer(n_playouts=1_000, use_heuristics=False)
    # players.InputPlayer(),
]*3

game = regicide.RegicideGame(players, 1337)
result = game.playout()
print(f"{result = }")
