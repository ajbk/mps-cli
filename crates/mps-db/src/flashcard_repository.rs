#[cfg(test)]
mod tests {
    use super::*;
    use mps_flashcards::{
        CanonicalCatalog, FlashcardCard, FlashcardStatus, VisualBrief, LOCKED_STYLE_PROFILE,
    };
    use serde_json::{json, Value};

    fn catalog() -> CanonicalCatalog {
        CanonicalCatalog::load_json(include_str!("../../../data/reference/mps_database_v1_core.json"))
            .expect("canonical catalog should load")
    }

    fn repository() -> FlashcardRepository {
        FlashcardRepository::open_in_memory(catalog()).expect("open flashcard repository")
    }

    fn card(id: &str) -> FlashcardCard {
        FlashcardCard {
            id: id.into(),
            source_exercise_id: "source_chair_achilles_stretch_row_4".into(),
            category: "Spinal Mobility".into(),
            teaching_copy_json: json!({"cue": "Breathe"}),
            style_profile: LOCKED_STYLE_PROFILE.into(),
            character_id: "teacher-01".into(),
            current_asset_id: None,
            version: 1,
            status: FlashcardStatus::Draft,
        }
    }

    fn brief(card_id: &str) -> VisualBrief {
        VisualBrief {
            id: "brief-1".into(),
            card_id: card_id.into(),
            exercise_id: "source_chair_achilles_stretch_row_4".into(),
            style_profile: LOCKED_STYLE_PROFILE.into(),
            character_id: "teacher-01".into(),
            outfit: "off-white crop top and charcoal biker shorts".into(),
            pose_json: json!({"position": "standing"}),
            apparatus: "Chair".into(),
            palette_json: json!({"cheekAccent": "#D98F9A"}),
            must_show_json: json!(["neutral lumbar position"]),
            must_not_show_json: json!(["arrows", "text"]),
            version: 1,
        }
    }

    #[test]
    fn creates_a_card_from_a_valid_source_exercise_id() {
        let repository = repository();
        let created = repository.create_flashcard(&card("card-1")).unwrap();

        assert_eq!(created.source_exercise_id, "source_chair_achilles_stretch_row_4");
        assert_eq!(repository.get_flashcard("card-1").unwrap(), Some(created));
        assert_eq!(repository.audit_event_count("card-1").unwrap(), 1);
    }

    #[test]
    fn lists_cards_by_status_and_category() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        let mut other = card("card-2");
        other.category = "Shoulder".into();
        repository.create_flashcard(&other).unwrap();

