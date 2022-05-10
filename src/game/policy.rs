use mcts::{
    tree_policy::{PolicyRng, TreePolicy},
    MoveInfo, SearchHandle, MCTS,
};

#[derive(Clone, Debug)]
pub enum MyPolicy {
    UCTBase { exploration_constant: f64 },
    UCTVariation2,
    UCTVariation4 { max_score: u64 },
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

                match self {
                    MyPolicy::UCTBase {
                        exploration_constant,
                    } => {
                        if n_i == 0.0 {
                            std::f64::INFINITY
                        } else {
                            let explore_term = (2.0 * N_i.ln() / n_i).sqrt();
                            mu_i + exploration_constant * explore_term
                        }
                    }
                    MyPolicy::UCTVariation4 { max_score } => {
                        if n_i == 0.0 {
                            std::f64::INFINITY
                        } else {
                            mu_i + *max_score as f64
                                * (N_i.ln() + 3.0 * N_i.ln().ln() / (2.0 * n_i))
                        }
                    }
                    MyPolicy::UCTVariation2 => todo!(),
                }
            })
            .unwrap()
    }
}
