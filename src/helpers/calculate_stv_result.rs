use crate::{
    api_types::ApiResult,
    models::{CandidateId, Vote, VotingResult},
};

pub fn calculate_stv_result(
    candidates: Vec<CandidateId>,
    votes: Vec<Vec<CandidateId>>,
    number_of_winners: u32,
) -> ApiResult<VotingResult> {
    todo!()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        api_types::ApiError,
        helpers::calculate_stv_result,
        models::{
            CandidateId, CandidateResultData, PassingCandidateResult, Voting, VotingResult,
            VotingRoundResult, VotingState,
        },
    };

    #[tokio::test(flavor = "multi_thread")]
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
            winners: vec!["a".to_string()],
        };

        match result {
            Ok(res) => assert_eq!(res, expected_result),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_surplus_votes_passed() {
        let candidates = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let votes: Vec<Vec<CandidateId>> = vec![
            vec!["a".to_string(), "c".to_string(), "b".to_string()],
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        ];
        let result = calculate_stv_result(candidates, votes, 2);

        let expected_result = VotingResult {
            round_results: vec![
                VotingRoundResult {
                    round: 1,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 2.0,
                            },
                            is_selected: true,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: 2.0 / 3.0,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "c".to_string(),
                        vote_count: 1.0 / 3.0,
                    }),
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: 1.0,
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
    async fn test_surplus_votes_no_next_candidate() {
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
                                vote_count: quota,
                            },
                            is_selected: true,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "b".to_string(),
                                vote_count: ((4.0 - quota) / 4.0) * 2.0,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "c".to_string(),
                        vote_count: ((4.0 - quota) / 4.0),
                    }),
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![PassingCandidateResult {
                        data: CandidateResultData {
                            name: "b".to_string(),
                            vote_count: ((4.0 - quota) / 4.0) * 2.0 + ((4.0 - quota) / 4.0),
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
    async fn test_greater_surplus_distributed_first() {
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
        let quota = (votes.len() as f64 / (2.0 + 1.0)) + 1.0;
        let result = calculate_stv_result(candidates, votes, 2);

        let expected_first_round = VotingRoundResult {
            round: 1,
            candidate_results: vec![
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "a".to_string(),
                        vote_count: quota,
                    },
                    is_selected: true,
                },
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "b".to_string(),
                        vote_count: quota,
                    },
                    is_selected: true,
                },
                PassingCandidateResult {
                    data: CandidateResultData {
                        name: "c".to_string(),
                        vote_count: (16.0 - quota) * (1.0 / 16.0),
                    },
                    is_selected: false,
                },
            ],
            dropped_candidate: Some(CandidateResultData {
                name: "d".to_string(),
                vote_count: ((12.0 - quota) + (16.0 - quota) * (15.0 / 16.0)) * (1.0 / 12.0),
            }),
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
    async fn test_dropped_candidate_votes_transfer_proportionally() {
        let candidates = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let a_b_c_votes: Vec<Vec<String>> =
            std::iter::repeat(vec!["a".to_string(), "b".to_string(), "c".to_string()])
                .take(10)
                .collect();
        let b_a_c_votes: Vec<Vec<String>> =
            std::iter::repeat(vec!["b".to_string(), "a".to_string(), "c".to_string()])
                .take(7)
                .collect();
        let b_c_a_votes: Vec<Vec<String>> =
            std::iter::repeat(vec!["b".to_string(), "c".to_string(), "a".to_string()])
                .take(2)
                .collect();
        let c_b_a_votes: Vec<Vec<String>> =
            std::iter::repeat(vec!["c".to_string(), "b".to_string(), "a".to_string()])
                .take(10)
                .collect();

        let votes: Vec<Vec<CandidateId>> =
            [a_b_c_votes, b_a_c_votes, b_c_a_votes, c_b_a_votes].concat();

        let quota = (votes.len() as f64 / (1.0 + 1.0)) + 1.0;
        let result = calculate_stv_result(candidates, votes, 1);

        let expected_result = VotingResult {
            round_results: vec![
                VotingRoundResult {
                    round: 1,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 10.0,
                            },
                            is_selected: false,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 1.0,
                            },
                            is_selected: false,
                        },
                    ],
                    dropped_candidate: Some(CandidateResultData {
                        name: "b".to_string(),
                        vote_count: 9.0,
                    }),
                },
                VotingRoundResult {
                    round: 2,
                    candidate_results: vec![
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "a".to_string(),
                                vote_count: 10.0 + (7.0 / 9.0) * 9.0,
                            },
                            is_selected: true,
                        },
                        PassingCandidateResult {
                            data: CandidateResultData {
                                name: "c".to_string(),
                                vote_count: 10.0 + (2.0 / 9.0) * 9.0,
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