        let cards = repository
            .list_flashcards(FlashcardListFilter {
                status: Some(FlashcardStatus::Draft),
                category: Some("Spinal Mobility".into()),
            })
            .unwrap();

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "card-1");
    }

    #[test]
    fn stores_a_versioned_brief_and_asset() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        repository.save_visual_brief(&brief("card-1")).unwrap();
        repository
            .record_asset(&FlashcardAsset {
                id: "asset-1".into(),
                card_id: "card-1".into(),
                brief_id: "brief-1".into(),
                repo_path: "docs/assets/asset-1.png".into(),
                provider_job_id: Some("provider-job-1".into()),
                version: 2,
                status: FlashcardAssetStatus::NeedsReview,
            })
            .unwrap();

        assert_eq!(repository.latest_brief_version("card-1").unwrap(), Some(1));
        assert_eq!(repository.asset_version("asset-1").unwrap(), Some(2));
        assert_eq!(
            repository
                .get_flashcard("card-1")
                .unwrap()
                .unwrap()
                .current_asset_id,
            Some("asset-1".into())
        );
        assert_eq!(repository.audit_event_count("card-1").unwrap(), 3);
    }

    #[test]
    fn automated_review_audit_uses_the_system_service_actor() {
        let repository = repository();
        repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-1"))
            .unwrap();
        repository
            .save_visual_brief_for_studio("studio-a", "teacher-01", &brief("card-1"))
            .unwrap();
        repository
            .record_asset(&FlashcardAsset {
                id: "asset-1".into(),
                card_id: "card-1".into(),
                brief_id: "brief-1".into(),
                repo_path: "docs/assets/asset-1.png".into(),
                provider_job_id: Some("provider-job-1".into()),
                version: 1,
                status: FlashcardAssetStatus::NeedsReview,
            })
            .unwrap();
        repository
            .record_automated_review_for_studio(
                "studio-a",
                "automated-review-service",
                &FlashcardReview {
                    id: "review-1".into(),
                    card_id: "card-1".into(),
                    asset_id: "asset-1".into(),
                    passed: true,
                    findings_json: json!([]),
                    reviewer_kind: ReviewerKind::Automated,
                    reviewer_id: None,
                },
            )
            .unwrap();

        assert_eq!(
            audit_actor_for_action(&repository, "card-1", "flashcard_review.recorded"),
            Some(("system".into(), Some("automated-review-service".into())))
        );
    }

    #[test]
    fn records_jobs_and_explicit_audit_events() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        repository
            .create_job(&FlashcardJob {
                id: "job-1".into(),
                card_id: "card-1".into(),
                kind: FlashcardJobKind::Generate,
                status: FlashcardJobStatus::Queued,
                input_json: json!({"brief_id": "brief-1"}),
                output_json: None,
                error: None,
            })
            .unwrap();
        repository
            .record_audit_event(
                "card-1",
                AuditActorKind::Teacher,
                Some("teacher-01"),
                "flashcard.note_added",
                &json!({"note": "Check elbow placement"}),
            )
            .unwrap();

        assert_eq!(repository.audit_event_count("card-1").unwrap(), 3);
    }

    #[test]
    fn rejects_review_jobs_before_they_can_be_queued() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();

        assert!(repository
            .create_job(&FlashcardJob {
                id: "review-job-1".into(),
                card_id: "card-1".into(),
                kind: FlashcardJobKind::Review,
                status: FlashcardJobStatus::Queued,
                input_json: json!({}),
                output_json: None,
                error: None,
            })
            .expect_err("review jobs have no supported worker lifecycle")
            .to_string()
            .contains("review jobs are not supported"));
        assert_eq!(repository.audit_event_count("card-1").unwrap(), 1);
    }

    #[test]
    fn worker_completion_atomically_persists_asset_review_and_legal_status() {
        let repository = repository();
        repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-1"))
            .unwrap();
        let mut worker_brief = brief("card-1");
        worker_brief.outfit = LOCKED_OUTFIT.into();
        repository
            .save_visual_brief_for_studio("studio-a", "teacher-01", &worker_brief)
            .unwrap();
        repository
            .create_job_for_studio(
                "studio-a",
                "teacher-01",
                &FlashcardJob {
                    id: "job-1".into(),
                    card_id: "card-1".into(),
                    kind: FlashcardJobKind::Generate,
                    status: FlashcardJobStatus::Queued,
                    input_json: json!({"brief_id": "brief-1"}),
                    output_json: None,
                    error: None,
                },
            )
            .unwrap();
        assert!(repository
            .complete_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-1",
                "job-1",
                &FlashcardWorkerCompletion {
                    claim_id: "claim-1".into(),
                    status: FlashcardJobStatus::Failed,
                    asset: None,
                    review: None,
                    error_code: Some("visual_contract_invalid".into()),
                },
            )
            .expect_err("queued jobs cannot be completed before an authenticated claim")
            .to_string()
            .contains("running"));
        assert_eq!(
            repository
                .claim_job_for_worker("studio-a", "visual-worker", "card-1", "job-1", "claim-1")
                .unwrap()
                .status,
            FlashcardJobStatus::Running
        );
        assert_eq!(
            repository
                .claim_job_for_worker(
                    "studio-a",
                    "visual-worker",
                    "card-1",
                    "job-1",
                    "claim-1",
                )
                .unwrap()
                .input_json
                .get("claim_id")
                .and_then(Value::as_str),
            Some("claim-1")
        );
        assert!(repository
            .claim_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-1",
                "job-1",
                "claim-1-retry",
            )
            .expect_err("a same-worker retry with a different attempt must be rejected")
            .to_string()
            .contains("persisted running attempt"));
        assert!(repository
            .claim_job_for_worker(
                "studio-a",
                "other-worker",
                "card-1",
                "job-1",
                "claim-other",
            )
            .expect_err("a running job must reject an unrelated worker claim")
            .to_string()
            .contains("another worker"));
        assert!(repository
            .complete_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-1",
                "job-1",
                &FlashcardWorkerCompletion {
                    claim_id: "claim-other".into(),
                    status: FlashcardJobStatus::Failed,
                    asset: None,
                    review: None,
                    error_code: Some("visual_contract_invalid".into()),
                },
            )
            .expect_err("completion must require the persisted worker claim")
            .to_string()
            .contains("claim token"));

        let completed = repository
            .complete_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-1",
                "job-1",
                &FlashcardWorkerCompletion {
                    claim_id: "claim-1".into(),
                    status: FlashcardJobStatus::Succeeded,
                    asset: Some(FlashcardAsset {
                        id: "asset-1".into(),
                        card_id: "card-1".into(),
                        brief_id: "brief-1".into(),
                        repo_path: "object://mps-flashcards/card-1-v1.png".into(),
                        provider_job_id: Some("provider-1".into()),
                        version: 1,
                        status: FlashcardAssetStatus::NeedsReview,
                    }),
                    review: Some(FlashcardReview {
                        id: "review-1".into(),
                        card_id: "card-1".into(),
                        asset_id: "asset-1".into(),
                        passed: true,
                        findings_json: json!([]),
                        reviewer_kind: ReviewerKind::Automated,
                        reviewer_id: None,
                    }),
                    error_code: None,
                },
            )
            .unwrap();

        assert_eq!(completed.status, FlashcardJobStatus::Succeeded);
        assert_eq!(repository.asset_version("asset-1").unwrap(), Some(1));
        assert_eq!(
            repository.get_flashcard("card-1").unwrap().unwrap().status,
            FlashcardStatus::NeedsReview
        );
        let retry = repository
            .complete_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-1",
                "job-1",
                &FlashcardWorkerCompletion {
                    claim_id: "claim-1".into(),
                    status: FlashcardJobStatus::Succeeded,
                    asset: Some(FlashcardAsset {
                        id: "asset-1".into(),
                        card_id: "card-1".into(),
                        brief_id: "brief-1".into(),
                        repo_path: "object://mps-flashcards/card-1-v1.png".into(),
                        provider_job_id: Some("provider-1".into()),
                        version: 99,
                        status: FlashcardAssetStatus::NeedsReview,
                    }),
                    review: Some(FlashcardReview {
                        id: "review-1".into(),
                        card_id: "card-1".into(),
                        asset_id: "asset-1".into(),
                        passed: true,
                        findings_json: json!([]),
                        reviewer_kind: ReviewerKind::Automated,
                        reviewer_id: None,
                    }),
                    error_code: None,
                },
            )
            .unwrap();
        assert_eq!(retry.status, FlashcardJobStatus::Succeeded);
        assert!(repository
            .complete_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-1",
                "job-1",
                &FlashcardWorkerCompletion {
                    claim_id: "claim-1".into(),
                    status: FlashcardJobStatus::Succeeded,
                    asset: Some(FlashcardAsset {
                        id: "asset-conflict".into(),
                        card_id: "card-1".into(),
                        brief_id: "brief-1".into(),
                        repo_path: "object://mps-flashcards/card-1-v1.png".into(),
                        provider_job_id: Some("provider-1".into()),
                        version: 1,
                        status: FlashcardAssetStatus::NeedsReview,
                    }),
                    review: Some(FlashcardReview {
                        id: "review-conflict".into(),
                        card_id: "card-1".into(),
                        asset_id: "asset-conflict".into(),
                        passed: true,
                        findings_json: json!([]),
                        reviewer_kind: ReviewerKind::Automated,
                        reviewer_id: None,
                    }),
                    error_code: None,
                },
            )
            .is_err());

        repository
            .transition_status("card-1", FlashcardStatus::RevisionRequested)
            .unwrap();
        let mut next_worker_brief = worker_brief.clone();
        next_worker_brief.id = "brief-1-v2".into();
        next_worker_brief.version = 2;
        repository
            .save_visual_brief_for_studio("studio-a", "teacher-01", &next_worker_brief)
            .unwrap();
        repository
            .create_job_for_studio(
                "studio-a",
                "teacher-01",
                &FlashcardJob {
                    id: "job-1-v2".into(),
                    card_id: "card-1".into(),
                    kind: FlashcardJobKind::Regenerate,
                    status: FlashcardJobStatus::Queued,
                    input_json: json!({"brief_id": "brief-1-v2"}),
                    output_json: None,
                    error: None,
                },
            )
            .unwrap();
        repository
            .claim_job_for_worker("studio-a", "visual-worker", "card-1", "job-1-v2", "claim-1-v2")
            .unwrap();
        repository
            .complete_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-1",
                "job-1-v2",
                &FlashcardWorkerCompletion {
                    claim_id: "claim-1-v2".into(),
                    status: FlashcardJobStatus::Succeeded,
                    asset: Some(FlashcardAsset {
                        id: "asset-1-v2".into(),
                        card_id: "card-1".into(),
                        brief_id: "brief-1-v2".into(),
                        repo_path: "object://mps-flashcards/card-1-v2.png".into(),
                        provider_job_id: Some("provider-1-v2".into()),
                        version: 1,
                        status: FlashcardAssetStatus::NeedsReview,
                    }),
                    review: Some(FlashcardReview {
                        id: "review-1-v2".into(),
                        card_id: "card-1".into(),
                        asset_id: "asset-1-v2".into(),
                        passed: true,
                        findings_json: json!([]),
                        reviewer_kind: ReviewerKind::Automated,
                        reviewer_id: None,
                    }),
                    error_code: None,
                },
            )
            .unwrap();
        assert_eq!(repository.asset_version("asset-1-v2").unwrap(), Some(2));

        repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-2"))
            .unwrap();
        let mut second_brief = brief("card-2");
        second_brief.id = "brief-2".into();
        second_brief.outfit = LOCKED_OUTFIT.into();
        repository
            .save_visual_brief_for_studio("studio-a", "teacher-01", &second_brief)
            .unwrap();
        repository
            .create_job_for_studio(
                "studio-a",
                "teacher-01",
                &FlashcardJob {
                    id: "job-2".into(),
                    card_id: "card-2".into(),
                    kind: FlashcardJobKind::Generate,
                    status: FlashcardJobStatus::Queued,
                    input_json: json!({"brief_id": "brief-2"}),
                    output_json: None,
                    error: None,
                },
            )
            .unwrap();
        repository
            .claim_job_for_worker("studio-a", "visual-worker", "card-2", "job-2", "claim-2")
            .unwrap();
        repository
            .complete_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-2",
                "job-2",
                &FlashcardWorkerCompletion {
                    claim_id: "claim-2".into(),
                    status: FlashcardJobStatus::Failed,
                    asset: None,
                    review: None,
                    error_code: Some("visual_contract_invalid".into()),
                },
            )
            .unwrap();
        assert_eq!(
            repository.get_flashcard("card-2").unwrap().unwrap().status,
            FlashcardStatus::RevisionRequested
        );
        assert_eq!(
            repository
                .get_job_for_studio("studio-a", "card-2", "job-2")
                .unwrap()
                .unwrap()
                .status,
            FlashcardJobStatus::Failed
        );

        repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-3"))
            .unwrap();
        let mut third_brief = brief("card-3");
        third_brief.id = "brief-3".into();
        third_brief.outfit = LOCKED_OUTFIT.into();
        repository
            .save_visual_brief_for_studio("studio-a", "teacher-01", &third_brief)
            .unwrap();
        repository
            .create_job_for_studio(
                "studio-a",
                "teacher-01",
                &FlashcardJob {
                    id: "job-3".into(),
                    card_id: "card-3".into(),
                    kind: FlashcardJobKind::Generate,
                    status: FlashcardJobStatus::Queued,
                    input_json: json!({"brief_id": "brief-3"}),
                    output_json: None,
                    error: None,
                },
            )
            .unwrap();
        repository
            .claim_job_for_worker("studio-a", "visual-worker", "card-3", "job-3", "claim-3")
            .unwrap();
        repository
            .complete_job_for_worker(
                "studio-a",
                "visual-worker",
                "card-3",
                "job-3",
                &FlashcardWorkerCompletion {
                    claim_id: "claim-3".into(),
                    status: FlashcardJobStatus::Succeeded,
                    asset: Some(FlashcardAsset {
                        id: "asset-3".into(),
                        card_id: "card-3".into(),
                        brief_id: "brief-3".into(),
                        repo_path: "object://mps-flashcards/card-3-v1.png".into(),
                        provider_job_id: None,
                        version: 1,
                        status: FlashcardAssetStatus::Rejected,
                    }),
                    review: Some(FlashcardReview {
                        id: "review-3".into(),
                        card_id: "card-3".into(),
                        asset_id: "asset-3".into(),
                        passed: false,
                        findings_json: json!([{"code": "visual_check_failed", "severity": "error"}]),
                        reviewer_kind: ReviewerKind::Automated,
                        reviewer_id: None,
                    }),
                    error_code: None,
                },
            )
            .unwrap();
        assert_eq!(
            repository.get_flashcard("card-3").unwrap().unwrap().status,
            FlashcardStatus::RevisionRequested
        );
        assert_eq!(
            repository
                .get_job_for_studio("studio-a", "card-3", "job-3")
                .unwrap()
                .unwrap()
                .status,
            FlashcardJobStatus::Succeeded
        );
    }

    #[test]
    fn rejects_publish_until_an_approved_card_has_a_passing_latest_review() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        repository.transition_status("card-1", FlashcardStatus::Generating).unwrap();
        repository.transition_status("card-1", FlashcardStatus::NeedsReview).unwrap();
        repository.transition_status("card-1", FlashcardStatus::Approved).unwrap();

        let error = repository
            .publish_flashcard("card-1", AuditActorKind::Teacher, Some("teacher-01"))
            .expect_err("a card cannot publish without a review");
        assert!(error.to_string().contains("current asset"));

        repository.save_visual_brief(&brief("card-1")).unwrap();
        repository
            .record_asset(&FlashcardAsset {
                id: "asset-1".into(),
                card_id: "card-1".into(),
                brief_id: "brief-1".into(),
                repo_path: "docs/assets/asset-1.png".into(),
                provider_job_id: None,
                version: 1,
                status: FlashcardAssetStatus::Approved,
            })
            .unwrap();
        repository
            .record_review(&FlashcardReview {
                id: "review-1".into(),
                card_id: "card-1".into(),
                asset_id: "asset-1".into(),
                passed: true,
                findings_json: json!([]),
                reviewer_kind: ReviewerKind::Automated,
                reviewer_id: None,
            })
            .unwrap();

        repository
            .record_review(&FlashcardReview {
                id: "review-2".into(),
                card_id: "card-1".into(),
                asset_id: "asset-1".into(),
                passed: false,
                findings_json: json!(["elbow angle is unclear"]),
                reviewer_kind: ReviewerKind::Automated,
                reviewer_id: None,
            })
            .unwrap();
        assert!(repository
            .publish_flashcard("card-1", AuditActorKind::Teacher, Some("teacher-01"))
            .expect_err("a failed latest review must block publication")
            .to_string()
            .contains("latest review"));

        repository
            .record_review(&FlashcardReview {
                id: "review-3".into(),
                card_id: "card-1".into(),
                asset_id: "asset-1".into(),
                passed: true,
                findings_json: json!([]),
                reviewer_kind: ReviewerKind::Teacher,
                reviewer_id: Some("teacher-01".into()),
            })
            .unwrap();

        repository
            .record_asset(&FlashcardAsset {
                id: "asset-2".into(),
                card_id: "card-1".into(),
                brief_id: "brief-1".into(),
                repo_path: "docs/assets/asset-2.png".into(),
                provider_job_id: None,
                version: 2,
                status: FlashcardAssetStatus::Approved,
            })
            .unwrap();
        assert!(repository
            .publish_flashcard("card-1", AuditActorKind::Teacher, Some("teacher-01"))
            .expect_err("a review for an older asset must not unlock publication")
            .to_string()
            .contains("current asset"));
        repository
            .record_review(&FlashcardReview {
                id: "review-4".into(),
                card_id: "card-1".into(),
                asset_id: "asset-2".into(),
                passed: true,
                findings_json: json!([]),
                reviewer_kind: ReviewerKind::Teacher,
                reviewer_id: Some("teacher-01".into()),
            })
            .unwrap();

        assert!(repository
            .transition_status("card-1", FlashcardStatus::Published)
            .expect_err("direct publication must be rejected")
            .to_string()
            .contains("publish_flashcard"));
        assert!(repository
            .publish_flashcard("card-1", AuditActorKind::Chatgpt, Some("chatgpt-1"))
            .expect_err("AI actors must not publish")
            .to_string()
            .contains("teacher"));

        repository
            .publish_flashcard("card-1", AuditActorKind::Teacher, Some("teacher-01"))
            .unwrap();
        assert_eq!(
            repository.get_flashcard("card-1").unwrap().unwrap().status,
            FlashcardStatus::Published
        );
        assert_eq!(repository.audit_event_count("card-1").unwrap(), 12);
        assert_eq!(
            audit_actor_for_action(&repository, "card-1", "flashcard.published"),
            Some(("teacher".into(), Some("teacher-01".into()))),
        );
    }

    #[test]
    fn rejects_visual_briefs_that_do_not_match_the_card_source_exercise() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        let missing_card_brief = brief("missing-card");
        assert!(repository
            .save_visual_brief(&missing_card_brief)
            .expect_err("brief card must exist")
            .to_string()
            .contains("flashcard not found"));
        let mut mismatched = brief("card-1");
        mismatched.exercise_id = "source_chair_arm_work_with_split_pedal_row_5".into();

        assert!(repository
            .save_visual_brief(&mismatched)
            .expect_err("brief exercise must match the source exercise")
            .to_string()
            .contains("source exercise"));
        assert_eq!(repository.audit_event_count("card-1").unwrap(), 1);
    }

    #[test]
    fn rejects_reviews_and_assets_that_break_card_ownership() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        repository.create_flashcard(&card("card-2")).unwrap();
        repository.save_visual_brief(&brief("card-1")).unwrap();
        let mut card_two_brief = brief("card-2");
        card_two_brief.id = "brief-2".into();
        repository.save_visual_brief(&card_two_brief).unwrap();

        assert!(repository
            .record_asset(&FlashcardAsset {
                id: "bad-asset".into(),
                card_id: "card-1".into(),
                brief_id: "brief-2".into(),
                repo_path: "docs/assets/bad.png".into(),
                provider_job_id: None,
                version: 1,
                status: FlashcardAssetStatus::NeedsReview,
            })
            .expect_err("asset brief must belong to the same card")
            .to_string()
            .contains("brief"));
        assert_eq!(repository.asset_version("bad-asset").unwrap(), None);
        assert_eq!(
            repository
                .get_flashcard("card-1")
                .unwrap()
                .unwrap()
                .current_asset_id,
            None
        );

        repository
            .record_asset(&FlashcardAsset {
                id: "asset-2".into(),
                card_id: "card-2".into(),
                brief_id: "brief-2".into(),
                repo_path: "docs/assets/asset-2.png".into(),
                provider_job_id: None,
                version: 1,
                status: FlashcardAssetStatus::Approved,
            })
            .unwrap();
        assert!(repository
            .record_review(&FlashcardReview {
                id: "cross-card-review".into(),
                card_id: "card-1".into(),
                asset_id: "asset-2".into(),
                passed: true,
                findings_json: json!([]),
                reviewer_kind: ReviewerKind::Teacher,
                reviewer_id: Some("teacher-01".into()),
            })
            .expect_err("reviews cannot use another card's asset")
            .to_string()
            .contains("does not belong"));
    }

    fn audit_actor_for_action(
        repository: &FlashcardRepository,
        card_id: &str,
        action: &str,
    ) -> Option<(String, Option<String>)> {
        repository.runtime.block_on(async {
            sqlx::query_as::<_, (String, Option<String>)>(
                "SELECT actor_kind, actor_id FROM flashcard_audit_events WHERE card_id = ? AND action = ? ORDER BY id DESC LIMIT 1",
            )
            .bind(card_id)
            .bind(action)
            .fetch_optional(&repository.pool)
            .await
            .expect("audit lookup should succeed")
        })
    }
}
use std::ops::Deref;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use mps_flashcards::{
    can_transition, validate_new_draft, validate_visual_brief, CanonicalCatalog, FlashcardCard,
    FlashcardStatus, VisualBrief,
};
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};
use tokio::runtime::Runtime;

const LOCKED_CHARACTER_ID: &str = "teacher-01";
const LOCKED_OUTFIT: &str = "off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts";
const DUSTY_ROSE_CHEEK_ACCENT: &str = "#D98F9A";

