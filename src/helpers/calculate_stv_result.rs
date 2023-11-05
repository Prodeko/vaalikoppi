use std::collections::HashMap;

use crate::{
    api_types::{ApiError, ApiResult},
    models::{
        CandidateId, CandidateResultData, PassingCandidateResult, VotingResult, VotingRoundResult,
    },
};
use float_cmp::approx_eq;
use rand::seq::IteratorRandom;

pub fn calculate_stv_result(
    candidates: Vec<CandidateId>,
    votes: Vec<Vec<CandidateId>>,
    number_of_winners: usize,
) -> ApiResult<VotingResult> {
    // TODO sanitize inputs
    let mut round_results: Vec<VotingRoundResult> = vec![];
    let mut winner_count = 0;
    let mut voting_is_finished = false;
    let mut round = 1;

    let valid_votes: Vec<&Vec<CandidateId>> =
        votes.iter().filter(|vote| !vote.is_empty()).collect();
    let valid_vote_count = valid_votes.len();
    let quota = valid_vote_count as f64 / (number_of_winners as f64 + 1.0) + 1.0;

    let mut vote_counts: HashMap<CandidateId, f64> = HashMap::new();

    // Initialize vote_counts map with 0 votes for all candidates
    for candidate in candidates.iter() {
        vote_counts.insert(candidate.to_owned(), 0.0);
    }

    // Count first preference votes
    votes.iter().try_for_each(|ballot| {
        if let Some(id) = ballot.first() {
            let existing_count = vote_counts.get_mut(id).ok_or(ApiError::InvalidInput)?;
            *existing_count += 1.0;
        }
        Ok::<(), ApiError>(())
    })?;

    while !voting_is_finished {
        if round > 10000 {
            return Err(ApiError::VotingAlgorithmError);
        }

        let accept_all_candidates = vote_counts.len() + winner_count <= number_of_winners;

        let mut selected_candidates_with_surplus_votes = vote_counts
            .iter()
            .filter(|(_, votes)| (*votes >= &quota) || accept_all_candidates)
            .map(|(id, votes)| (id.clone(), *votes))
            .collect::<Vec<_>>();
        // Sort in descending order
        selected_candidates_with_surplus_votes.sort_by(|(_, old), (_, new)| new.total_cmp(&old));

        winner_count += selected_candidates_with_surplus_votes.len();

        selected_candidates_with_surplus_votes
            .iter()
            .for_each(|(id, _)| {
                vote_counts.remove(id);
            });

        // Transfer surplus votes to secondary preferences if they exist
        let selected_candidates_without_transferred_surplus_votes =
            selected_candidates_with_surplus_votes
                .iter()
                .map(|(c, v)| {
                    let surplus_votes = v - quota;

                    let clone = vote_counts.clone();

                    let secondary_options =
                        find_secondary_preferences(&clone, &votes, c).collect::<Vec<_>>();

                    let portion_of_vote = surplus_votes / secondary_options.len() as f64;
                    let non_null_secondary_options = secondary_options.into_iter().flatten();

                    let mut total_surplus_tranferred = 0.0;

                    non_null_secondary_options.into_iter().for_each(|c_id| {
                        let count = vote_counts.entry(c_id.to_string()).or_insert(0.0);
                        *count += portion_of_vote;
                        total_surplus_tranferred += portion_of_vote;
                    });

                    (c.clone(), v - total_surplus_tranferred)
                })
                .collect::<Vec<_>>();

        if winner_count == number_of_winners || vote_counts.is_empty() {
            println!("FINISH VOTING");
            let candidate_results = collect_candidate_results(
                &selected_candidates_without_transferred_surplus_votes,
                &vote_counts,
            );
            let round_result = VotingRoundResult {
                round,
                candidate_results,
                dropped_candidate: None,
            };

            round_results.push(round_result);
            voting_is_finished = true;
            Ok(())
        }
        // Drop candidate
        else if selected_candidates_with_surplus_votes.is_empty() {
            println!("DROP CANDIDATE");
            let min_number_of_votes = vote_counts
                .iter()
                .map(|(_, votes)| *votes)
                .min_by(|old, new| old.total_cmp(new))
                .ok_or(ApiError::VotingAlgorithmError)?;

            let mut clone = vote_counts.clone();
            let clone2 = vote_counts.clone();

            let candidate_to_be_dropped = clone2
                .iter()
                .filter(|(_, &votes)| {
                    approx_eq!(f64, min_number_of_votes, votes, epsilon = 0.000001)
                })
                .choose(&mut rand::thread_rng())
                .ok_or(ApiError::VotingAlgorithmError)?;

            println!("to be dropped: {:?}", candidate_to_be_dropped);
            vote_counts.remove(candidate_to_be_dropped.0);
            clone.remove(candidate_to_be_dropped.0);

            let secondary_preferences =
                find_secondary_preferences(&clone, &votes, candidate_to_be_dropped.0)
                    .collect::<Vec<_>>();
            println!("SECONDARY PREFERENCES: {:?}", secondary_preferences);
            let portion_of_votes = candidate_to_be_dropped.1 / secondary_preferences.len() as f64;
            let non_null_secondary_preferences = secondary_preferences.into_iter().flatten();

            let candidate_results = collect_candidate_results(
                &selected_candidates_without_transferred_surplus_votes,
                &vote_counts,
            );

            let dropped_candidate = Some(CandidateResultData {
                name: candidate_to_be_dropped.0.to_owned(),
                vote_count: *candidate_to_be_dropped.1,
            });

            let round_result = VotingRoundResult {
                round,
                candidate_results,
                dropped_candidate,
            };
            round_results.push(round_result);
            round += 1;

            non_null_secondary_preferences
                .map(|c| {
                    vote_counts
                        .get_mut(c)
                        .map(|count| *count += portion_of_votes)
                        .ok_or(ApiError::InternalServerError)
                })
                .collect::<ApiResult<Vec<_>>>()
                .map(|_| ())
        } else {
            Ok(())
        }?;
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

fn find_secondary_preferences<'a>(
    vote_counts: &'a HashMap<CandidateId, f64>,
    votes: &'a Vec<Vec<CandidateId>>,
    id: &'a CandidateId,
) -> impl Iterator<Item = Option<&'a String>> {
    votes
        .iter()
        .filter(move |v| v.first().map(|f| f == id).unwrap_or(false))
        .map(|v| v.iter().find(|c| vote_counts.contains_key(*c)))
}

