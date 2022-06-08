from encodings import utf_8
import os
import random
from multiprocessing import Pool
from typing import Any, Dict, Optional
from datetime import datetime

import numpy as np
import pandas as pd
import regicide
from tqdm import tqdm

from plots import plot

DATA_PATH = "data"
# CSV_PATH = os.path.join(DATA_PATH, f"games_gridsearch_{datetime.now()}.csv")
CSV_PATH = os.path.join(DATA_PATH, f"games_gridsearch_heatmap.csv")
TEX_PATH = os.path.join("plots", "results.tex")
COLUMNS = ["score", "agent", "player_count", "tree_policy", "deterministic_samples"]


def to_latex(tex_path):
    data = load_data()

    def human_format(num):
        num = float("{:.3g}".format(num))
        magnitude = 0
        while abs(num) >= 1000:
            magnitude += 1
            num /= 1000.0
        return "{}{}".format(
            "{:f}".format(num).rstrip("0").rstrip("."),
            ["", "K", "M", "B", "T"][magnitude],
        )

    def clean_name(x: str):
        try:
            name, num = x.split("_")
        except ValueError:
            return x
        return f"{name} {human_format(int(num))}"

    def get_playouts(x: str):
        try:
            _, num = x.split("_")
        except ValueError:
            return np.nan
        return int(num)

    data["Agent"] = data["agent"].map(clean_name)
    data["agent_playouts"] = data["agent"].map(get_playouts)
    data["Player count"] = data["player_count"]
    data["Policy"] = data["tree_policy"]
    data["Win rate"] = data["score"] == 12
    data["Mean score"] = data["score"]
    data = data.sort_values(
        by=["agent_playouts", "player_count", "tree_policy"], na_position="first"
    )
    data["Count"] = [1] * len(data)

    data = data.groupby(["Agent", "Player count", "Policy"], sort=False).agg(
        {
            "Win rate": "mean",
            "Mean score": "mean",
            "Count": "count",
        }
    )
    # data = data.reset_index()
    (
        data.style.format(precision=2)
        .highlight_max(subset=["Win rate", "Mean score"], props="bfseries:;")
        .to_latex(
            tex_path,
            hrules=True,
            clines="skip-last;data",
            position_float="centering",
            caption="""
                Experimental results showing aggregated statitics of
                simulated games. The maximum values in Mean score and Win rate are highlighted in bold.""",
            label="table:results",
        )
    )


def single_playout(kwargs: Dict[str, Any]) -> pd.DataFrame:
    mcts_playouts = kwargs["mcts_playouts"]
    player_count = kwargs["player_count"]
    policy_variation = kwargs["policy_variation"]
    num_threads = kwargs.get("num_threads", 1)
    deterministic_samples = kwargs["deterministic_samples"]
    seed = kwargs.get("seed", None)
    agent = f"MCTS_{mcts_playouts}"

    gamers = [
        regicide.players.MCTSPlayer(
            playouts=mcts_playouts,
            num_threads=num_threads,
            use_heuristics=False,
            policy_variation=policy_variation,
            deterministic_samples=deterministic_samples,
        ),
    ] * player_count

    game = regicide.RegicideGame(gamers, seed)
    game.playout()

    return pd.DataFrame(
        [
            [
                game.reward(),
                agent,
                player_count,
                policy_variation,
                deterministic_samples,
            ]
        ],
        columns=COLUMNS,
    )


def load_data():
    if not os.path.exists(DATA_PATH):
        os.mkdir(DATA_PATH)

    if os.path.exists(CSV_PATH):
        data = pd.read_csv(CSV_PATH)
    else:
        data = pd.DataFrame(columns=COLUMNS)

    return data


def random_playout(samples):
    data = load_data()

    for _ in tqdm(range(samples)):
        for player_count in [1, 2, 3, 4]:
            gamers = [
                regicide.players.RandomPlayer(),
            ] * player_count

            game = regicide.RegicideGame(gamers, None)
            game.playout()

            row = pd.DataFrame(
                [[game.reward(), "Random", player_count, np.nan, np.nan]], columns=COLUMNS
            )
            data = pd.concat([data, row])

    data.to_csv(CSV_PATH, index=False)


def generate_data_gridsearch(samples, threads_per_player) -> pd.DataFrame:
    args = []
    for _ in range(samples):
        seed = random.randint(0, 1_000_000)
        for policy_variation in [0]:
            for player_count in [3]:
            # for player_count in [3]:
                # exponents = [0] * 32 + [1] * 16 + [2] * 8 + [3] * 4 + [4] * 2 + [5] * 1
                # exponents = sum([[6 - i] * 2 ** (i) for i in range(7)], [])
                # exponents = [5] * ((500 - 384) // 4)
                exponents = [0,1,2,3,4,5,6]
                # exponents = [6]
                for mcts_playouts in [round(10**exp) for exp in exponents]:
                    # for mcts_playouts in [round(10 ** (i/10.0)) for i in range(1,40)]:
                    d_samples = [10**exp for exp in exponents if 10**exp*mcts_playouts == 1e+6]
                    for deterministic_samples in d_samples:
                        args.append(
                            {
                                "player_count": player_count,
                                "mcts_playouts": mcts_playouts,
                                "num_threads": threads_per_player,
                                "policy_variation": policy_variation,
                                "deterministic_samples": deterministic_samples,
                                "seed": seed,
                            }
                        )


    with Pool(6) as pool:
        new_data = list(tqdm(pool.imap(single_playout, args), total=len(args)))
    # new_data = [single_playout(a) for a in tqdm(args)]
    
        data = load_data()
        data = pd.concat([data] + new_data)

    data.to_csv(CSV_PATH, index=False)


def compare_playouts_before_expansion():
    data = load_data()
    def get_playouts(x: str):
        try:
            _, num = x.split("_")
        except ValueError:
            return np.nan
        return int(num)
    data["win"] = data["score"] == 12
    data["playouts"] = data["agent"].map(get_playouts)
    data = data.groupby("agent").mean()
    print(data)


if __name__ == "__main__":
    # random_playout(1000)
    # while True:
    #     generate_data_gridsearch(1, 6)
    compare_playouts_before_expansion()
    plot(CSV_PATH)
    # to_latex(TEX_PATH)
