use std::collections::{hash_map::Entry, HashMap, HashSet};

use crate::{
    api_types::{ApiError, ApiResult},
    models::{
        CandidateId, CandidateResultData, PassingCandidateResult, VotingResult, VotingRoundResult,
    },
};
use float_cmp::approx_eq;
use rand::seq::IteratorRandom;

type Vote = Vec<CandidateId>;

struct WeightedVote<'a> {
    weight: f64,
    vote: &'a Vote,
}

type VoteMap<'a> = HashMap<CandidateId, Vec<WeightedVote<'a>>>;

fn get_current_vote_counts_of_candidates<'a>(
    vote_map: &'a VoteMap,
) -> impl Iterator<Item = (&'a CandidateId, f64)> {
    vote_map
        .iter()
        .map(|(id, votes)| (id, votes.iter().map(|v| v.weight).sum()))
}

pub fn calculate_stv_result(
    candidates: Vec<CandidateId>,
    votes: Vec<Vote>,
    number_of_winners: usize,
) -> ApiResult<VotingResult> {
    // TODO sanitize inputs
    let mut round_results: Vec<VotingRoundResult> = vec![];
    let mut winner_count = 0;
    let mut voting_is_finished = false;
    let mut round: usize = 1;

    let valid_votes: Vec<&Vec<CandidateId>> =
        votes.iter().filter(|vote| !vote.is_empty()).collect();
    let valid_vote_count = valid_votes.len();
    let quota = valid_vote_count as f64 / (number_of_winners as f64 + 1.0) + 1.0;

    let mut vote_map: VoteMap = VoteMap::new();

    // Insert empty list of votes for each candidate
    candidates.iter().for_each(|c| {
        vote_map.insert(c.to_owned(), vec![]);
    });

    // Create WeightedVotes from votes and insert them into vote_map
    votes.iter().for_each(|ballot| {
        if let Some(id) = ballot.first() {
            let weighted_votes_of_candidate = vote_map.entry(id.to_owned()).or_default();
            weighted_votes_of_candidate.push(WeightedVote {
                weight: 1.0,
                vote: ballot,
            });
        }
    });

    while !voting_is_finished {
        if round > (candidates.len() + 1) {
            // At least one candidate is removed from the pool on every round,
            // Thus we should never go this far
            return Err(ApiError::VotingAlgorithmError("Too many voting rounds!"));
        }

        let accept_all_candidates = vote_map.len() + winner_count <= number_of_winners;

        let elected_candidates = get_current_vote_counts_of_candidates(&vote_map)
            .filter(|(_, votes)| (*votes >= quota) || accept_all_candidates)
            .map(|(id, _)| id.clone())
            .collect::<HashSet<_>>();

        let round_result: VotingRoundResult = if !elected_candidates.is_empty() {
            transfer_surplus_votes(&mut vote_map, &elected_candidates, quota, round)
        } else {
            drop_one_candidate(&mut vote_map, round)
        }?;

        round_results.push(round_result);
        winner_count += elected_candidates.len();
        round += 1;

        if winner_count == number_of_winners || vote_map.is_empty() {
            println!("FINISH VOTING");
            voting_is_finished = true;
        }
    }

    let winners = round_results
        .iter()
        .flat_map(|res| res.candidate_results.iter().filter(|c| c.is_selected))
        .map(|c| c.data.name.clone())
        .collect::<Vec<_>>();

    Ok(VotingResult {
        round_results,
        winners,
    })
}

