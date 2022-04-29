from regicide import actions, card, players
import regicide

class CustomPlayer():
    def __init__(self) -> None:
        pass

    def play(self, state):
        c = card.Card(card.CardSuit.Spades, card.CardValue.Queen)
        special_action = actions.ActionPlay(c)
        return special_action
    
players = [
    players.MCTSPlayer(n_playouts=1_000, use_heuristics=False)
    # players.InputPlayer(),
]*3

game = regicide.RegicideGame(players, None)
result = game.playout()
print(f"{result = }")