fn collect_candidate_results(
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
}

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
                        },
                        is_selected: false,
                    }],
                    dropped_candidate: Some(CandidateResultData {
                        name: "b".to_string(),
                        vote_count: 0.0,
                    }),
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "a".to_string(),
                            vote_count: 1.0,
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
                        },
                        is_selected: true,
                    },
                    PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: 0.0,
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
        ];
        let quota = (4.0 / (2.0 + 1.0)) + 1.0;
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
                            },
                            is_selected: true,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 0.0,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 0.0,
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
                            vote_count: (4.0 - quota) * (2.0 / 4.0),
                        },
                        is_selected: false,
                    }],
                    dropped_candidate: Some(CandidateResultData {
                        name: "c".to_string(),
                        vote_count: (4.0 - quota) * (1.0 / 4.0),
                    }),
                },
                VotingRoundResult {
                    round: 3,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: (4.0 - quota) * ((2.0 / 4.0) + (1.0 / 4.0)),
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

        let votes = [a_b_votes, b_a_votes, b_d_votes, a_c_votes].concat();
        let _quota = (votes.len() as f64 / (2.0 + 1.0)) + 1.0;
        let result = calculate_stv_result(candidates, votes, 2);

        let expected_first_round = VotingRoundResult {
            round: 1,
            candidate_results: vec![
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "a".to_string(),
                        vote_count: 16.0,
                    },
                    is_selected: true,
                },
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "b".to_string(),
                        vote_count: 12.0,
                    },
                    is_selected: true,
                },
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "c".to_string(),
                        vote_count: 0.0,
                    },
                    is_selected: false,
                },
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "d".to_string(),
                        vote_count: 0.0,
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
        let c_votes: Vec<Vec<String>> = std::iter::repeat(vec!["c".to_string()]).take(10).collect();

        let votes: Vec<Vec<CandidateId>> =
            [a_b_votes, a_c_votes, a_votes, b_votes, c_votes].concat();

        let quota = (votes.len() as f64 / (1.0 + 1.0)) + 1.0; // 24.5
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
                            },
                            is_selected: true,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 10.0,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 10.0,
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
                        },
                        is_selected: false,
                    }],
                    dropped_candidate: Some(CandidateResultData {
                        name: "c".to_string(),
                        vote_count: 10.0 + (27.0 - quota) * (9.0 / 27.0),
                    }),
                },
                VotingRoundResult {
                    round: 3,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: 10.0 + (27.0 - quota) * (10.0 / 27.0),
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
    async fn test_vote_transfer_chain() {
        let candidates = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
            "f".to_string(),
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
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 4.0,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 2.0,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "d".to_string(),
                        vote_count: 1.0,
                    }),
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 6.0,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 4.0,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "c".to_string(),
                        vote_count: 3.0,
                    }),
                },
                VotingRoundResult {
                    round: 3,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "a".to_string(),
                            vote_count: 6.0,
                        },
                        is_selected: false,
                    }],
                    dropped_candidate: Some(CandidateResultData {
                        name: "b".to_string(),
                        vote_count: 5.0,
                    }),
                },
                VotingRoundResult {
                    round: 4,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "a".to_string(),
                            vote_count: 7.0,
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
}
