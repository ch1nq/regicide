from encodings import utf_8
import os
import random
from multiprocessing import Pool
from typing import Any, Dict, Optional
import logging
from datetime import datetime

import pandas as pd
import regicide
from tqdm import tqdm

from plots import plot

DATA_PATH = "data"
CSV_PATH = os.path.join(DATA_PATH, "games_gridsearch.csv")
COLUMNS = ["score", "agent", "player_count", "tree_policy"]

if not os.path.exists("logs"):
    os.mkdir("logs")
logging.basicConfig(
    filename=f"logs/regicide-{datetime.now().strftime('%Y%m%d%H%M%S')}.log", 
    encoding="utf-8", 
    level=logging.DEBUG,
)

def single_playout(kwargs: Dict[str, Any]) -> pd.DataFrame:
    mcts_playouts = kwargs["mcts_playouts"]
    player_count = kwargs["player_count"]
    policy_variation = kwargs["policy_variation"]
    num_threads = kwargs.get("num_threads", 1)
    agent = f"MCTS_{mcts_playouts}"

    logging.info(f"Started game with parameters: {mcts_playouts=}, {player_count=}, {policy_variation=}.")
    
    gamers = [
        regicide.players.MCTSPlayer(
            playouts=mcts_playouts,
            num_threads=num_threads,
            use_heuristics=False,
            policy_variation=policy_variation
        ),
    ] * player_count

    game = regicide.RegicideGame(gamers, None)
    game.playout()
    
    logging.info(f"Finished game with parameters: {mcts_playouts=}, {player_count=}, {policy_variation=}. Outcome: {game.reward()}")

    return pd.DataFrame(
        [[game.reward(), agent, player_count, policy_variation]], 
        columns=COLUMNS
    )


def multi_playout(kwargs, queue=None) -> Optional[pd.DataFrame]:
    data = pd.DataFrame(columns=COLUMNS)
    
    for _ in range(kwargs["samples"]):
        row = single_playout(kwargs)
        data = pd.concat([data, row])
    
    if queue is None:
        return data
    else:
        queue.put(data)


def generate_data_gridsearch(samples) -> pd.DataFrame:
    args = []
    for _ in range(samples):
        for policy_variation in [0,3,4]:
            for player_count in [1,2,3,4]:
                for mcts_playouts in [10 ** i for i in [1,5]]:
                    args.append({
                        "player_count": player_count,
                        "mcts_playouts": mcts_playouts,
                        "num_threads": 1,
                        "policy_variation": policy_variation,
                    })

    # Randomize the order of the hyperparameters to get a more 
    # realistic ETA while running the computation.
    # random.shuffle(args)

    if not os.path.exists(DATA_PATH):
        os.mkdir(DATA_PATH)

    if os.path.exists(CSV_PATH):
        data = pd.read_csv(CSV_PATH)
    else:
        data = pd.DataFrame(columns=COLUMNS)
    
    with Pool(8) as pool:
        new_data = list(tqdm(pool.imap(single_playout, args), total=len(args)))
        data = pd.concat([data] + new_data)

    data.to_csv(CSV_PATH, index=False)
    logging.info(f"Wrote results to {CSV_PATH}")



if __name__ == "__main__":
    # logging.info(f"Starting gridsearch")
    generate_data_gridsearch(1)
    plot(CSV_PATH)

