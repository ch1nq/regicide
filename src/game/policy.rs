use mcts::{
    tree_policy::{PolicyRng, TreePolicy},
    MoveInfo, SearchHandle, MCTS,
};

#[derive(Clone, Debug)]
pub enum MyPolicy {
    UCTBase { exploration_constant: f64 },
    UCTVariation3 { max_score: f64, delta: f64 },
    UCTVariation4 { max_score: f64 },
}

impl<Spec: MCTS<TreePolicy = Self>> TreePolicy<Spec> for MyPolicy {
    type ThreadLocalData = PolicyRng;
    type MoveEvaluation = ();

    fn choose_child<'a, MoveIter>(
        &self,
        moves: MoveIter,
        mut handle: SearchHandle<Spec>,
    ) -> &'a MoveInfo<Spec>
    where
        MoveIter: Iterator<Item = &'a MoveInfo<Spec>> + Clone,
    {
        // adjusted total visits
        #[allow(non_snake_case)]
        let N_i = moves.clone().map(|x| x.visits()).sum::<u64>() as f64 + 1.0;

        handle
            .thread_data()
            .policy_data
            .select_by_key(moves, |mov| {
                // number of node obervations
                let n_i = mov.visits() as f64;

                // mean action value
                let mu_i = mov.sum_rewards() as f64 / n_i as f64;

                // Aliases to simplify math below
                let sqrt = f64::sqrt;
                let ln = f64::ln;

                match self {
                    _ if n_i == 0.0 => std::f64::INFINITY,
                    MyPolicy::UCTBase {
                        exploration_constant,
                    } => {
                        let explore_term = sqrt(2.0 * ln(N_i) / n_i);
                        mu_i + exploration_constant * explore_term
                    }
                    MyPolicy::UCTVariation3 { max_score, delta } => {
                        let numerator = (1.0 + 1.0 / n_i) * ln(sqrt(n_i + 1.0) / delta);
                        let denominator = 2.0 * n_i;
                        mu_i + max_score * sqrt(numerator / denominator)
                    }
                    MyPolicy::UCTVariation4 { max_score } => {
                        let numerator = ln(N_i) + (3.0 * ln(ln(N_i) + 1.0));
                        let denominator = 2.0 * n_i;
                        mu_i + max_score * sqrt(numerator / denominator)
                    }
                }
            })
            .unwrap()
    }
}
