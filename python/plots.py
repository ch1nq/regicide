import os
from re import split
from typing import Dict, List

import pandas as pd
import seaborn as sns
from matplotlib import pyplot as plt
import numpy as np
import scipy

PLOT_PATH = "plots"

def plot(csv_path: str):
    sns.set_theme()
    plt.figure(figsize=(8,4))

    if not os.path.exists(PLOT_PATH):
        os.mkdir(PLOT_PATH)
    data = pd.read_csv(csv_path)

    def human_format(num):
        num = float('{:.3g}'.format(num))
        magnitude = 0
        while abs(num) >= 1000:
            magnitude += 1
            num /= 1000.0
        return '{}{}'.format('{:f}'.format(num).rstrip('0').rstrip('.'), ['', 'K', 'M', 'B', 'T'][magnitude])

    def clean_name(x: str):
        try:
            name, num = x.split("_")
        except ValueError:
            return x
        return f"{name}_{human_format(int(num))}"
    
    def get_playouts(x: str):
        try:
            _, num = x.split("_")
        except ValueError:
            return np.nan
        return int(num)

    data["Agent"] = data["agent"].map(clean_name)
    data["agent_playouts"] = data["agent"].map(get_playouts)

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

    plt.title("Score distribution of MCTS agents in 3 player games")
    subset = make_subset(
        data,
        "agent",
        {"player_count": [3], "agent": [f"MCTS_{10 ** i}" for i in range(6)]},
    )
    sns.kdeplot(
        data=subset,
        x="score",
        hue="agent",
        # fill=True,
        # linewidth=0,
        linewidth=1,
        bw_adjust=1.2,
        common_norm=False,
        # palette="crest",
        # alpha=0.3,
        clip=(0, 12),
    ) 
    # sns.histplot(
    #     data=subset,
    #     x="score",
    #     hue="agent",
    #     fill=True,
    #     # linewidth=0,
    #     common_norm=False,
    #     # palette="crest",
    #     # alpha=0.3,
    #     stat="density",
    #     # element="step",
    #     element="poly",
    #     # fill=False,
    # )
    plt.tight_layout()
    plt.savefig(os.path.join(PLOT_PATH, "hist_agents.png"), dpi=300)
    plt.clf()


    plt.title("Performance of MCTS_1000 agent in across different player counts")
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


    plt.title("Score distribution of the MCTS 1K agent across UCT policy variations in 3-player games")
    subset = make_subset(data, "tree_policy", {"agent": ["MCTS_1000"], "tree_policy": [0,1,2,3,4]})
    subset = subset[subset["player_count"] == 3]
    bw = 1
    g = sns.histplot(
        data=subset,    
        x="score",
        # discrete=True,
        element="poly",
        # fill=False,
        # multiple="dodge",
        hue="tree_policy",
        bins=12,
        # binwidth=bw,
        binrange=(-0.5*bw,12+0.5*bw),
        stat="proportion",
        common_norm=False,
    )
    # sns.kdeplot(
    #     data=subset, 
    #     x="score", 
    #     hue="tree_policy", 
    #     common_norm=False, 
    #     clip=(0,12)
    # )
    plt.savefig(os.path.join(PLOT_PATH, "hist_policies.png"), dpi=300)
    plt.clf()



    plt.title("Win rate of the MCTS 1K agent across UCT policy variations and player counts")
    subset = make_subset(data, "tree_policy", {"agent": ["MCTS_1000"], "tree_policy": [0,1,2,3,4]})
    subset["win"] = subset["score"] == 12
    subset = subset.groupby(["tree_policy", "player_count"], sort=False).mean().reset_index()
    subset = subset.pivot("tree_policy", "player_count", "win")
    sns.heatmap(
        data=subset, 
        cmap=sns.cubehelix_palette(
            start=2.6, rot=0, dark=0.1, light=.95, reverse=True, as_cmap=True
        ),
        annot=True, 
        vmin=0, vmax=1,
        fmt=".2f",
    )
    plt.tight_layout()
    plt.savefig(os.path.join(PLOT_PATH, "heatmap_policy_player_count_winrate.png"), dpi=300)
    plt.clf()


    plt.title("Win rate MCTS agents across UCT policy variations in 3-player games")
    subset = data[data["player_count"] == 3]
    subset["win"] = subset["score"] == 12
    subset = subset.groupby(["agent", "tree_policy"], sort=False).mean().reset_index()
    subset = subset.pivot("agent", "tree_policy", "win")
    sns.heatmap(
        data=subset, 
        cmap=sns.cubehelix_palette(
            start=2.6, rot=0, dark=0.1, light=.95, reverse=True, as_cmap=True
        ),
        annot=True, 
        vmin=0, vmax=1,
        fmt=".2f",
    )
    plt.tight_layout()
    plt.savefig(os.path.join(PLOT_PATH, "heatmap_policy_agent_winrate.png"), dpi=300)
    plt.clf()
        

    plt.title("Mean score across agents and player counts")
    subset = data.groupby(["agent", "player_count"], sort=False).mean().reset_index()
    subset = subset.pivot("agent", "player_count", "score")
    
    # Make "Random" the first row
    subset = subset.loc[["Random"] + [i for i in subset.index if i != "Random"]]
    
    sns.heatmap(
        data=subset, 
        cmap=sns.cubehelix_palette(start=2.6, rot=0, dark=0.1, light=.95, reverse=True, as_cmap=True),
        annot=True, 
        vmin=0, vmax=12,
        fmt=".2f",
    )
    plt.xlabel("Player count")
    plt.ylabel("Agent")
    plt.tight_layout()
    plt.savefig(os.path.join(PLOT_PATH, "heatmap.png"), dpi=300)
    plt.clf()


    plt.title("Win rate across agents and player counts")
    subset = data
    subset["win"] = subset["score"] == 12
    subset = subset.groupby(["agent", "player_count"], sort=False).mean().reset_index()
    subset = subset.pivot("agent", "player_count", "win")
    
    # Make "Random" the first row
    subset = subset.loc[["Random"] + [i for i in subset.index if i != "Random"]]
    
    sns.heatmap(
        data=subset, 
        cmap=sns.cubehelix_palette(start=2.6, rot=0, dark=0.1, light=.95, reverse=True, as_cmap=True),
        annot=True, 
        vmin=0, vmax=1,
        fmt=".2f",
    )
    plt.xlabel("Player count")
    plt.ylabel("Agent")
    plt.tight_layout()
    plt.savefig(os.path.join(PLOT_PATH, "heatmap_winrate.png"), dpi=300)
    plt.clf()


    plt.title("Number of observations by player count and agents")
    subset = data.groupby(["agent", "player_count"], sort=False).count().reset_index()
    subset = subset.pivot("agent", "player_count", "score")
    
    # Make "Random" the first row
    subset = subset.loc[["Random"] + [i for i in subset.index if i != "Random"]]
    
    sns.heatmap(
        data=subset, 
        cmap=sns.cubehelix_palette(start=2.6, rot=0, dark=0.1, light=.95, reverse=True, as_cmap=True),
        annot=True, 
        fmt=".4g",
    )
    plt.xlabel("Player count")
    plt.ylabel("Agent")
    plt.tight_layout()
    plt.savefig(os.path.join(PLOT_PATH, "heatmap_count.png"), dpi=300)
    plt.clf()


    plt.title("Mean score across MCTS agents in 3-player games")
    subset = data[data["player_count"] == 3]
    subset = subset[subset["agent"].str.contains("MCTS")]
    subset["agent_playouts"] = subset["agent_playouts"].apply(np.log10)
    subset_mean = subset.groupby("agent_playouts").mean().reset_index()

    def func(x, a, b, c):
        return b + a * np.log(x + c) 
        # return b + a * x 

    param, cov = scipy.optimize.curve_fit(func, subset_mean["agent_playouts"], subset_mean["score"], (0,12,0.1))
    xs = np.linspace(0,8,100);
    ys = [func(x, param[0], param[1], param[2]) for x in xs]
    
    g = sns.scatterplot(
        data=subset, 
        x="agent_playouts", 
        y="score",
        alpha=0.1,
        label="Observation",
    )
    g = sns.scatterplot(
        ax=g, 
        data=subset_mean,
        x="agent_playouts", 
        y="score",
        label="Agent mean",
    )
    g = sns.lineplot(
        ax=g, 
        x=xs, 
        y=ys, 
        color="#333",
        alpha=0.7,
        label="Regression to $\log(x)$"
    )
    
    g.set_xticklabels([f"$10^{{{int(exp)}}}$" for exp in g.get_xticks()])
    g.set_xlabel("Playouts in MCTS agents")
    g.set_ylabel("Score")

    plt.xlim(-0.2,8)
    plt.ylim((0,12.3))
    plt.tight_layout()
    plt.savefig(os.path.join(PLOT_PATH, "score_regression.png"), dpi=300)
    plt.clf()
    
    subset = data
    subset = subset[subset["player_count"] == 3]
    subset = subset[subset["agent"].str.contains("MCTS")]
    subset = subset.sort_values(by="agent_playouts")
    subset["win"] = subset["score"].map(lambda x: int(x == 12))
    subset["loss"] = 1 - subset["win"]
    subset = subset.groupby(["Agent"], sort=False)[["win", "loss"]].mean()
    subset.plot.bar(stacked=True)

    plt.title("Win/loss rate of MCTS agents in 3-player games")
    plt.xlabel("Agent")
    plt.ylabel("Win/loss rate")
    plt.tick_params(axis='x', rotation=0)
    plt.tight_layout()
    plt.savefig(os.path.join(PLOT_PATH, "win_rate.png"), dpi=300)
    plt.clf()


    subset = make_subset(data, "player_count", {})
    subset = subset[subset["agent"].str.contains("MCTS")]
    subset = subset.sort_values(by=["agent_playouts", "player_count"])
    g = sns.FacetGrid(
        subset, 
        col="player_count",
        row="Agent",
        hue="tree_policy",
        margin_titles=True,
        dropna=True,
        height=2,
    )
    # g.map(
    #     sns.kdeplot,
    #     "score",
    #     linewidth=1,
    #     bw_adjust=1.5,
    #     common_norm=False,
    #     alpha=1,
    #     clip=((0, 12)),
    # )
    g.map(
        sns.histplot,
        "score",
        stat="proportion",
        discrete=True,
        binrange=(0,12),
    )
    g.add_legend()

    # plt.set_title("Score distribution across player counts, agents and UCT policies")
    plt.savefig(os.path.join(PLOT_PATH, "combined_performance_plot.png"), dpi=300, )
    plt.clf()