#[derive(Debug, Clone, Default)]
pub struct FlashcardListFilter {
    pub status: Option<FlashcardStatus>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FlashcardPatch {
    pub category: Option<String>,
    pub teaching_copy_json: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashcardAssetStatus {
    Generating,
    NeedsReview,
    Rejected,
    Approved,
}

#[derive(Debug, Clone)]
pub struct FlashcardAsset {
    pub id: String,
    pub card_id: String,
    pub brief_id: String,
    pub repo_path: String,
    pub provider_job_id: Option<String>,
    pub version: i64,
    pub status: FlashcardAssetStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashcardJobKind {
    Generate,
    Review,
    Regenerate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashcardJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct FlashcardJob {
    pub id: String,
    pub card_id: String,
    pub kind: FlashcardJobKind,
    pub status: FlashcardJobStatus,
    pub input_json: Value,
    pub output_json: Option<Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewerKind {
    Automated,
    Teacher,
}

#[derive(Debug, Clone)]
pub struct FlashcardReview {
    pub id: String,
    pub card_id: String,
    pub asset_id: String,
    pub passed: bool,
    pub findings_json: Value,
    pub reviewer_kind: ReviewerKind,
    pub reviewer_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FlashcardWorkerCompletion {
    pub claim_id: String,
    pub status: FlashcardJobStatus,
    pub asset: Option<FlashcardAsset>,
    pub review: Option<FlashcardReview>,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FlashcardVisualDetails {
    pub asset: Option<FlashcardAsset>,
    pub automated_review: Option<FlashcardReview>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditActorKind {
    Teacher,
    Chatgpt,
    System,
}

#[derive(Debug, FromRow)]
struct CardRow {
    id: String,
    source_exercise_id: String,
    category: String,
    teaching_copy_json: String,
    style_profile: String,
    character_id: String,
    current_asset_id: Option<String>,
    version: i64,
    status: String,
}

#[derive(Debug, FromRow)]
struct JobRow {
    id: String,
    card_id: String,
    kind: String,
    status: String,
    input_json: String,
    output_json: Option<String>,
    error: Option<String>,
}

pub struct FlashcardRepository {
    pool: SqlitePool,
    runtime: Arc<RepositoryRuntime>,
    catalog: CanonicalCatalog,
}

struct RepositoryRuntime(Option<Runtime>);

impl RepositoryRuntime {
    fn new(runtime: Runtime) -> Self {
        Self(Some(runtime))
    }
}

impl Deref for RepositoryRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("repository runtime is available until drop")
    }
}

impl Drop for RepositoryRuntime {
    fn drop(&mut self) {
        if let Some(runtime) = self.0.take() {
            runtime.shutdown_background();
        }
    }
}

impl FlashcardRepository {
    pub fn open(path: &str, catalog: CanonicalCatalog) -> Result<Self> {
        let runtime = Runtime::new().context("create Tokio runtime")?;
        let database_url = format!("sqlite:{path}?mode=rwc");
        let pool = runtime.block_on(async {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
                .with_context(|| format!("connect to SQLite database at {path}"))?;
            sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            Ok::<_, anyhow::Error>(pool)
        })?;

        Ok(Self {
            pool,
            runtime: Arc::new(RepositoryRuntime::new(runtime)),
            catalog,
        })
    }

    pub fn open_in_memory(catalog: CanonicalCatalog) -> Result<Self> {
        let runtime = Runtime::new().context("create Tokio runtime")?;
        let pool = runtime.block_on(async {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .context("connect to in-memory SQLite")?;
            sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            Ok::<_, anyhow::Error>(pool)
        })?;

        Ok(Self {
            pool,
            runtime: Arc::new(RepositoryRuntime::new(runtime)),
            catalog,
        })
    }

    pub fn create_flashcard(&self, card: &FlashcardCard) -> Result<FlashcardCard> {
        validate_new_draft(card, &self.catalog).map_err(|error| anyhow!(error))?;
        let card = card.clone();
        let timestamp = timestamp();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            sqlx::query("INSERT INTO flashcard_cards (id, source_exercise_id, category, teaching_copy_json, style_profile, character_id, current_asset_id, version, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&card.id).bind(&card.source_exercise_id).bind(&card.category)
                .bind(serde_json::to_string(&card.teaching_copy_json)?)
                .bind(&card.style_profile).bind(&card.character_id).bind(&card.current_asset_id)
                .bind(card.version).bind(status_to_db(card.status)).bind(&timestamp).bind(&timestamp)
                .execute(&mut *transaction).await?;
            record_audit_in_transaction(&mut transaction, &card.id, AuditActorKind::System, None, "flashcard.created", json!({"version": card.version})).await?;
            transaction.commit().await?;
            Ok(())
        })?;
        Ok(card)
    }

    pub fn create_flashcard_for_studio(
        &self,
        studio_id: &str,
        teacher_id: &str,
        card: &FlashcardCard,
    ) -> Result<FlashcardCard> {
        self.create_flashcard_for_actor(studio_id, teacher_id, AuditActorKind::Teacher, card)
    }

    pub fn create_flashcard_for_actor(
        &self,
        studio_id: &str,
        teacher_id: &str,
        actor_kind: AuditActorKind,
        card: &FlashcardCard,
    ) -> Result<FlashcardCard> {
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        validate_new_draft(card, &self.catalog).map_err(|error| anyhow!(error))?;
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let actor_kind = actor_kind.clone();
        let card = card.clone();
        let now = timestamp();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            sqlx::query("INSERT INTO flashcard_cards (id, source_exercise_id, category, teaching_copy_json, style_profile, character_id, current_asset_id, version, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&card.id)
                .bind(&card.source_exercise_id)
                .bind(&card.category)
                .bind(serde_json::to_string(&card.teaching_copy_json)?)
                .bind(&card.style_profile)
                .bind(&card.character_id)
                .bind(&card.current_asset_id)
                .bind(card.version)
                .bind(status_to_db(card.status))
                .bind(&now)
                .bind(&now)
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &card.id,
                actor_kind,
                Some(&teacher_id),
                "flashcard.created",
                json!({
                    "studio_id": studio_id,
                    "source_exercise_id": card.source_exercise_id,
                    "version": card.version,
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })?;
        Ok(card)
    }

    pub fn get_flashcard(&self, id: &str) -> Result<Option<FlashcardCard>> {
        let id = id.to_owned();
        self.runtime.block_on(async {
            sqlx::query_as::<_, CardRow>("SELECT id, source_exercise_id, category, teaching_copy_json, style_profile, character_id, current_asset_id, version, status FROM flashcard_cards WHERE id = ?")
                .bind(id).fetch_optional(&self.pool).await?.map(card_from_row).transpose()
        })
    }

    pub fn list_flashcards(&self, filter: FlashcardListFilter) -> Result<Vec<FlashcardCard>> {
        self.runtime.block_on(async {
            let mut query = String::from("SELECT id, source_exercise_id, category, teaching_copy_json, style_profile, character_id, current_asset_id, version, status FROM flashcard_cards WHERE 1 = 1");
            if filter.status.is_some() { query.push_str(" AND status = ?"); }
            if filter.category.is_some() { query.push_str(" AND category = ?"); }
            query.push_str(" ORDER BY created_at, id");
            let mut statement = sqlx::query_as::<_, CardRow>(&query);
            if let Some(status) = filter.status { statement = statement.bind(status_to_db(status)); }
            if let Some(category) = filter.category { statement = statement.bind(category); }
            statement.fetch_all(&self.pool).await?.into_iter().map(card_from_row).collect()
        })
    }

    pub fn get_flashcard_for_studio(
        &self,
        studio_id: &str,
        id: &str,
    ) -> Result<Option<FlashcardCard>> {
        require_identity("studio", studio_id)?;
        let studio_id = studio_id.to_owned();
        let id = id.to_owned();
        self.runtime.block_on(async {
            sqlx::query_as::<_, CardRow>(
                "SELECT cards.id, cards.source_exercise_id, cards.category, cards.teaching_copy_json, cards.style_profile, cards.character_id, cards.current_asset_id, cards.version, cards.status
                 FROM flashcard_cards AS cards
                 WHERE cards.id = ?
                   AND EXISTS (
                       SELECT 1 FROM flashcard_audit_events AS owner
                       WHERE owner.card_id = cards.id
                         AND owner.action = 'flashcard.created'
                         AND json_extract(owner.payload_json, '$.studio_id') = ?
                   )",
            )
            .bind(id)
            .bind(studio_id)
            .fetch_optional(&self.pool)
            .await?
            .map(card_from_row)
            .transpose()
        })
    }

    pub fn visual_details_for_studio(
        &self,
        studio_id: &str,
        card_id: &str,
    ) -> Result<FlashcardVisualDetails> {
        require_identity("studio", studio_id)?;
        let studio_id = studio_id.to_owned();
        let card_id = card_id.to_owned();
        self.runtime.block_on(async {
            let asset: Option<(String, String, String, String, Option<String>, i64, String)> =
                sqlx::query_as(
                    "SELECT assets.id, assets.card_id, assets.brief_id, assets.repo_path,
                            assets.provider_job_id, assets.version, assets.status
                     FROM flashcard_assets AS assets
                     WHERE assets.card_id = ?
                       AND assets.id = (SELECT current_asset_id FROM flashcard_cards WHERE id = ?)
                       AND EXISTS (
                           SELECT 1 FROM flashcard_audit_events AS owner
                           WHERE owner.card_id = assets.card_id
                             AND owner.action = 'flashcard.created'
                             AND json_extract(owner.payload_json, '$.studio_id') = ?
                       )",
                )
                .bind(&card_id)
                .bind(&card_id)
                .bind(&studio_id)
                .fetch_optional(&self.pool)
                .await?;
            let asset = asset.map(asset_from_tuple).transpose()?;
            let automated_review = if let Some(asset) = asset.as_ref() {
                let review: Option<(String, String, String, i64, String, i64, Option<String>)> =
                    sqlx::query_as(
                        "SELECT reviews.id, reviews.card_id, reviews.asset_id, reviews.passed,
                                reviews.findings_json, assets.version, reviews.reviewer_id
                         FROM flashcard_reviews AS reviews
                         JOIN flashcard_assets AS assets ON assets.id = reviews.asset_id
                         WHERE reviews.card_id = ?
                           AND reviews.asset_id = ?
                           AND reviews.reviewer_kind = 'automated'
                           AND EXISTS (
                               SELECT 1 FROM flashcard_audit_events AS owner
                               WHERE owner.card_id = reviews.card_id
                                 AND owner.action = 'flashcard.created'
                                 AND json_extract(owner.payload_json, '$.studio_id') = ?
                           )
                         ORDER BY reviews.created_at DESC, reviews.rowid DESC
                         LIMIT 1",
                    )
                    .bind(&card_id)
                    .bind(&asset.id)
                    .bind(&studio_id)
                    .fetch_optional(&self.pool)
                    .await?;
                review.map(review_from_tuple).transpose()?
            } else {
                None
            };
            Ok(FlashcardVisualDetails {
                asset,
                automated_review,
            })
        })
    }

    pub fn visual_details_for_job_for_studio(
        &self,
        studio_id: &str,
        card_id: &str,
        asset_id: Option<&str>,
        review_id: Option<&str>,
    ) -> Result<FlashcardVisualDetails> {
        require_identity("studio", studio_id)?;
        let studio_id = studio_id.to_owned();
        let card_id = card_id.to_owned();
        let asset_id = asset_id.map(str::to_owned);
        let review_id = review_id.map(str::to_owned);
        self.runtime.block_on(async {
            let review: Option<(String, String, String, i64, String, i64, Option<String>)> =
                if let Some(review_id) = review_id {
                    sqlx::query_as(
                        "SELECT reviews.id, reviews.card_id, reviews.asset_id, reviews.passed,
                                reviews.findings_json, assets.version, reviews.reviewer_id
                         FROM flashcard_reviews AS reviews
                         JOIN flashcard_assets AS assets ON assets.id = reviews.asset_id
                         WHERE reviews.id = ?
                           AND reviews.card_id = ?
                           AND reviews.reviewer_kind = 'automated'
                           AND (? IS NULL OR reviews.asset_id = ?)
                           AND EXISTS (
                               SELECT 1 FROM flashcard_audit_events AS owner
                               WHERE owner.card_id = reviews.card_id
                                 AND owner.action = 'flashcard.created'
                                 AND json_extract(owner.payload_json, '$.studio_id') = ?
                           )",
                    )
                    .bind(review_id)
                    .bind(&card_id)
                    .bind(&asset_id)
                    .bind(&asset_id)
                    .bind(&studio_id)
                    .fetch_optional(&self.pool)
                    .await?
                } else {
                    None
                };
            let selected_asset_id = asset_id.or_else(|| review.as_ref().map(|row| row.2.clone()));
            let asset: Option<(String, String, String, String, Option<String>, i64, String)> =
                if let Some(asset_id) = selected_asset_id {
                    sqlx::query_as(
                        "SELECT assets.id, assets.card_id, assets.brief_id, assets.repo_path,
                                assets.provider_job_id, assets.version, assets.status
                         FROM flashcard_assets AS assets
                         WHERE assets.id = ?
                           AND assets.card_id = ?
                           AND EXISTS (
                               SELECT 1 FROM flashcard_audit_events AS owner
                               WHERE owner.card_id = assets.card_id
                                 AND owner.action = 'flashcard.created'
                                 AND json_extract(owner.payload_json, '$.studio_id') = ?
                           )",
                    )
                    .bind(asset_id)
                    .bind(&card_id)
                    .bind(&studio_id)
                    .fetch_optional(&self.pool)
                    .await?
                } else {
                    None
                };
            let asset = asset.map(asset_from_tuple).transpose()?;
            let automated_review = match (review, asset.as_ref()) {
                (Some(row), Some(asset)) if row.2 == asset.id => Some(review_from_tuple(row)?),
                _ => None,
            };
            Ok(FlashcardVisualDetails {
                asset,
                automated_review,
            })
        })
    }

    pub fn asset_for_studio(
        &self,
        studio_id: &str,
        card_id: &str,
        asset_id: &str,
    ) -> Result<Option<FlashcardAsset>> {
        require_identity("studio", studio_id)?;
        let studio_id = studio_id.to_owned();
        let card_id = card_id.to_owned();
        let asset_id = asset_id.to_owned();
        self.runtime.block_on(async {
            let asset: Option<(String, String, String, String, Option<String>, i64, String)> =
                sqlx::query_as(
                    "SELECT assets.id, assets.card_id, assets.brief_id, assets.repo_path,
                            assets.provider_job_id, assets.version, assets.status
                     FROM flashcard_assets AS assets
                     WHERE assets.id = ? AND assets.card_id = ?
                       AND EXISTS (
                           SELECT 1 FROM flashcard_audit_events AS owner
                           WHERE owner.card_id = assets.card_id
                             AND owner.action = 'flashcard.created'
                             AND json_extract(owner.payload_json, '$.studio_id') = ?
                       )",
                )
                .bind(asset_id)
                .bind(card_id)
                .bind(studio_id)
                .fetch_optional(&self.pool)
                .await?;
            asset.map(asset_from_tuple).transpose()
        })
    }

    pub fn transition_generating_to_needs_review_if_idle_for_actor(
        &self,
        studio_id: &str,
        teacher_id: &str,
        actor_kind: AuditActorKind,
        card_id: &str,
    ) -> Result<()> {
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let card_id = card_id.to_owned();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &card_id).await?;
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            if card.status != FlashcardStatus::Generating {
                return Err(anyhow!("invalid flashcard status transition"));
            }

            let updated = sqlx::query(
                "UPDATE flashcard_cards
                 SET status = 'needs-review', updated_at = ?
                 WHERE id = ?
                   AND status = 'generating'
                   AND NOT EXISTS (
                       SELECT 1 FROM flashcard_jobs AS jobs
                       WHERE jobs.card_id = flashcard_cards.id
                         AND jobs.kind IN ('generate', 'regenerate')
                         AND jobs.status IN ('queued', 'running')
                   )",
            )
            .bind(timestamp())
            .bind(&card_id)
            .execute(&mut *transaction)
            .await?;
            if updated.rows_affected() != 1 {
                let active: i64 = sqlx::query_scalar(
                    "SELECT EXISTS (
                         SELECT 1 FROM flashcard_jobs AS jobs
                         WHERE jobs.card_id = ?
                           AND jobs.kind IN ('generate', 'regenerate')
                           AND jobs.status IN ('queued', 'running')
                     )",
                )
                .bind(&card_id)
                .fetch_one(&mut *transaction)
                .await?;
                return if active == 1 {
                    Err(anyhow!(
                        "cannot submit review while a generation job is active; worker completion submits it"
                    ))
                } else {
                    Err(anyhow!("invalid flashcard status transition"))
                };
            }

            record_audit_in_transaction(
                &mut transaction,
                &card_id,
                actor_kind,
                Some(&teacher_id),
                "flashcard.status_transitioned",
                json!({
                    "studio_id": studio_id,
                    "from": status_to_db(card.status),
                    "to": status_to_db(FlashcardStatus::NeedsReview),
                    "card_version": card.version,
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn list_flashcards_for_studio(
        &self,
        studio_id: &str,
        filter: FlashcardListFilter,
    ) -> Result<Vec<FlashcardCard>> {
        require_identity("studio", studio_id)?;
        let studio_id = studio_id.to_owned();
        self.runtime.block_on(async {
            let mut query = String::from(
                "SELECT cards.id, cards.source_exercise_id, cards.category, cards.teaching_copy_json, cards.style_profile, cards.character_id, cards.current_asset_id, cards.version, cards.status
                 FROM flashcard_cards AS cards
                 WHERE EXISTS (
                     SELECT 1 FROM flashcard_audit_events AS owner
                     WHERE owner.card_id = cards.id
                       AND owner.action = 'flashcard.created'
                       AND json_extract(owner.payload_json, '$.studio_id') = ?
                 )",
            );
            if filter.status.is_some() {
                query.push_str(" AND cards.status = ?");
            }
            if filter.category.is_some() {
                query.push_str(" AND cards.category = ?");
            }
            query.push_str(" ORDER BY cards.created_at, cards.id");
            let mut statement = sqlx::query_as::<_, CardRow>(&query).bind(studio_id);
            if let Some(status) = filter.status {
                statement = statement.bind(status_to_db(status));
            }
            if let Some(category) = filter.category {
                statement = statement.bind(category);
            }
            statement
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(card_from_row)
                .collect()
        })
    }

    pub fn update_flashcard_for_studio(
        &self,
        studio_id: &str,
        teacher_id: &str,
        card_id: &str,
        patch: FlashcardPatch,
    ) -> Result<FlashcardCard> {
        self.update_flashcard_for_actor(studio_id, teacher_id, AuditActorKind::Teacher, card_id, patch)
    }

    pub fn update_flashcard_for_actor(
        &self,
        studio_id: &str,
        teacher_id: &str,
        actor_kind: AuditActorKind,
        card_id: &str,
        patch: FlashcardPatch,
    ) -> Result<FlashcardCard> {
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        if patch.category.is_none() && patch.teaching_copy_json.is_none() {
            return Err(anyhow!("flashcard patch contains no editable fields"));
        }
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let actor_kind = actor_kind.clone();
        let card_id = card_id.to_owned();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &card_id).await?;
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            let category = patch.category.unwrap_or(card.category);
            let teaching_copy_json = patch
                .teaching_copy_json
                .unwrap_or(card.teaching_copy_json);
            let next_version = card.version + 1;
            sqlx::query(
                "UPDATE flashcard_cards
                 SET category = ?, teaching_copy_json = ?, current_asset_id = NULL,
                     version = ?, status = 'draft', updated_at = ?
                 WHERE id = ?",
            )
            .bind(&category)
            .bind(serde_json::to_string(&teaching_copy_json)?)
            .bind(next_version)
            .bind(timestamp())
            .bind(&card_id)
            .execute(&mut *transaction)
            .await?;
            record_audit_in_transaction(
                &mut transaction,
                &card_id,
                actor_kind,
                Some(&teacher_id),
                "flashcard.updated",
                json!({
                    "studio_id": studio_id,
                    "from_version": card.version,
                    "to_version": next_version,
                    "source_exercise_id": card.source_exercise_id,
                    "style_profile": card.style_profile,
                }),
            )
            .await?;
            let updated = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found after update: {card_id}"))?;
            transaction.commit().await?;
            Ok(updated)
        })
    }

    pub fn save_visual_brief(&self, brief: &VisualBrief) -> Result<()> {
        validate_visual_brief(brief).map_err(|error| anyhow!(error))?;
        if self.catalog.find(&brief.exercise_id).is_none() {
            return Err(anyhow!("visual brief exercise is not in the canonical catalog"));
        }
        let brief = brief.clone();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            let card: Option<(String,)> = sqlx::query_as("SELECT source_exercise_id FROM flashcard_cards WHERE id = ?")
                .bind(&brief.card_id).fetch_optional(&mut *transaction).await?;
            let source_exercise_id = card.ok_or_else(|| anyhow!("flashcard not found: {}", brief.card_id))?.0;
            if brief.exercise_id != source_exercise_id {
                return Err(anyhow!("visual brief exercise must match the card source exercise"));
            }
            sqlx::query("INSERT INTO visual_briefs (id, card_id, exercise_id, brief_json, version, created_at) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(&brief.id).bind(&brief.card_id).bind(&brief.exercise_id)
                .bind(serde_json::to_string(&brief)?).bind(brief.version).bind(timestamp())
                .execute(&mut *transaction).await?;
            record_audit_in_transaction(&mut transaction, &brief.card_id, AuditActorKind::System, None, "visual_brief.saved", json!({"brief_id": brief.id, "version": brief.version})).await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn save_visual_brief_for_studio(
        &self,
        studio_id: &str,
        teacher_id: &str,
        brief: &VisualBrief,
    ) -> Result<()> {
        self.save_visual_brief_for_actor(studio_id, teacher_id, AuditActorKind::Teacher, brief)
    }

    pub fn save_visual_brief_for_actor(
        &self,
        studio_id: &str,
        teacher_id: &str,
        actor_kind: AuditActorKind,
        brief: &VisualBrief,
    ) -> Result<()> {
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        validate_visual_brief(brief).map_err(|error| anyhow!(error))?;
        if self.catalog.find(&brief.exercise_id).is_none() {
            return Err(anyhow!(
                "visual brief exercise is not in the canonical catalog"
            ));
        }
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let actor_kind = actor_kind.clone();
        let brief = brief.clone();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &brief.card_id).await?;
            let card = get_flashcard_in_transaction(&mut transaction, &brief.card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {}", brief.card_id))?;
            if brief.exercise_id != card.source_exercise_id {
                return Err(anyhow!(
                    "visual brief exercise must match the card source exercise"
                ));
            }
            sqlx::query("INSERT INTO visual_briefs (id, card_id, exercise_id, brief_json, version, created_at) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(&brief.id)
                .bind(&brief.card_id)
                .bind(&brief.exercise_id)
                .bind(serde_json::to_string(&brief)?)
                .bind(brief.version)
                .bind(timestamp())
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &brief.card_id,
                actor_kind,
                Some(&teacher_id),
                "visual_brief.saved",
                json!({
                    "studio_id": studio_id,
                    "brief_id": brief.id,
                    "card_version": card.version,
                    "brief_version": brief.version,
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn create_job(&self, job: &FlashcardJob) -> Result<()> {
        reject_unsupported_job_kind(job)?;
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            let now = timestamp();
            sqlx::query("INSERT INTO flashcard_jobs (id, card_id, kind, status, input_json, output_json, error, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&job.id).bind(&job.card_id).bind(job_kind_to_db(&job.kind)).bind(job_status_to_db(&job.status))
                .bind(serde_json::to_string(&job.input_json)?).bind(job.output_json.as_ref().map(serde_json::to_string).transpose()?)
                .bind(&job.error).bind(&now).bind(&now).execute(&mut *transaction).await?;
            record_audit_in_transaction(&mut transaction, &job.card_id, AuditActorKind::System, None, "flashcard_job.created", json!({"job_id": job.id})).await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn create_job_for_studio(
        &self,
        studio_id: &str,
        teacher_id: &str,
        job: &FlashcardJob,
    ) -> Result<()> {
        self.create_job_for_actor(studio_id, teacher_id, AuditActorKind::Teacher, job)
    }

    pub fn create_job_for_actor(
        &self,
        studio_id: &str,
        teacher_id: &str,
        actor_kind: AuditActorKind,
        job: &FlashcardJob,
    ) -> Result<()> {
        reject_unsupported_job_kind(job)?;
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let actor_kind = actor_kind.clone();
        let job = job.clone();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &job.card_id).await?;
            let card = get_flashcard_in_transaction(&mut transaction, &job.card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {}", job.card_id))?;
            let next_status = match &job.kind {
                FlashcardJobKind::Generate | FlashcardJobKind::Regenerate => {
                    Some(FlashcardStatus::Generating)
                }
                FlashcardJobKind::Review => None,
            };
            let brief_version = if matches!(
                &job.kind,
                FlashcardJobKind::Generate | FlashcardJobKind::Regenerate
            ) {
                let brief_id = job
                    .input_json
                    .get("brief_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("generation job requires a visual brief ID"))?;
                let brief: Option<(String, i64)> =
                    sqlx::query_as("SELECT card_id, version FROM visual_briefs WHERE id = ?")
                        .bind(brief_id)
                        .fetch_optional(&mut *transaction)
                        .await?;
                let (brief_card_id, brief_version) =
                    brief.ok_or_else(|| anyhow!("visual brief not found: {brief_id}"))?;
                if brief_card_id != job.card_id {
                    return Err(anyhow!(
                        "visual brief does not belong to the flashcard job"
                    ));
                }
                Some(brief_version)
            } else {
                None
            };
            if let Some(next) = next_status {
                if !can_transition(card.status, next) {
                    return Err(anyhow!("invalid flashcard status transition"));
                }
            }
            let now = timestamp();
            sqlx::query("INSERT INTO flashcard_jobs (id, card_id, kind, status, input_json, output_json, error, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&job.id)
                .bind(&job.card_id)
                .bind(job_kind_to_db(&job.kind))
                .bind(job_status_to_db(&job.status))
                .bind(serde_json::to_string(&job.input_json)?)
                .bind(job.output_json.as_ref().map(serde_json::to_string).transpose()?)
                .bind(&job.error)
                .bind(&now)
                .bind(&now)
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &job.card_id,
                actor_kind.clone(),
                Some(&teacher_id),
                "flashcard_job.created",
                json!({
                    "studio_id": studio_id,
                    "job_id": job.id,
                    "kind": job_kind_to_db(&job.kind),
                    "card_version": card.version,
                    "brief_version": brief_version,
                }),
            )
            .await?;
            if let Some(next) = next_status {
                sqlx::query("UPDATE flashcard_cards SET status = ?, updated_at = ? WHERE id = ?")
                    .bind(status_to_db(next))
                    .bind(timestamp())
                    .bind(&job.card_id)
                    .execute(&mut *transaction)
                    .await?;
                record_audit_in_transaction(
                    &mut transaction,
                    &job.card_id,
                    actor_kind,
                    Some(&teacher_id),
                    "flashcard.status_transitioned",
                    json!({
                        "studio_id": studio_id,
                        "from": status_to_db(card.status),
                        "to": status_to_db(next),
                        "job_id": job.id,
                        "card_version": card.version,
                    }),
                )
                .await?;
            }
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn get_job_for_studio(
        &self,
        studio_id: &str,
        card_id: &str,
        job_id: &str,
    ) -> Result<Option<FlashcardJob>> {
        require_identity("studio", studio_id)?;
        let studio_id = studio_id.to_owned();
        let card_id = card_id.to_owned();
        let job_id = job_id.to_owned();
        self.runtime.block_on(async {
            let row = sqlx::query_as::<_, JobRow>(
                "SELECT jobs.id, jobs.card_id, jobs.kind, jobs.status, jobs.input_json,
                        jobs.output_json, jobs.error
                 FROM flashcard_jobs AS jobs
                 WHERE jobs.id = ? AND jobs.card_id = ?
                   AND EXISTS (
                       SELECT 1 FROM flashcard_audit_events AS owner
                       WHERE owner.card_id = jobs.card_id
                         AND owner.action = 'flashcard.created'
                         AND json_extract(owner.payload_json, '$.studio_id') = ?
                   )",
            )
            .bind(job_id)
            .bind(card_id)
            .bind(studio_id)
            .fetch_optional(&self.pool)
            .await?;
            row.map(job_from_row).transpose()
        })
    }

    pub fn claim_job_for_worker(
        &self,
        studio_id: &str,
        worker_id: &str,
        card_id: &str,
        job_id: &str,
        claim_id: &str,
    ) -> Result<FlashcardJob> {
        require_identity("studio", studio_id)?;
        require_identifier("worker", worker_id)?;
        require_identifier("card", card_id)?;
        require_identifier("job", job_id)?;
        require_identifier("claim", claim_id)?;
        let studio_id = studio_id.to_owned();
        let worker_id = worker_id.to_owned();
        let card_id = card_id.to_owned();
        let job_id = job_id.to_owned();
        let claim_id = claim_id.to_owned();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &card_id).await?;
            let mut job = get_job_in_transaction(&mut transaction, &job_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard job not found: {job_id}"))?;
            if job.card_id != card_id {
                return Err(anyhow!("flashcard job does not belong to the card"));
            }
            if matches!(job.status, FlashcardJobStatus::Succeeded | FlashcardJobStatus::Failed) {
                let (claimed_by, _persisted_claim_id) = worker_claim_for_job(&job)?;
                if claimed_by != worker_id {
                    return Err(anyhow!("worker claim is held by another worker"));
                }
                // A terminal claim is read-only reconciliation. Return its persisted token so a
                // retry after a lost terminal response can confirm the committed outcome.
                transaction.commit().await?;
                return Ok(job);
            }
            if job.status == FlashcardJobStatus::Running {
                let (claimed_by, persisted_claim_id) = worker_claim_for_job(&job)?;
                if claimed_by != worker_id {
                    return Err(anyhow!("worker claim is held by another worker"));
                }
                if persisted_claim_id != claim_id {
                    return Err(anyhow!("worker claim token does not match the persisted running attempt"));
                }
                transaction.commit().await?;
                return Ok(job);
            }
            if job.status != FlashcardJobStatus::Queued {
                return Err(anyhow!("worker can only claim queued generation jobs"));
            }
            if !matches!(job.kind, FlashcardJobKind::Generate | FlashcardJobKind::Regenerate) {
                return Err(anyhow!("worker can only claim generation jobs"));
            }
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            if card.status != FlashcardStatus::Generating {
                return Err(anyhow!("worker claim requires a generating flashcard"));
            }
            let mut input_json = job
                .input_json
                .as_object()
                .cloned()
                .ok_or_else(|| anyhow!("generation job input must be an object"))?;
            if input_json.contains_key("claim_id") || input_json.contains_key("claimed_by") {
                return Err(anyhow!("queued worker job already contains a claim"));
            }
            input_json.insert("claim_id".into(), Value::String(claim_id.clone()));
            input_json.insert("claimed_by".into(), Value::String(worker_id.clone()));
            job.input_json = Value::Object(input_json);
            job.status = FlashcardJobStatus::Running;
            let updated = sqlx::query("UPDATE flashcard_jobs SET status = ?, input_json = ?, updated_at = ? WHERE id = ? AND card_id = ? AND status = 'queued'")
                .bind(job_status_to_db(&job.status))
                .bind(serde_json::to_string(&job.input_json)?)
                .bind(timestamp())
                .bind(&job.id)
                .bind(&card_id)
                .execute(&mut *transaction)
                .await?;
            if updated.rows_affected() != 1 {
                return Err(anyhow!("worker claim could not atomically claim the job"));
            }
            record_audit_in_transaction(
                &mut transaction,
                &card_id,
                AuditActorKind::System,
                Some(&worker_id),
                "flashcard_job.claimed",
                json!({
                    "studio_id": studio_id,
                    "worker_id": worker_id,
                    "job_id": job.id,
                    "claim_id": claim_id,
                    "status": job_status_to_db(&job.status),
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(job)
        })
    }

    pub fn complete_job_for_worker(
        &self,
        studio_id: &str,
        worker_id: &str,
        card_id: &str,
        job_id: &str,
        completion: &FlashcardWorkerCompletion,
    ) -> Result<FlashcardJob> {
        require_identity("studio", studio_id)?;
        require_identity("worker", worker_id)?;
        require_identifier("card", card_id)?;
        require_identifier("job", job_id)?;
        let studio_id = studio_id.to_owned();
        let worker_id = worker_id.to_owned();
        let card_id = card_id.to_owned();
        let job_id = job_id.to_owned();
        let completion = completion.clone();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &card_id).await?;
            let mut job = get_job_in_transaction(&mut transaction, &job_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard job not found: {job_id}"))?;
            if job.card_id != card_id {
                return Err(anyhow!("flashcard job does not belong to the card"));
            }
            validate_worker_claim(&job, &worker_id, &completion.claim_id)?;
            if matches!(job.status, FlashcardJobStatus::Succeeded | FlashcardJobStatus::Failed) {
                validate_terminal_worker_retry(&mut transaction, &job, &card_id, &completion)
                    .await?;
                transaction.commit().await?;
                return Ok(job);
            }
            if job.status != FlashcardJobStatus::Running {
                return Err(anyhow!("worker callback requires a running job"));
            }
            if !matches!(job.kind, FlashcardJobKind::Generate | FlashcardJobKind::Regenerate) {
                return Err(anyhow!("worker callbacks only complete generation jobs"));
            }
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            if card.status != FlashcardStatus::Generating {
                return Err(anyhow!("worker callback requires a generating flashcard"));
            }

            let brief_id = job
                .input_json
                .get("brief_id")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("generation job is missing its visual brief ID"))?;
            let brief_row: Option<(String, String)> = sqlx::query_as(
                "SELECT card_id, brief_json FROM visual_briefs WHERE id = ?",
            )
            .bind(brief_id)
            .fetch_optional(&mut *transaction)
            .await?;
            let (brief_card_id, brief_json) =
                brief_row.ok_or_else(|| anyhow!("visual brief not found: {brief_id}"))?;
            if brief_card_id != card_id {
                return Err(anyhow!("visual brief does not belong to the flashcard job"));
            }
            let brief: VisualBrief = serde_json::from_str(&brief_json)
                .context("parse worker visual brief")?;
            require_worker_visual_brief(&brief, &card)?;

            match completion.status {
                FlashcardJobStatus::Succeeded => {
                    let asset = completion
                        .asset
                        .as_ref()
                        .ok_or_else(|| anyhow!("successful worker callback requires an asset"))?;
                    let review = completion
                        .review
                        .as_ref()
                        .ok_or_else(|| anyhow!("successful worker callback requires a review"))?;
                    validate_worker_completion(&card_id, brief_id, asset, review)?;
                    if asset.status
                        != if review.passed {
                            FlashcardAssetStatus::NeedsReview
                        } else {
                            FlashcardAssetStatus::Rejected
                        }
                    {
                        return Err(anyhow!("worker asset status does not match review result"));
                    }
                    sqlx::query("UPDATE flashcard_cards SET updated_at = ? WHERE id = ?")
                        .bind(timestamp())
                        .bind(&card_id)
                        .execute(&mut *transaction)
                        .await?;
                    let next_asset_version =
                        next_asset_version_in_transaction(&mut transaction, &card_id).await?;
                    sqlx::query("INSERT INTO flashcard_assets (id, card_id, brief_id, repo_path, provider_job_id, version, status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                        .bind(&asset.id)
                        .bind(&card_id)
                        .bind(&asset.brief_id)
                        .bind(&asset.repo_path)
                        .bind(&asset.provider_job_id)
                        .bind(next_asset_version)
                        .bind(asset_status_to_db(&asset.status))
                        .bind(timestamp())
                        .execute(&mut *transaction)
                        .await?;
                    sqlx::query("UPDATE flashcard_cards SET current_asset_id = ?, updated_at = ? WHERE id = ?")
                        .bind(&asset.id)
                        .bind(timestamp())
                        .bind(&card_id)
                        .execute(&mut *transaction)
                        .await?;
                    sqlx::query("INSERT INTO flashcard_reviews (id, card_id, asset_id, passed, findings_json, reviewer_kind, reviewer_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                        .bind(&review.id)
                        .bind(&card_id)
                        .bind(&asset.id)
                        .bind(review.passed)
                        .bind(serde_json::to_string(&review.findings_json)? )
                        .bind(reviewer_kind_to_db(&ReviewerKind::Automated))
                        .bind(None::<String>)
                        .bind(timestamp())
                        .execute(&mut *transaction)
                        .await?;
                    record_audit_in_transaction(
                        &mut transaction,
                        &card_id,
                        AuditActorKind::System,
                        Some(&worker_id),
                        "flashcard_asset.recorded",
                        json!({
                            "studio_id": studio_id,
                            "worker_id": worker_id,
                            "job_id": job.id,
                            "asset_id": asset.id,
                            "asset_version": next_asset_version,
                            "status": asset_status_to_db(&asset.status),
                        }),
                    )
                    .await?;
                    record_audit_in_transaction(
                        &mut transaction,
                        &card_id,
                        AuditActorKind::System,
                        Some(&worker_id),
                        "flashcard_review.recorded",
                        json!({
                            "studio_id": studio_id,
                            "worker_id": worker_id,
                            "job_id": job.id,
                            "review_id": review.id,
                            "asset_id": asset.id,
                            "passed": review.passed,
                        }),
                    )
                    .await?;
                    transition_worker_card_status(
                        &mut transaction,
                        &card,
                        FlashcardStatus::NeedsReview,
                        &worker_id,
                        &studio_id,
                        &job.id,
                    )
                    .await?;
                    if !review.passed {
                        let needs_review = get_flashcard_in_transaction(&mut transaction, &card_id)
                            .await?
                            .ok_or_else(|| anyhow!("flashcard not found after review transition"))?;
                        transition_worker_card_status(
                            &mut transaction,
                            &needs_review,
                            FlashcardStatus::RevisionRequested,
                            &worker_id,
                            &studio_id,
                            &job.id,
                        )
                        .await?;
                    }
                    job.output_json = Some(json!({
                        "asset_id": asset.id,
                        "asset_version": next_asset_version,
                        "review_id": review.id,
                        "passed": review.passed,
                    }));
                    job.error = None;
                    job.status = FlashcardJobStatus::Succeeded;
                }
                FlashcardJobStatus::Failed => {
                    if completion.asset.is_some() || completion.review.is_some() {
                        return Err(anyhow!("failed worker callback cannot include an asset or review"));
                    }
                    let error_code = completion
                        .error_code
                        .as_deref()
                        .filter(|value| is_safe_worker_code(value))
                        .unwrap_or("visual_worker_failed")
                        .to_owned();
                    transition_worker_card_status(
                        &mut transaction,
                        &card,
                        FlashcardStatus::NeedsReview,
                        &worker_id,
                        &studio_id,
                        &job.id,
                    )
                    .await?;
                    let needs_review = get_flashcard_in_transaction(&mut transaction, &card_id)
                        .await?
                        .ok_or_else(|| anyhow!("flashcard not found after failure transition"))?;
                    transition_worker_card_status(
                        &mut transaction,
                        &needs_review,
                        FlashcardStatus::RevisionRequested,
                        &worker_id,
                        &studio_id,
                        &job.id,
                    )
                    .await?;
                    job.output_json = Some(json!({ "status": "failed" }));
                    job.error = Some(error_code);
                    job.status = FlashcardJobStatus::Failed;
                }
                FlashcardJobStatus::Queued | FlashcardJobStatus::Running => {
                    return Err(anyhow!("worker callback must be terminal"));
                }
            }
            sqlx::query("UPDATE flashcard_jobs SET status = ?, output_json = ?, error = ?, updated_at = ? WHERE id = ? AND card_id = ?")
                .bind(job_status_to_db(&job.status))
                .bind(job.output_json.as_ref().map(serde_json::to_string).transpose()? )
                .bind(&job.error)
                .bind(timestamp())
                .bind(&job.id)
                .bind(&card_id)
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &card_id,
                AuditActorKind::System,
                Some(&worker_id),
                "flashcard_job.completed",
                json!({
                    "studio_id": studio_id,
                    "worker_id": worker_id,
                    "job_id": job.id,
                    "status": job_status_to_db(&job.status),
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(job)
        })
    }

    pub fn latest_brief_version_for_studio(
        &self,
        studio_id: &str,
        card_id: &str,
    ) -> Result<Option<i64>> {
        require_identity("studio", studio_id)?;
        let studio_id = studio_id.to_owned();
        let card_id = card_id.to_owned();
        self.runtime.block_on(async {
            Ok(sqlx::query_scalar::<_, Option<i64>>(
                "SELECT MAX(briefs.version)
                 FROM visual_briefs AS briefs
                 WHERE briefs.card_id = ?
                   AND EXISTS (
                       SELECT 1 FROM flashcard_audit_events AS owner
                       WHERE owner.card_id = briefs.card_id
                         AND owner.action = 'flashcard.created'
                         AND json_extract(owner.payload_json, '$.studio_id') = ?
                   )",
            )
            .bind(card_id)
            .bind(studio_id)
            .fetch_one(&self.pool)
            .await?)
        })
    }

    pub fn record_asset(&self, asset: &FlashcardAsset) -> Result<()> {
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            let brief_card_id: Option<(String,)> = sqlx::query_as("SELECT card_id FROM visual_briefs WHERE id = ?")
                .bind(&asset.brief_id).fetch_optional(&mut *transaction).await?;
            if brief_card_id.ok_or_else(|| anyhow!("visual brief not found: {}", asset.brief_id))?.0 != asset.card_id {
                return Err(anyhow!("asset brief does not belong to the asset card"));
            }
            sqlx::query("INSERT INTO flashcard_assets (id, card_id, brief_id, repo_path, provider_job_id, version, status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&asset.id).bind(&asset.card_id).bind(&asset.brief_id).bind(&asset.repo_path)
                .bind(&asset.provider_job_id).bind(asset.version).bind(asset_status_to_db(&asset.status)).bind(timestamp())
                .execute(&mut *transaction).await?;
            sqlx::query("UPDATE flashcard_cards SET current_asset_id = ?, updated_at = ? WHERE id = ?")
                .bind(&asset.id).bind(timestamp()).bind(&asset.card_id).execute(&mut *transaction).await?;
            record_audit_in_transaction(&mut transaction, &asset.card_id, AuditActorKind::System, None, "flashcard_asset.recorded", json!({"asset_id": asset.id, "version": asset.version})).await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn record_review(&self, review: &FlashcardReview) -> Result<()> {
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            let asset: Option<(String, String)> = sqlx::query_as("SELECT card_id, brief_id FROM flashcard_assets WHERE id = ?")
                .bind(&review.asset_id).fetch_optional(&mut *transaction).await?;
            let (asset_card_id, brief_id) = asset.ok_or_else(|| anyhow!("flashcard asset not found: {}", review.asset_id))?;
            let brief_card_id: Option<(String,)> = sqlx::query_as("SELECT card_id FROM visual_briefs WHERE id = ?")
                .bind(&brief_id).fetch_optional(&mut *transaction).await?;
            let brief_card_id = brief_card_id.ok_or_else(|| anyhow!("visual brief not found: {brief_id}"))?.0;
            if review.card_id != asset_card_id || asset_card_id != brief_card_id {
                return Err(anyhow!("review asset does not belong to the review card"));
            }
            sqlx::query("INSERT INTO flashcard_reviews (id, card_id, asset_id, passed, findings_json, reviewer_kind, reviewer_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&review.id).bind(&review.card_id).bind(&review.asset_id).bind(review.passed)
                .bind(serde_json::to_string(&review.findings_json)?).bind(reviewer_kind_to_db(&review.reviewer_kind))
                .bind(&review.reviewer_id).bind(timestamp()).execute(&mut *transaction).await?;
            record_audit_in_transaction(&mut transaction, &review.card_id, AuditActorKind::System, None, "flashcard_review.recorded", json!({"review_id": review.id, "passed": review.passed})).await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn record_automated_review_for_studio(
        &self,
        studio_id: &str,
        teacher_id: &str,
        review: &FlashcardReview,
    ) -> Result<()> {
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        if review.reviewer_kind != ReviewerKind::Automated || review.reviewer_id.is_some() {
            return Err(anyhow!(
                "automated review endpoint only accepts automated review results"
            ));
        }
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let review = review.clone();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &review.card_id).await?;
            let asset_version =
                assert_review_asset_in_transaction(&mut transaction, &review).await?;
            let card = get_flashcard_in_transaction(&mut transaction, &review.card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {}", review.card_id))?;
            sqlx::query("INSERT INTO flashcard_reviews (id, card_id, asset_id, passed, findings_json, reviewer_kind, reviewer_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&review.id)
                .bind(&review.card_id)
                .bind(&review.asset_id)
                .bind(review.passed)
                .bind(serde_json::to_string(&review.findings_json)?)
                .bind(reviewer_kind_to_db(&review.reviewer_kind))
                .bind(&review.reviewer_id)
                .bind(timestamp())
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &review.card_id,
                AuditActorKind::System,
                Some(&teacher_id),
                "flashcard_review.recorded",
                json!({
                    "studio_id": studio_id,
                    "review_id": review.id,
                    "asset_id": review.asset_id,
                    "passed": review.passed,
                    "reviewer_kind": "automated",
                    "card_version": card.version,
                    "asset_version": asset_version,
                    "validation_result": if review.passed { "passed" } else { "failed" },
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn transition_status(&self, card_id: &str, next: FlashcardStatus) -> Result<()> {
        if next == FlashcardStatus::Published {
            return Err(anyhow!("use publish_flashcard for teacher-authorized publication"));
        }
        let card_id = card_id.to_owned();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            if !can_transition(card.status, next) {
                return Err(anyhow!("invalid flashcard status transition"));
            }
            sqlx::query("UPDATE flashcard_cards SET status = ?, updated_at = ? WHERE id = ?")
                .bind(status_to_db(next)).bind(timestamp()).bind(&card_id).execute(&mut *transaction).await?;
            record_audit_in_transaction(&mut transaction, &card_id, AuditActorKind::System, None, "flashcard.status_transitioned", json!({"from": status_to_db(card.status), "to": status_to_db(next)})).await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn transition_status_for_studio(
        &self,
        studio_id: &str,
        teacher_id: &str,
        card_id: &str,
        next: FlashcardStatus,
    ) -> Result<()> {
        self.transition_status_for_actor(studio_id, teacher_id, AuditActorKind::Teacher, card_id, next)
    }

    pub fn transition_status_for_actor(
        &self,
        studio_id: &str,
        teacher_id: &str,
        actor_kind: AuditActorKind,
        card_id: &str,
        next: FlashcardStatus,
    ) -> Result<()> {
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        if matches!(next, FlashcardStatus::Approved | FlashcardStatus::Published) {
            return Err(anyhow!(
                "use the guarded approval or publication repository method"
            ));
        }
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let actor_kind = actor_kind.clone();
        let card_id = card_id.to_owned();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &card_id).await?;
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            if !can_transition(card.status, next) {
                return Err(anyhow!("invalid flashcard status transition"));
            }
            sqlx::query("UPDATE flashcard_cards SET status = ?, updated_at = ? WHERE id = ?")
                .bind(status_to_db(next))
                .bind(timestamp())
                .bind(&card_id)
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &card_id,
                actor_kind,
                Some(&teacher_id),
                "flashcard.status_transitioned",
                json!({
                    "studio_id": studio_id,
                    "from": status_to_db(card.status),
                    "to": status_to_db(next),
                    "card_version": card.version,
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn approve_flashcard_for_studio(
        &self,
        studio_id: &str,
        teacher_id: &str,
        card_id: &str,
        review_id: &str,
        findings_json: &Value,
    ) -> Result<()> {
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let card_id = card_id.to_owned();
        let review_id = review_id.to_owned();
        let findings_json = findings_json.clone();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &card_id).await?;
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            require_locked_card_contract(&card, &self.catalog)?;
            if !can_transition(card.status, FlashcardStatus::Approved) {
                return Err(anyhow!("invalid flashcard status transition"));
            }
            let current_asset_id = card
                .current_asset_id
                .ok_or_else(|| anyhow!("cannot approve without a current asset"))?;
            let asset_version = current_asset_contract_version_in_transaction(
                &mut transaction,
                &card,
                &current_asset_id,
            )
            .await?;
            require_latest_review_in_transaction(
                &mut transaction,
                &card_id,
                &current_asset_id,
                ReviewerKind::Automated,
                "automated",
            )
            .await?;
            sqlx::query("INSERT INTO flashcard_reviews (id, card_id, asset_id, passed, findings_json, reviewer_kind, reviewer_id, created_at) VALUES (?, ?, ?, 1, ?, 'teacher', ?, ?)")
                .bind(&review_id)
                .bind(&card_id)
                .bind(&current_asset_id)
                .bind(serde_json::to_string(&findings_json)?)
                .bind(&teacher_id)
                .bind(timestamp())
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &card_id,
                AuditActorKind::Teacher,
                Some(&teacher_id),
                "flashcard_review.recorded",
                json!({
                    "studio_id": studio_id,
                    "review_id": review_id,
                    "asset_id": current_asset_id,
                    "passed": true,
                    "reviewer_kind": "teacher",
                    "card_version": card.version,
                    "asset_version": asset_version,
                }),
            )
            .await?;
            sqlx::query("UPDATE flashcard_cards SET status = 'approved', updated_at = ? WHERE id = ?")
                .bind(timestamp())
                .bind(&card_id)
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &card_id,
                AuditActorKind::Teacher,
                Some(&teacher_id),
                "flashcard.approved",
                json!({
                    "studio_id": studio_id,
                    "from": status_to_db(card.status),
                    "to": status_to_db(FlashcardStatus::Approved),
                    "asset_id": current_asset_id,
                    "card_version": card.version,
                    "asset_version": asset_version,
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn publish_flashcard_for_studio(
        &self,
        studio_id: &str,
        teacher_id: &str,
        card_id: &str,
    ) -> Result<()> {
        require_identity("studio", studio_id)?;
        require_identity("teacher", teacher_id)?;
        let studio_id = studio_id.to_owned();
        let teacher_id = teacher_id.to_owned();
        let card_id = card_id.to_owned();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            assert_card_studio_in_transaction(&mut transaction, &studio_id, &card_id).await?;
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            require_locked_card_contract(&card, &self.catalog)?;
            if !can_transition(card.status, FlashcardStatus::Published) {
                return Err(anyhow!("invalid flashcard status transition"));
            }
            let current_asset_id = card
                .current_asset_id
                .ok_or_else(|| anyhow!("cannot publish without a current asset"))?;
            let asset_version = current_asset_contract_version_in_transaction(
                &mut transaction,
                &card,
                &current_asset_id,
            )
            .await?;
            require_latest_review_in_transaction(
                &mut transaction,
                &card_id,
                &current_asset_id,
                ReviewerKind::Automated,
                "automated",
            )
            .await?;
            require_latest_review_in_transaction(
                &mut transaction,
                &card_id,
                &current_asset_id,
                ReviewerKind::Teacher,
                "teacher",
            )
            .await?;
            sqlx::query("UPDATE flashcard_cards SET status = 'published', updated_at = ? WHERE id = ?")
                .bind(timestamp())
                .bind(&card_id)
                .execute(&mut *transaction)
                .await?;
            record_audit_in_transaction(
                &mut transaction,
                &card_id,
                AuditActorKind::Teacher,
                Some(&teacher_id),
                "flashcard.published",
                json!({
                    "studio_id": studio_id,
                    "from": status_to_db(card.status),
                    "to": status_to_db(FlashcardStatus::Published),
                    "asset_id": current_asset_id,
                    "card_version": card.version,
                    "asset_version": asset_version,
                }),
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn publish_flashcard(
        &self,
        card_id: &str,
        actor_kind: AuditActorKind,
        actor_id: Option<&str>,
    ) -> Result<()> {
        if actor_kind != AuditActorKind::Teacher
            || !matches!(actor_id, Some(id) if !id.trim().is_empty())
        {
            return Err(anyhow!("only an identified teacher may publish a flashcard"));
        }
        let card_id = card_id.to_owned();
        let actor_id = actor_id.map(str::to_owned);
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            let card = get_flashcard_in_transaction(&mut transaction, &card_id)
                .await?
                .ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
            if !can_transition(card.status, FlashcardStatus::Published) {
                return Err(anyhow!("invalid flashcard status transition"));
            }
            let current_asset_id = card.current_asset_id.ok_or_else(|| anyhow!("cannot publish without a current asset"))?;
            let review_passed: Option<i64> = sqlx::query_scalar(
                "SELECT reviews.passed FROM flashcard_reviews AS reviews JOIN flashcard_assets AS assets ON assets.id = reviews.asset_id WHERE reviews.card_id = ? AND assets.card_id = ? AND assets.id = ? ORDER BY reviews.created_at DESC, reviews.id DESC LIMIT 1",
            )
            .bind(&card_id)
            .bind(&card_id)
            .bind(&current_asset_id)
            .fetch_optional(&mut *transaction)
            .await?;
            if review_passed != Some(1) {
                return Err(anyhow!("cannot publish: current asset's latest review has not passed"));
            }
            sqlx::query("UPDATE flashcard_cards SET status = ?, updated_at = ? WHERE id = ?")
                .bind(status_to_db(FlashcardStatus::Published)).bind(timestamp()).bind(&card_id).execute(&mut *transaction).await?;
            record_audit_in_transaction(&mut transaction, &card_id, actor_kind, actor_id.as_deref(), "flashcard.published", json!({"from": status_to_db(card.status), "to": status_to_db(FlashcardStatus::Published), "asset_id": current_asset_id})).await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn record_audit_event(&self, card_id: &str, actor_kind: AuditActorKind, actor_id: Option<&str>, action: &str, payload: &Value) -> Result<()> {
        if action == "flashcard.created" {
            return Err(anyhow!(
                "flashcard ownership audit events may only be written during card creation"
            ));
        }
        let card_id = card_id.to_owned();
        let actor_id = actor_id.map(str::to_owned);
        let action = action.to_owned();
        let payload = payload.clone();
        self.runtime.block_on(async {
            let mut transaction = self.pool.begin().await?;
            record_audit_in_transaction(&mut transaction, &card_id, actor_kind, actor_id.as_deref(), &action, payload).await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    pub fn latest_brief_version(&self, card_id: &str) -> Result<Option<i64>> { self.scalar_i64("SELECT MAX(version) FROM visual_briefs WHERE card_id = ?", card_id) }
    pub fn asset_version(&self, asset_id: &str) -> Result<Option<i64>> { self.scalar_i64("SELECT version FROM flashcard_assets WHERE id = ?", asset_id) }
    pub fn audit_event_count(&self, card_id: &str) -> Result<i64> {
        let card_id = card_id.to_owned();
        self.runtime.block_on(async { Ok(sqlx::query_scalar("SELECT COUNT(*) FROM flashcard_audit_events WHERE card_id = ?").bind(card_id).fetch_one(&self.pool).await?) })
    }

    fn scalar_i64(&self, statement: &'static str, value: &str) -> Result<Option<i64>> {
        let value = value.to_owned();
        self.runtime.block_on(async { Ok(sqlx::query_scalar(statement).bind(value).fetch_one(&self.pool).await?) })
    }

}

async fn get_flashcard_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> Result<Option<FlashcardCard>> {
    sqlx::query_as::<_, CardRow>("SELECT id, source_exercise_id, category, teaching_copy_json, style_profile, character_id, current_asset_id, version, status FROM flashcard_cards WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut **transaction)
        .await?
        .map(card_from_row)
        .transpose()
}

async fn get_job_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> Result<Option<FlashcardJob>> {
    sqlx::query_as::<_, JobRow>(
        "SELECT id, card_id, kind, status, input_json, output_json, error
         FROM flashcard_jobs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut **transaction)
    .await?
    .map(job_from_row)
    .transpose()
}

async fn assert_card_studio_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    studio_id: &str,
    card_id: &str,
) -> Result<()> {
    let belongs_to_studio: i64 = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM flashcard_audit_events
             WHERE card_id = ?
               AND action = 'flashcard.created'
               AND json_extract(payload_json, '$.studio_id') = ?
         )",
    )
    .bind(card_id)
    .bind(studio_id)
    .fetch_one(&mut **transaction)
    .await?;
    if belongs_to_studio != 1 {
        return Err(anyhow!("flashcard not found: {card_id}"));
    }
    Ok(())
}

async fn assert_review_asset_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    review: &FlashcardReview,
) -> Result<i64> {
    let asset: Option<(String, String, i64)> =
        sqlx::query_as("SELECT card_id, brief_id, version FROM flashcard_assets WHERE id = ?")
            .bind(&review.asset_id)
            .fetch_optional(&mut **transaction)
            .await?;
    let (asset_card_id, brief_id, asset_version) =
        asset.ok_or_else(|| anyhow!("flashcard asset not found: {}", review.asset_id))?;
    let brief_card_id: Option<(String,)> =
        sqlx::query_as("SELECT card_id FROM visual_briefs WHERE id = ?")
            .bind(&brief_id)
            .fetch_optional(&mut **transaction)
            .await?;
    let brief_card_id =
        brief_card_id.ok_or_else(|| anyhow!("visual brief not found: {brief_id}"))?.0;
    if review.card_id != asset_card_id || asset_card_id != brief_card_id {
        return Err(anyhow!(
            "review asset does not belong to the review card"
        ));
    }
    Ok(asset_version)
}

async fn current_asset_contract_version_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    card: &FlashcardCard,
    asset_id: &str,
) -> Result<i64> {
    let asset: Option<(i64, String, String)> = sqlx::query_as(
        "SELECT assets.version, assets.status, briefs.brief_json
         FROM flashcard_assets AS assets
         JOIN visual_briefs AS briefs ON briefs.id = assets.brief_id
         WHERE assets.id = ? AND assets.card_id = ? AND briefs.card_id = ?",
    )
    .bind(asset_id)
    .bind(&card.id)
    .bind(&card.id)
    .fetch_optional(&mut **transaction)
    .await?;
    let (asset_version, asset_status, brief_json) =
        asset.ok_or_else(|| anyhow!("current flashcard asset not found: {asset_id}"))?;
    if asset_status != "approved" {
        return Err(anyhow!("cannot continue: current asset is not approved"));
    }
    let brief: VisualBrief =
        serde_json::from_str(&brief_json).context("parse current visual brief")?;
    let cheek_accent = brief
        .palette_json
        .get("cheekAccent")
        .and_then(Value::as_str);
    if brief.card_id != card.id
        || brief.exercise_id != card.source_exercise_id
        || brief.style_profile != LOCKED_STYLE_PROFILE
        || brief.character_id != LOCKED_CHARACTER_ID
        || brief.outfit != LOCKED_OUTFIT
        || cheek_accent != Some(DUSTY_ROSE_CHEEK_ACCENT)
    {
        return Err(anyhow!(
            "cannot continue: current asset visual contract does not match locked references"
        ));
    }
    Ok(asset_version)
}

async fn require_latest_review_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    card_id: &str,
    asset_id: &str,
    reviewer_kind: ReviewerKind,
    review_label: &str,
) -> Result<()> {
    let review_passed: Option<i64> = sqlx::query_scalar(
        "SELECT passed
         FROM flashcard_reviews
         WHERE card_id = ? AND asset_id = ? AND reviewer_kind = ?
         ORDER BY created_at DESC, rowid DESC
         LIMIT 1",
    )
    .bind(card_id)
    .bind(asset_id)
    .bind(reviewer_kind_to_db(&reviewer_kind))
    .fetch_optional(&mut **transaction)
    .await?;
    if review_passed != Some(1) {
        return Err(anyhow!(
            "cannot continue: current asset's latest {review_label} review has not passed"
        ));
    }
    Ok(())
}

async fn record_audit_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    card_id: &str,
    actor_kind: AuditActorKind,
    actor_id: Option<&str>,
    action: &str,
    payload: Value,
) -> Result<()> {
    sqlx::query("INSERT INTO flashcard_audit_events (card_id, actor_kind, actor_id, action, payload_json, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(card_id)
        .bind(actor_kind_to_db(&actor_kind))
        .bind(actor_id)
        .bind(action)
        .bind(serde_json::to_string(&payload)?)
        .bind(timestamp())
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

fn require_locked_card_contract(
    card: &FlashcardCard,
    catalog: &CanonicalCatalog,
) -> Result<()> {
    if catalog.find(&card.source_exercise_id).is_none()
        || card.style_profile != LOCKED_STYLE_PROFILE
        || card.character_id != LOCKED_CHARACTER_ID
    {
        return Err(anyhow!(
            "cannot continue: card source or locked visual contract is invalid"
        ));
    }
    Ok(())
}

fn require_identity(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} identity must not be empty"));
    }
    Ok(())
}

fn require_identifier(label: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(anyhow!("{label} identifier is invalid"));
    }
    Ok(())
}

fn is_safe_worker_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

async fn next_asset_version_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    card_id: &str,
) -> Result<i64> {
    let current: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM flashcard_assets WHERE card_id = ?",
    )
    .bind(card_id)
    .fetch_one(&mut **transaction)
    .await?;
    current
        .checked_add(1)
        .ok_or_else(|| anyhow!("flashcard asset version is exhausted"))
}

fn is_safe_worker_asset_path(value: &str) -> bool {
    if value.is_empty()
        || value.contains('\\')
        || value.chars().any(|character| character.is_control())
        || value.starts_with('/')
        || value.starts_with('~')
        || value.starts_with("file:")
        || value.starts_with("http:")
        || value.starts_with("https:")
        || value.starts_with("data:")
    {
        return false;
    }
    if let Some(key) = value.strip_prefix("object://mps-flashcards/") {
        return !key.is_empty()
            && !key.split('/').any(|part| part.is_empty() || part == "." || part == "..");
    }
    if value.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        return false;
    }
    value.starts_with("web/assets/flashcard-images/")
        || value.starts_with("docs/assets/flashcard-images/")
}

fn worker_claim_for_job(job: &FlashcardJob) -> Result<(String, String)> {
    let claim_id = job
        .input_json
        .get("claim_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("worker job claim is missing"))?;
    let claimed_by = job
        .input_json
        .get("claimed_by")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("worker job claimant is missing"))?;
    require_identifier("claim", claim_id)?;
    require_identifier("worker", claimed_by)?;
    Ok((claimed_by.to_owned(), claim_id.to_owned()))
}

fn validate_worker_claim(job: &FlashcardJob, worker_id: &str, claim_id: &str) -> Result<()> {
    require_identifier("claim", claim_id)?;
    let (claimed_by, persisted_claim_id) = worker_claim_for_job(job)?;
    if claimed_by != worker_id || persisted_claim_id != claim_id {
        return Err(anyhow!("worker claim token does not match the job"));
    }
    Ok(())
}

async fn validate_terminal_worker_retry(
    transaction: &mut Transaction<'_, Sqlite>,
    job: &FlashcardJob,
    card_id: &str,
    completion: &FlashcardWorkerCompletion,
) -> Result<()> {
    if job.status != completion.status {
        return Err(anyhow!("worker callback conflicts with the terminal job status"));
    }
    match &completion.status {
        FlashcardJobStatus::Failed => {
            if completion.asset.is_some() || completion.review.is_some() {
                return Err(anyhow!("failed worker callback cannot include an asset or review"));
            }
            let requested_error = completion
                .error_code
                .as_deref()
                .filter(|value| is_safe_worker_code(value))
                .unwrap_or("visual_worker_failed");
            if job.error.as_deref() != Some(requested_error) {
                return Err(anyhow!("worker callback conflicts with the persisted failure outcome"));
            }
        }
        FlashcardJobStatus::Succeeded => {
            let asset = completion
                .asset
                .as_ref()
                .ok_or_else(|| anyhow!("successful worker callback requires an asset"))?;
            let review = completion
                .review
                .as_ref()
                .ok_or_else(|| anyhow!("successful worker callback requires a review"))?;
            let brief_id = job
                .input_json
                .get("brief_id")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("generation job is missing its visual brief ID"))?;
            validate_worker_completion(card_id, brief_id, asset, review)?;
            if asset.status
                != if review.passed {
                    FlashcardAssetStatus::NeedsReview
                } else {
                    FlashcardAssetStatus::Rejected
                }
            {
                return Err(anyhow!("worker asset status does not match review result"));
            }
            let output = job
                .output_json
                .as_ref()
                .ok_or_else(|| anyhow!("terminal worker job output is missing"))?;
            if output.get("asset_id").and_then(Value::as_str) != Some(asset.id.as_str())
                || output.get("review_id").and_then(Value::as_str) != Some(review.id.as_str())
                || output.get("passed").and_then(Value::as_bool) != Some(review.passed)
            {
                return Err(anyhow!("worker callback conflicts with the persisted job outcome"));
            }
            let persisted_asset: Option<(String, String, Option<String>)> = sqlx::query_as(
                "SELECT brief_id, repo_path, provider_job_id
                 FROM flashcard_assets WHERE id = ? AND card_id = ?",
            )
            .bind(&asset.id)
            .bind(card_id)
            .fetch_optional(&mut **transaction)
            .await?;
            let (persisted_brief_id, persisted_path, persisted_provider_job_id) = persisted_asset
                .ok_or_else(|| anyhow!("persisted worker asset is missing"))?;
            if persisted_brief_id != asset.brief_id
                || persisted_path != asset.repo_path
                || persisted_provider_job_id != asset.provider_job_id
            {
                return Err(anyhow!("worker callback conflicts with the persisted asset outcome"));
            }
            let persisted_findings: String = sqlx::query_scalar(
                "SELECT findings_json FROM flashcard_reviews WHERE id = ? AND card_id = ? AND asset_id = ?",
            )
            .bind(&review.id)
            .bind(card_id)
            .bind(&asset.id)
            .fetch_optional(&mut **transaction)
            .await?
            .ok_or_else(|| anyhow!("persisted worker review is missing"))?;
            let requested_findings = serde_json::to_string(&review.findings_json)?;
            if persisted_findings != requested_findings {
                return Err(anyhow!("worker callback conflicts with the persisted review outcome"));
            }
        }
        FlashcardJobStatus::Queued | FlashcardJobStatus::Running => {
            return Err(anyhow!("worker callback must be terminal"));
        }
    }
    Ok(())
}

fn require_worker_visual_brief(brief: &VisualBrief, card: &FlashcardCard) -> Result<()> {
    let cheek_accent = brief
        .palette_json
        .get("cheekAccent")
        .and_then(Value::as_str);
    if brief.card_id != card.id
        || brief.exercise_id != card.source_exercise_id
        || brief.style_profile != LOCKED_STYLE_PROFILE
        || brief.character_id != LOCKED_CHARACTER_ID
        || brief.outfit != LOCKED_OUTFIT
        || cheek_accent != Some(DUSTY_ROSE_CHEEK_ACCENT)
    {
        return Err(anyhow!("worker visual brief does not match the locked contract"));
    }
    Ok(())
}

fn validate_worker_completion(
    card_id: &str,
    brief_id: &str,
    asset: &FlashcardAsset,
    review: &FlashcardReview,
) -> Result<()> {
    require_identifier("asset", &asset.id)?;
    require_identifier("brief", &asset.brief_id)?;
    require_identifier("review", &review.id)?;
    if asset.card_id != card_id
        || asset.brief_id != brief_id
        || review.card_id != card_id
        || review.asset_id != asset.id
        || review.reviewer_kind != ReviewerKind::Automated
        || review.reviewer_id.is_some()
    {
        return Err(anyhow!("worker completion ownership or reviewer contract is invalid"));
    }
    if asset.version < 1 || !is_safe_worker_asset_path(&asset.repo_path) {
        return Err(anyhow!("worker asset path or version is invalid"));
    }
    if asset
        .provider_job_id
        .as_deref()
        .is_some_and(|value| !is_safe_worker_code(value))
    {
        return Err(anyhow!("worker provider job ID is invalid"));
    }
    Ok(())
}

async fn transition_worker_card_status(
    transaction: &mut Transaction<'_, Sqlite>,
    card: &FlashcardCard,
    next: FlashcardStatus,
    worker_id: &str,
    studio_id: &str,
    job_id: &str,
) -> Result<()> {
    if !can_transition(card.status, next) {
        return Err(anyhow!("invalid worker flashcard status transition"));
    }
    sqlx::query("UPDATE flashcard_cards SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status_to_db(next))
        .bind(timestamp())
        .bind(&card.id)
        .execute(&mut **transaction)
        .await?;
    record_audit_in_transaction(
        transaction,
        &card.id,
        AuditActorKind::System,
        Some(worker_id),
        "flashcard.status_transitioned",
        json!({
            "studio_id": studio_id,
            "worker_id": worker_id,
            "job_id": job_id,
            "from": status_to_db(card.status),
            "to": status_to_db(next),
        }),
    )
    .await?;
    Ok(())
}

fn timestamp() -> String { SystemTime::now().duration_since(UNIX_EPOCH).expect("clock after epoch").as_millis().to_string() }
fn status_to_db(status: FlashcardStatus) -> &'static str { match status { FlashcardStatus::Draft => "draft", FlashcardStatus::Generating => "generating", FlashcardStatus::NeedsReview => "needs-review", FlashcardStatus::RevisionRequested => "revision-requested", FlashcardStatus::Approved => "approved", FlashcardStatus::Published => "published" } }
fn status_from_db(status: &str) -> Result<FlashcardStatus> { match status { "draft" => Ok(FlashcardStatus::Draft), "generating" => Ok(FlashcardStatus::Generating), "needs-review" => Ok(FlashcardStatus::NeedsReview), "revision-requested" => Ok(FlashcardStatus::RevisionRequested), "approved" => Ok(FlashcardStatus::Approved), "published" => Ok(FlashcardStatus::Published), _ => Err(anyhow!("unknown flashcard status: {status}")) } }
fn asset_status_to_db(status: &FlashcardAssetStatus) -> &'static str { match status { FlashcardAssetStatus::Generating => "generating", FlashcardAssetStatus::NeedsReview => "needs-review", FlashcardAssetStatus::Rejected => "rejected", FlashcardAssetStatus::Approved => "approved" } }
fn job_kind_to_db(kind: &FlashcardJobKind) -> &'static str { match kind { FlashcardJobKind::Generate => "generate", FlashcardJobKind::Review => "review", FlashcardJobKind::Regenerate => "regenerate" } }
fn reject_unsupported_job_kind(job: &FlashcardJob) -> Result<()> {
    if matches!(job.kind, FlashcardJobKind::Review) {
        return Err(anyhow!("review jobs are not supported; submit automated reviews through the trusted review endpoint"));
    }
    Ok(())
}
fn job_status_to_db(status: &FlashcardJobStatus) -> &'static str { match status { FlashcardJobStatus::Queued => "queued", FlashcardJobStatus::Running => "running", FlashcardJobStatus::Succeeded => "succeeded", FlashcardJobStatus::Failed => "failed" } }
fn reviewer_kind_to_db(kind: &ReviewerKind) -> &'static str { match kind { ReviewerKind::Automated => "automated", ReviewerKind::Teacher => "teacher" } }
fn actor_kind_to_db(kind: &AuditActorKind) -> &'static str { match kind { AuditActorKind::Teacher => "teacher", AuditActorKind::Chatgpt => "chatgpt", AuditActorKind::System => "system" } }
fn card_from_row(row: CardRow) -> Result<FlashcardCard> { Ok(FlashcardCard { id: row.id, source_exercise_id: row.source_exercise_id, category: row.category, teaching_copy_json: serde_json::from_str(&row.teaching_copy_json)?, style_profile: row.style_profile, character_id: row.character_id, current_asset_id: row.current_asset_id, version: row.version, status: status_from_db(&row.status)? }) }
fn job_kind_from_db(kind: &str) -> Result<FlashcardJobKind> { match kind { "generate" => Ok(FlashcardJobKind::Generate), "review" => Ok(FlashcardJobKind::Review), "regenerate" => Ok(FlashcardJobKind::Regenerate), _ => Err(anyhow!("unknown flashcard job kind: {kind}")) } }
fn job_status_from_db(status: &str) -> Result<FlashcardJobStatus> { match status { "queued" => Ok(FlashcardJobStatus::Queued), "running" => Ok(FlashcardJobStatus::Running), "succeeded" => Ok(FlashcardJobStatus::Succeeded), "failed" => Ok(FlashcardJobStatus::Failed), _ => Err(anyhow!("unknown flashcard job status: {status}")) } }
fn job_from_row(row: JobRow) -> Result<FlashcardJob> { Ok(FlashcardJob { id: row.id, card_id: row.card_id, kind: job_kind_from_db(&row.kind)?, status: job_status_from_db(&row.status)?, input_json: serde_json::from_str(&row.input_json)?, output_json: row.output_json.map(|value| serde_json::from_str(&value)).transpose()?, error: row.error }) }

fn asset_from_tuple(
    row: (String, String, String, String, Option<String>, i64, String),
) -> Result<FlashcardAsset> {
    Ok(FlashcardAsset {
        id: row.0,
        card_id: row.1,
        brief_id: row.2,
        repo_path: row.3,
        provider_job_id: row.4,
        version: row.5,
        status: asset_status_from_db(&row.6)?,
    })
}

fn review_from_tuple(
    row: (String, String, String, i64, String, i64, Option<String>),
) -> Result<FlashcardReview> {
    Ok(FlashcardReview {
        id: row.0,
        card_id: row.1,
        asset_id: row.2,
        passed: row.3 == 1,
        findings_json: serde_json::from_str(&row.4)?,
        reviewer_kind: ReviewerKind::Automated,
        reviewer_id: row.6,
    })
}

fn asset_status_from_db(status: &str) -> Result<FlashcardAssetStatus> {
    match status {
        "generating" => Ok(FlashcardAssetStatus::Generating),
        "needs-review" => Ok(FlashcardAssetStatus::NeedsReview),
        "rejected" => Ok(FlashcardAssetStatus::Rejected),
        "approved" => Ok(FlashcardAssetStatus::Approved),
        _ => Err(anyhow!("unknown flashcard asset status: {status}")),
    }
}
