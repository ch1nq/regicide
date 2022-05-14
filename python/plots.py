import os
from typing import Dict, List

import pandas as pd
import seaborn as sns
from matplotlib import pyplot as plt

PLOT_PATH = "plots"

def plot(csv_path: str):
    sns.set_theme()
    if not os.path.exists(PLOT_PATH):
        os.mkdir(PLOT_PATH)
    data = pd.read_csv(csv_path)

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
    
    subset = make_subset(data, "tree_policy", {"agent": ["MCTS_1000"], "tree_policy": [0,1,2,3,4]})
    bw = 1
    sns.histplot(
        data=subset,    
        x="score",
        # discrete=True,
        element="poly",
        # multiple="dodge",
        hue="tree_policy",
        bins=12,
        binwidth=bw,
        binrange=(-0.5*bw,12+0.5*bw),
        # kde=True,
        # stat="density",
        common_norm=True,
    )
    plt.savefig(os.path.join(PLOT_PATH, "hist_policies.png"), dpi=300)
    plt.clf()
    
    # Performance of different MCTS agents across different player counts
    subset = data.groupby(["agent", "player_count"]).mean().reset_index()
    subset = subset.pivot("agent", "player_count", "score")
    sns.heatmap(data=subset, cmap="crest_r")
    plt.savefig(os.path.join(PLOT_PATH, "heatmap.png"), dpi=300)
    plt.clf()



    #           agent           players         policy          
    # agent     agent     
    # players   players/agent   players
    # policy    policy/agent    policy/players  policy
    
    sns.jointplot(data=data, x="score", y="tree_policy", hue="agent")
    plt.savefig(os.path.join(PLOT_PATH, "jointplot.png"), dpi=300)
    plt.clf()

    subset = make_subset(data, "player_count", {})
    g = sns.FacetGrid(
        data, 
        col="player_count",
        row="agent",
        hue="tree_policy",
        margin_titles=True,
        dropna=True,
        height=2,
    )
    g.map(
        sns.kdeplot,
        "score",
        linewidth=1,
        bw_adjust=1,
        common_norm=False,
        alpha=1,
        clip=((0, 12)),
    )
    g.add_legend()
    
    # sns.catplot(
    #     data=data, 
    #     x="agent", 
    #     y="score", 
    #     hue="tree_policy", 
    #     kind="violin",
    # )

    plt.savefig(os.path.join(PLOT_PATH, "combined_performance_plot.png"), dpi=300)
    plt.clf()