fn drop_one_candidate<'a>(vote_map: &'a mut VoteMap, round: usize) -> ApiResult<VotingRoundResult> {
    println!("DROP ONE CANDIDATE");
    let mut vote_counts = get_current_vote_counts_of_candidates(vote_map)
        .map(|(c, v)| (c.to_owned(), v))
        .collect::<Vec<_>>();

    vote_counts.sort_by(|(_, old), (_, new)| new.total_cmp(old));
    let &min_number_of_votes = vote_counts
        .iter()
        .map(|(_, votes)| votes)
        .min_by(|old, new| old.total_cmp(new))
        .ok_or(ApiError::VotingAlgorithmError(
            "Expected to find at least one entry in vote_counts, found none",
        ))?;

    let candidates_with_votes_equal_to_minimum_value = vote_counts
        .iter()
        .filter(|(_, votes)| approx_eq!(f64, min_number_of_votes, *votes, epsilon = 0.000001))
        .collect::<Vec<_>>();

    // If there are multiple candidates with equal votes, choose one at random
    let candidate_to_be_dropped = candidates_with_votes_equal_to_minimum_value
        .clone()
        .into_iter()
        .choose(&mut rand::thread_rng())
        .ok_or(ApiError::VotingAlgorithmError(
            "Expected to find at least one value in candidates_with_votes_equal_to_minimum_value",
        ))?;

    let votes_of_dropped_candidate =
        vote_map
            .remove(&candidate_to_be_dropped.0)
            .ok_or(ApiError::VotingAlgorithmError(
                "Could not find candidate to be dropped in vote_map",
            ))?;

    // Transfer votes
    for vote in votes_of_dropped_candidate {
        let secondary_preference = find_secondary_preference(vote_map, vote.vote);
        if let Some(secondary_preference) = secondary_preference {
            vote_map.get_mut(secondary_preference).unwrap().push(vote);
        }
    }

    let candidate_results = vote_counts
        .iter()
        .filter(|(c, _)| *c != candidate_to_be_dropped.0)
        .map(|(c, v)| PassingCandidateResult {
            data: CandidateResultData {
                name: c.clone(),
                vote_count: v.clone(),
                is_draw: candidates_with_votes_equal_to_minimum_value.len() > 1
                    && candidates_with_votes_equal_to_minimum_value
                        .to_owned()
                        .contains(&&(c.clone(), *v)),
            },
            is_selected: false,
        })
        .collect::<Vec<_>>();

    let dropped_candidate = Some(CandidateResultData {
        name: candidate_to_be_dropped.0.to_owned(),
        vote_count: candidate_to_be_dropped.1,
        is_draw: candidates_with_votes_equal_to_minimum_value.len() > 1
            && candidates_with_votes_equal_to_minimum_value.contains(&candidate_to_be_dropped),
    });

    Ok(VotingRoundResult {
        round: round.try_into().expect("Could not fit rounds into i32!"),
        candidate_results,
        dropped_candidate,
    })
}

fn transfer_surplus_votes(
    vote_map: &mut VoteMap,
    elected_candidates: &HashSet<CandidateId>,
    quota: f64,
    round: usize,
) -> ApiResult<VotingRoundResult> {
    println!("TRANSFER SURPLUS VOTES");
    let mut vote_counts = get_current_vote_counts_of_candidates(vote_map)
        .map(|(c, v)| (c.to_owned(), v))
        .collect::<Vec<_>>();
    vote_counts.sort_by(|(_, old), (_, new)| new.total_cmp(old));

    vote_counts
        .iter()
        .filter(|(c, _)| elected_candidates.contains(c))
        .map(|(c, v)| {
            let surplus = (v - quota).max(0.0); // Limit min value to 0 to prevent negative values from floating point issues
            let votes_to_be_transferred = vote_map.remove(c).ok_or(
                ApiError::VotingAlgorithmError("Could not find elected candidate in voting_map"),
            )?;

            votes_to_be_transferred.into_iter().for_each(|mut vote| {
                vote.weight = (vote.weight / v) * surplus;

                let secondary_preference_votes = find_secondary_preference(vote_map, vote.vote);
                if let Some(secondary_preference) = secondary_preference_votes {
                    vote_map.get_mut(secondary_preference).unwrap().push(vote);
                }
            });
            Ok(())
        })
        .collect::<ApiResult<_>>()?;

    let candidate_results = vote_counts
        .iter()
        .map(|(c, v)| PassingCandidateResult {
            data: CandidateResultData {
                name: (*c).to_owned(),
                vote_count: *v,
                is_draw: false,
            },
            is_selected: elected_candidates.contains(c),
        })
        .collect::<Vec<_>>();

    Ok(VotingRoundResult {
        round: round.try_into().expect("Could not fit rounds into i32!"),
        candidate_results,
        dropped_candidate: None,
    })
}

fn find_secondary_preference<'a>(
    vote_map: &HashMap<String, Vec<WeightedVote<'a>>>,
    vote: &'a Vote,
) -> Option<&'a CandidateId> {
    vote.iter().find(|c| vote_map.contains_key(*c))
}

