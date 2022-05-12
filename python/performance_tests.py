import regicide
from tqdm import tqdm
import seaborn as sns
from matplotlib import pyplot as plt
import os
import pandas as pd
from typing import Dict, List, Optional
from multiprocessing import Pool, Queue

DATA_PATH = "data"
PLOT_PATH = "plots"
CSV_PATH = os.path.join(DATA_PATH, "games_gridsearch.csv")
COLUMNS = ["score", "agent", "player_count", "tree_policy"]

def single_playout(kwargs) -> pd.DataFrame:
    mcts_playouts = kwargs["mcts_playouts"]
    player_count = kwargs["player_count"]
    policy_variation = kwargs["policy_variation"]
    agent = f"MCTS_{mcts_playouts}"
    
    gamers = [
        regicide.players.MCTSPlayer(
            playouts=mcts_playouts,
            num_threads=8,
            use_heuristics=False,
            policy_variation=policy_variation
        ),
    ] * player_count

    game = regicide.RegicideGame(gamers, None)
    game.playout()

    return pd.DataFrame([[game.reward(), agent, player_count, policy_variation]], columns=COLUMNS)


def generate_data(kwargs, queue=None) -> Optional[pd.DataFrame]:
    data = pd.DataFrame(columns=COLUMNS)
    
    for _ in tqdm(range(kwargs["samples"])):
        row = single_playout(kwargs)
        data = pd.concat([data, row])
    
    if queue is None:
        return data
    else:
        queue.put(data)


def generate_data_gridsearch(samples) -> pd.DataFrame:
    args = []
    for mcts_playouts in [10 ** i for i in range(1, 5)]:
        for player_count in range(1, 5):
            for policy_variation in [0,3,4]:
                for _ in range(samples):
                    args.append({
                        "player_count": player_count,
                        "mcts_playouts": mcts_playouts,
                        "samples": samples,
                        "policy_variation": policy_variation,
                    })

    if not os.path.exists(DATA_PATH):
        os.mkdir(DATA_PATH)

    if os.path.exists(CSV_PATH):
        data = pd.read_csv(CSV_PATH)
    else:
        data = pd.DataFrame(columns=COLUMNS)

    with Pool(8) as pool:
        new_data = list(tqdm(pool.imap(generate_data, args), total=len(args)))
        data = pd.concat([data] + new_data)

    data.to_csv(CSV_PATH, index=False)


def plot():
    sns.set_theme()
    if not os.path.exists(PLOT_PATH):
        os.mkdir(PLOT_PATH)
    data = pd.read_csv(CSV_PATH)

    def make_subset(
        data: pd.DataFrame, hue: str, filter: Dict[str, List]
    ) -> pd.DataFrame:
        """Prepares a subset of data to be plotted"""
        subset = data
        for key, values in filter.items():
            subset = subset.loc[data[key].isin(values)]
        subset = subset.sort_values(by=hue)
        subset[hue] = subset[hue].astype(str)
        return subset

    # Performance of different agents in 3 player games
    subset = make_subset(
        data,
        "agent",
        {"player_count": [3], "agent": [f"MCTS_{10 ** i}" for i in range(1, 5)]},
    )
    sns.kdeplot(
        data=subset,
        x="score",
        hue="agent",
        # fill=True,
        # linewidth=0,
        linewidth=1,
        bw_adjust=1.5,
        common_norm=False,
        # palette="crest",
        alpha=1,
        clip=(0, 12),
    )

    plt.savefig(os.path.join(PLOT_PATH, "hist_agents.png"), dpi=300)
    plt.clf()

    # Performance of MCTS_1000 agent in across different player counts
    subset = make_subset(data, "player_count", {"agent": ["MCTS_1000"]})
    sns.kdeplot(
        data=subset,
        x="score",
        hue="player_count",
        # fill=True,
        linewidth=1,
        bw_adjust=1.5,
        common_norm=False,
        alpha=1.0,
        # palette="crest",
        clip=(0, 12),
    )
    plt.savefig(os.path.join(PLOT_PATH, "hist_player_counts.png"), dpi=300)
    plt.clf()

    # Performance of MCTS_1000 agent in across different player counts
    subset = make_subset(data, "tree_policy", {"agent": ["MCTS_1000"], "tree_policy": [0,1,2,3,4]})
    sns.histplot(
        data=subset,
        x="score",
        hue="tree_policy",
        # discrete=True,
        # element="step",
        multiple="dodge",
        kde=True,
        stat="density",
        common_norm=False,
        # common_norm=False,
    )
    # sns.kdeplot(
    #     data=subset,
    #     x="score",
    #     hue="tree_policy",
    #     linewidth=1,
    #     bw_adjust=1.5,
    #     common_norm=False,
    #     alpha=1.0,
    #     clip=(0, 12),
    # )
    plt.savefig(os.path.join(PLOT_PATH, "hist_policies.png"), dpi=300)
    plt.clf()

    # Performance of different MCTS agents across different player counts
    subset = data.groupby(["agent", "player_count"]).mean().reset_index()
    subset = subset.pivot("agent", "player_count", "score")
    sns.heatmap(data=subset, cmap="crest_r")
    plt.savefig(os.path.join(PLOT_PATH, "heatmap.png"), dpi=300)
    plt.clf()

    subset = make_subset(data, "player_count", {})
    g = sns.FacetGrid(subset, col="agent", hue="player_count", sharey=False)
    g.map(
        sns.kdeplot,
        "score",
        linewidth=1,
        bw_adjust=1.5,
        common_norm=False,
        alpha=1,
        clip=((0, 12)),
    )

    plt.savefig(os.path.join(PLOT_PATH, "combined_performance_plot.png"), dpi=300)
    plt.clf()


if __name__ == "__main__":
    # generate_data_gridsearch(10)
    # plot()

    samples = 100
    results = generate_data({
        "player_count": 3,
        "mcts_playouts": 1000,
        "samples": samples,
        "policy_variation": 0,
    })
    print(results["score"].mean())
    results = generate_data({
        "player_count": 3,
        "mcts_playouts": 1000,
        "samples": samples,
        "policy_variation": 1,
    })
    print(results["score"].mean())