/* fn collect_candidate_results(
    selected_candidates: &Vec<(CandidateId, f64)>,
    vote_counts: &HashMap<CandidateId, f64>,
) -> Vec<PassingCandidateResult> {
    let winner_results = selected_candidates
        .into_iter()
        .map(|(c_id, votes)| PassingCandidateResult {
            data: CandidateResultData {
                name: c_id.clone(),
                vote_count: *votes,
            },
            is_selected: true,
        })
        .collect::<Vec<_>>();

    let passing_results = vote_counts
        .iter()
        .map(|(id, votes)| PassingCandidateResult {
            data: CandidateResultData {
                name: id.clone(),
                vote_count: *votes,
            },
            is_selected: false,
        })
        .collect::<Vec<_>>();

    [winner_results, passing_results].concat()
} */

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        helpers::calculate_stv_result,
        models::{
            CandidateId, CandidateResultData, PassingCandidateResult, VotingResult,
            VotingRoundResult,
        },
    };

    // TODO sanitize inputs
    /* #[tokio::test(flavor = "multi_thread")]
       async fn test_duplicate_candidate_throws() {
           let candidates = vec!["a".to_string(), "a".to_string()];
           let votes: Vec<Vec<CandidateId>> = vec![];
           let result = calculate_stv_result(candidates, votes, 1);
           assert!(result.is_err())
       }

       #[tokio::test(flavor = "multi_thread")]
       async fn test_duplicate_vote_throws() {
           let candidates = vec!["a".to_string(), "b".to_string()];
           let votes: Vec<Vec<CandidateId>> = vec![vec!["a".to_string(), "a".to_string()]];
           let result = calculate_stv_result(candidates, votes, 1);
           assert!(result.is_err())
       }

       #[tokio::test(flavor = "multi_thread")]
       async fn test_invalid_candidate_throws() {
           let candidates = vec!["a".to_string(), "b".to_string()];
           let votes: Vec<Vec<CandidateId>> = vec![vec!["c".to_string()]];
           let result = calculate_stv_result(candidates, votes, 1);
           assert!(result.is_err())
       }
    */

    #[tokio::test(flavor = "multi_thread")]
    async fn test_single_candidate_is_selected_with_no_votes() {
        let candidates = vec!["a".to_string()];
        let votes: Vec<Vec<CandidateId>> = vec![];
        let result = calculate_stv_result(candidates, votes, 1);

        let expected_result = VotingResult {
            round_results: vec![VotingRoundResult {
                round: 1,
                candidate_results: vec![PassingCandidateResult {
                    data: CandidateResultData {
                        name: "a".to_string(),
                        vote_count: 0.0,
                        is_draw: false,
                    },
                    is_selected: true,
                }],
                dropped_candidate: None,
            }],
            winners: vec!["a".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_single_candidate_is_selected_with_one_vote() {
        let candidates = vec!["a".to_string()];
        let votes: Vec<Vec<CandidateId>> = vec![vec!["a".to_string()]];
        let result = calculate_stv_result(candidates, votes, 1);

        let expected_result = VotingResult {
            round_results: vec![VotingRoundResult {
                round: 1,
                candidate_results: vec![PassingCandidateResult {
                    data: CandidateResultData {
                        name: "a".to_string(),
                        vote_count: 1.0,
                        is_draw: false,
                    },
                    is_selected: true,
                }],
                dropped_candidate: None,
            }],
            winners: vec!["a".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_two_candidates_one_vote() {
        let candidates = vec!["a".to_string(), "b".to_string()];
        let votes: Vec<Vec<CandidateId>> = vec![vec!["a".to_string()]];
        let result = calculate_stv_result(candidates, votes, 1);

        let expected_result = VotingResult {
            round_results: vec![
                VotingRoundResult {
                    round: 1,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "a".to_string(),
                            vote_count: 1.0,
                            is_draw: false,
                        },
                        is_selected: false,
                    }],
                    dropped_candidate: Some(CandidateResultData {
                        name: "b".to_string(),
                        vote_count: 0.0,
                        is_draw: false,
                    }),
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "a".to_string(),
                            vote_count: 1.0,
                            is_draw: false,
                        },
                        is_selected: true,
                    }],
                    dropped_candidate: None,
                },
            ],
            winners: vec!["a".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_two_candidates_two_spots_one_vote() {
        let candidates = vec!["a".to_string(), "b".to_string()];
        let votes: Vec<Vec<CandidateId>> = vec![vec!["a".to_string()]];
        let result = calculate_stv_result(candidates, votes, 2);

        let expected_result = VotingResult {
            round_results: vec![VotingRoundResult {
                round: 1,
                candidate_results: vec![
                    PassingCandidateResult {
                        data: CandidateResultData {
                            name: "a".to_string(),
                            vote_count: 1.0,
                            is_draw: false,
                        },
                        is_selected: true,
                    },
                    PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: 0.0,
                            is_draw: false,
                        },
                        is_selected: true,
                    },
                ],
                dropped_candidate: None,
            }],
            winners: vec!["a".to_string(), "b".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_surplus_votes_no_next_candidate_and_double_transfer() {
        let candidates = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let votes: Vec<Vec<CandidateId>> = vec![
            vec!["a".to_string(), "c".to_string(), "b".to_string()],
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["a".to_string()],
            vec!["b".to_string()],
        ];
        let quota = (5.0 / (2.0 + 1.0)) + 1.0;
        let result = calculate_stv_result(candidates, votes, 2);

        let expected_result = VotingResult {
            round_results: vec![
                VotingRoundResult {
                    round: 1,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 4.0,
                                is_draw: false,
                            },
                            is_selected: true,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 1.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 0.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: None,
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: 1.0 + (4.0 - quota) * (2.0 / 4.0),
                            is_draw: false,
                        },
                        is_selected: false,
                    }],
                    dropped_candidate: Some(CandidateResultData {
                        name: "c".to_string(),
                        vote_count: (4.0 - quota) * (1.0 / 4.0),
                        is_draw: false,
                    }),
                },
                VotingRoundResult {
                    round: 3,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: 1.0 + (4.0 - quota) * ((2.0 / 4.0) + (1.0 / 4.0)),
                            is_draw: false,
                        },
                        is_selected: true,
                    }],
                    dropped_candidate: None,
                },
            ],
            winners: vec!["a".to_string(), "b".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_candidates_over_quota_are_elected() {
        let candidates = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];

        let a_b_votes: Vec<Vec<String>> = std::iter::repeat(vec!["a".to_string(), "b".to_string()])
            .take(15)
            .collect();
        let b_a_votes: Vec<Vec<String>> = std::iter::repeat(vec!["b".to_string(), "c".to_string()])
            .take(11)
            .collect();
        let b_d_votes: Vec<Vec<String>> = std::iter::repeat(vec!["b".to_string(), "d".to_string()])
            .take(1)
            .collect();
        let a_c_votes = std::iter::repeat(vec!["a".to_string(), "c".to_string()])
            .take(1)
            .collect();
        let c_votes = std::iter::repeat(vec!["c".to_string()]).take(1).collect();

        let votes = [a_b_votes, b_a_votes, b_d_votes, a_c_votes, c_votes].concat();
        let _quota = (votes.len() as f64 / (2.0 + 1.0)) + 1.0;
        let result = calculate_stv_result(candidates, votes, 2);

        let expected_first_round = VotingRoundResult {
            round: 1,
            candidate_results: vec![
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "a".to_string(),
                        vote_count: 16.0,

                        is_draw: false,
                    },
                    is_selected: true,
                },
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "b".to_string(),
                        vote_count: 12.0,
                        is_draw: false,
                    },
                    is_selected: true,
                },
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "c".to_string(),
                        vote_count: 1.0,
                        is_draw: false,
                    },
                    is_selected: false,
                },
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "d".to_string(),
                        vote_count: 0.0,
                        is_draw: false,
                    },
                    is_selected: false,
                },
            ],
            dropped_candidate: None,
        };

        match result {
            Ok(res) => assert_eq!(
                res.round_results.first().unwrap().to_owned(),
                expected_first_round
            ),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_gregory_transfer_proportions() {
        let candidates = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let a_b_votes: Vec<Vec<String>> = std::iter::repeat(vec!["a".to_string(), "b".to_string()])
            .take(10)
            .collect();
        let a_c_votes: Vec<Vec<String>> = std::iter::repeat(vec!["a".to_string(), "c".to_string()])
            .take(9)
            .collect();
        let a_votes: Vec<Vec<String>> = std::iter::repeat(vec!["a".to_string()]).take(8).collect();
        let b_votes: Vec<Vec<String>> = std::iter::repeat(vec!["b".to_string()]).take(10).collect();
        let c_votes: Vec<Vec<String>> = std::iter::repeat(vec!["c".to_string()]).take(9).collect();

        let votes: Vec<Vec<CandidateId>> =
            [a_b_votes, a_c_votes, a_votes, b_votes, c_votes].concat();

        let quota = (votes.len() as f64 / (2.0 + 1.0)) + 1.0; // 24
        let result = calculate_stv_result(candidates, votes, 2);

        let expected_result = VotingResult {
            round_results: vec![
                VotingRoundResult {
                    round: 1,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 27.0,
                                is_draw: false,
                            },
                            is_selected: true,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 10.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 9.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: None,
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: 10.0 + (27.0 - quota) * (10.0 / 27.0),
                            is_draw: false,
                        },
                        is_selected: false,
                    }],
                    dropped_candidate: Some(CandidateResultData {
                        name: "c".to_string(),
                        vote_count: 9.0 + (27.0 - quota) * (9.0 / 27.0),
                        is_draw: false,
                    }),
                },
                VotingRoundResult {
                    round: 3,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: 10.0 + (27.0 - quota) * (10.0 / 27.0),
                            is_draw: false,
                        },
                        is_selected: true,
                    }],
                    dropped_candidate: None,
                },
            ],
            winners: vec!["a".to_string(), "b".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_vote_transfer_chain() {
        let candidates = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];

        let votes: Vec<Vec<CandidateId>> = vec![
            vec!["a".to_string()],
            vec!["a".to_string()],
            vec!["a".to_string()],
            vec!["a".to_string()],
            vec!["a".to_string()],
            vec!["a".to_string()],
            vec!["b".to_string()],
            vec!["b".to_string()],
            vec!["b".to_string()],
            vec!["b".to_string()],
            vec!["c".to_string()],
            vec!["c".to_string()],
            vec![
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
                "a".to_string(),
            ],
        ];

        let _quota = (votes.len() as f64 / (1.0 + 1.0)) + 1.0; // 8.5
        let result = calculate_stv_result(candidates, votes, 1);

        let expected_result = VotingResult {
            round_results: vec![
                VotingRoundResult {
                    round: 1,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 6.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 4.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 2.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "d".to_string(),
                        vote_count: 1.0,
                        is_draw: false,
                    }),
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 6.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 4.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "c".to_string(),
                        vote_count: 3.0,
                        is_draw: false,
                    }),
                },
                VotingRoundResult {
                    round: 3,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "a".to_string(),
                            vote_count: 6.0,
                            is_draw: false,
                        },
                        is_selected: false,
                    }],
                    dropped_candidate: Some(CandidateResultData {
                        name: "b".to_string(),
                        vote_count: 5.0,
                        is_draw: false,
                    }),
                },
                VotingRoundResult {
                    round: 4,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "a".to_string(),
                            vote_count: 7.0,
                            is_draw: false,
                        },
                        is_selected: true,
                    }],
                    dropped_candidate: None,
                },
            ],
            winners: vec!["a".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_prodeko_chairman_2024() {
        let candidates = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];

        let votes: Vec<Vec<CandidateId>> = vec![
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "a".to_string(),
                "e".to_string(),
            ],
            vec!["b".to_string(), "d".to_string()],
            vec!["c".to_string(), "a".to_string(), "b".to_string()],
            vec![
                "c".to_string(),
                "a".to_string(),
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
            ],
            vec![
                "d".to_string(),
                "a".to_string(),
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec![
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
                "a".to_string(),
                "d".to_string(),
            ],
            vec![
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
                "a".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "b".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "e".to_string(),
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
            vec![
                "a".to_string(),
                "e".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "b".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "c".to_string(),
                "b".to_string(),
                "a".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec!["b".to_string()],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "e".to_string(),
                "d".to_string(),
            ],
            vec![
                "c".to_string(),
                "b".to_string(),
                "a".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "b".to_string(),
                "e".to_string(),
                "c".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "c".to_string(),
                "a".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "e".to_string(),
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
            ],
            vec![
                "d".to_string(),
                "a".to_string(),
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec![
                "d".to_string(),
                "a".to_string(),
                "e".to_string(),
                "c".to_string(),
                "b".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec!["c".to_string()],
            vec![
                "e".to_string(),
                "a".to_string(),
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
            ],
            vec![
                "e".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "a".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec!["b".to_string()],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "c".to_string(),
                "d".to_string(),
                "b".to_string(),
                "a".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "a".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "d".to_string(),
                "b".to_string(),
                "c".to_string(),
                "a".to_string(),
                "e".to_string(),
            ],
            vec![
                "c".to_string(),
                "a".to_string(),
                "e".to_string(),
                "d".to_string(),
                "b".to_string(),
            ],
            vec![
                "c".to_string(),
                "a".to_string(),
                "e".to_string(),
                "d".to_string(),
                "b".to_string(),
            ],
            vec![
                "c".to_string(),
                "a".to_string(),
                "d".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec!["c".to_string()],
            vec![
                "e".to_string(),
                "d".to_string(),
                "a".to_string(),
                "c".to_string(),
                "b".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "a".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "d".to_string(),
                "c".to_string(),
                "a".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec!["d".to_string(), "b".to_string(), "c".to_string()],
            vec![
                "a".to_string(),
                "c".to_string(),
                "e".to_string(),
                "d".to_string(),
                "b".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "e".to_string(),
                "d".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec!["b".to_string(), "a".to_string(), "d".to_string()],
            vec![
                "e".to_string(),
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
            ],
            vec!["a".to_string()],
            vec!["b".to_string(), "c".to_string(), "a".to_string()],
            vec![
                "d".to_string(),
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "c".to_string(),
            ],
            vec![
                "e".to_string(),
                "b".to_string(),
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
            ],
            vec![
                "c".to_string(),
                "b".to_string(),
                "d".to_string(),
                "a".to_string(),
                "e".to_string(),
            ],
            vec![
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
                "a".to_string(),
            ],
            vec![
                "d".to_string(),
                "b".to_string(),
                "e".to_string(),
                "c".to_string(),
                "a".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
                "a".to_string(),
            ],
            vec![
                "d".to_string(),
                "c".to_string(),
                "a".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "e".to_string(),
                "b".to_string(),
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
            ],
            vec![
                "a".to_string(),
                "c".to_string(),
                "e".to_string(),
                "d".to_string(),
                "b".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
                "a".to_string(),
                "d".to_string(),
            ],
            vec![
                "a".to_string(),
                "c".to_string(),
                "d".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "c".to_string(),
                "b".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "a".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "c".to_string(),
                "b".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "d".to_string(),
                "a".to_string(),
                "e".to_string(),
                "c".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "b".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec!["e".to_string(), "c".to_string(), "a".to_string()],
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "e".to_string(),
                "d".to_string(),
            ],
            vec![
                "a".to_string(),
                "c".to_string(),
                "b".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
            ],
            vec![
                "a".to_string(),
                "d".to_string(),
                "c".to_string(),
                "b".to_string(),
                "e".to_string(),
            ],
            vec![
                "b".to_string(),
                "d".to_string(),
                "c".to_string(),
                "e".to_string(),
                "a".to_string(),
            ],
        ];

        let _quota = (votes.len() as f64 / (1.0 + 1.0)) + 1.0; // 41.5
        let result = calculate_stv_result(candidates, votes, 1);

        let expected_result = VotingResult {
            round_results: vec![
                VotingRoundResult {
                    round: 1,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 34.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 15.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 13.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "d".to_string(),
                                vote_count: 11.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "e".to_string(),
                        vote_count: 8.0,
                        is_draw: false,
                    }),
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 37.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 18.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 14.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "d".to_string(),
                        vote_count: 12.0,
                        is_draw: false,
                    }),
                },
                VotingRoundResult {
                    round: 3,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 42.0,
                                is_draw: false,
                            },
                            is_selected: true,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 21.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 18.0,
                                is_draw: false,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: None,
                },
            ],
            winners: vec!["a".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }
}
