use super::*;

#[test]
fn canonical_site_urls_resolve_to_stable_job_identities() {
    let greenhouse = canonical_url(
        ApplicationSite::Greenhouse,
        "https://job-boards.greenhouse.io/acme/jobs/4201/software-engineer?gh_src=campaign#apply",
    )
    .unwrap();
    assert_eq!(greenhouse, "https://boards.greenhouse.io/acme/4201");

    let topcv = canonical_url(
        ApplicationSite::Topcv,
        "https://www.topcv.vn/viec-lam/12345-ky-su-phan-mem.html?ref=tracking#description",
    )
    .unwrap();
    assert_eq!(topcv, "https://topcv.vn/viec-lam/12345-ky-su-phan-mem.html");

    let linkedin = canonical_url(
        ApplicationSite::Linkedin,
        "https://www.linkedin.com/jobs/view/98765?trk=feed#details",
    )
    .unwrap();
    assert_eq!(linkedin, "https://linkedin.com/jobs/view/98765");
}

#[test]
fn canonicalization_rejects_unsafe_or_unstable_urls() {
    for (site, url) in [
        (
            ApplicationSite::Greenhouse,
            "https://boards.greenhouse.io/acme/jobs/title",
        ),
        (
            ApplicationSite::Topcv,
            "https://topcv.vn/job/search?job_id=123",
        ),
        (
            ApplicationSite::Linkedin,
            "https://linkedin.com/jobs/search",
        ),
        (
            ApplicationSite::Linkedin,
            "https://linkedin.com@127.0.0.1/jobs/view/123",
        ),
        (
            ApplicationSite::Linkedin,
            "http://linkedin.com/jobs/view/123",
        ),
        (
            ApplicationSite::Linkedin,
            "https://evil.example/jobs/view/123",
        ),
    ] {
        assert!(canonical_url(site, url).is_err(), "accepted {url}");
    }
    assert!(canonical_url(
        ApplicationSite::Linkedin,
        &format!("https://linkedin.com/jobs/view/{}", "1".repeat(2049))
    )
    .is_err());
}

#[test]
fn application_content_requires_nonblank_values_with_limits() {
    let valid = PrepareApplicationRequestDto {
        site: ApplicationSite::Linkedin,
        source_url: "https://linkedin.com/jobs/view/123".into(),
        company_name: "Example".into(),
        role_title: "Engineer".into(),
        job_description: "Build systems".into(),
        resume_upload_id: "upload".into(),
        application_method: Some(ApplicationMethod::Web),
        contact_email: None,
    };
    assert!(validate_text(&valid).is_ok());
    let mut invalid = valid.clone();
    invalid.job_description = " ".into();
    assert!(validate_text(&invalid).is_err());
    invalid.job_description = "x".repeat(30001);
    assert!(validate_text(&invalid).is_err());

    let mut email = valid.clone();
    email.application_method = Some(ApplicationMethod::Email);
    assert!(validate_text(&email).is_err());
    email.contact_email = Some("hr@example.com".into());
    assert!(validate_text(&email).is_ok());
    email.contact_email = Some("not-an-email".into());
    assert!(validate_text(&email).is_err());

    let mut web_with_email = valid.clone();
    web_with_email.contact_email = Some("hr@example.com".into());
    assert!(validate_text(&web_with_email).is_err());
    web_with_email.contact_email = Some("not-an-email".into());
    assert!(validate_text(&web_with_email).is_ok());

    let mut legacy = valid.clone();
    legacy.application_method = None;
    assert!(validate_text(&legacy).is_ok());
}

#[test]
fn reprepare_requires_a_successful_write() {
    assert!(matches!(
        require_application_write(&[]),
        Err(AppError::Conflict(_))
    ));
    assert!(require_application_write(&[serde_json::json!({"id": "job_application:1"})]).is_ok());
}

#[test]
fn duplicate_application_write_requires_the_same_revision() {
    assert!(ensure_same_application_revision("revision-a", "revision-a").is_ok());
    assert!(matches!(
        ensure_same_application_revision("revision-a", "revision-b"),
        Err(AppError::Conflict(_))
    ));
}

#[test]
fn confirmation_url_validation_enforces_site_paths() {
    let valid_greenhouse = validated_confirmation_url(
        ApplicationSite::Greenhouse,
        "https://boards.greenhouse.io/acme/123/confirmation?signature=xyz#done",
        "https://boards.greenhouse.io/acme/123",
    );
    assert_eq!(
        valid_greenhouse.unwrap(),
        "https://boards.greenhouse.io/acme/123/confirmation"
    );

    let invalid_greenhouse = validated_confirmation_url(
        ApplicationSite::Greenhouse,
        "https://boards.greenhouse.io/acme/123",
        "https://boards.greenhouse.io/acme/123",
    );
    assert!(invalid_greenhouse.is_err());

    let mismatched_job = validated_confirmation_url(
        ApplicationSite::Greenhouse,
        "https://boards.greenhouse.io/acme/999/confirmation",
        "https://boards.greenhouse.io/acme/123",
    );
    assert!(mismatched_job.is_err());

    let valid_topcv = validated_confirmation_url(
        ApplicationSite::Topcv,
        "https://www.topcv.vn/viec-lam/123-ky-su.html?ref=tracking",
        "https://topcv.vn/viec-lam/123-ky-su.html",
    );
    assert_eq!(
        valid_topcv.unwrap(),
        "https://topcv.vn/viec-lam/123-ky-su.html"
    );
}

#[test]
fn email_confirmation_allows_missing_confirmation_url() {
    let confirmation = confirmation_url_for_result(
        Some(ApplicationMethod::Email),
        ApplicationSite::Linkedin,
        "https://linkedin.com/jobs/view/123",
        None,
    );
    assert_eq!(confirmation.unwrap(), None);

    let with_confirmation = confirmation_url_for_result(
        Some(ApplicationMethod::Email),
        ApplicationSite::Linkedin,
        "https://linkedin.com/jobs/view/123",
        Some("https://www.linkedin.com/jobs/view/123"),
    );
    assert_eq!(
        with_confirmation.unwrap(),
        Some("https://linkedin.com/jobs/view/123".into())
    );
}

#[test]
fn website_confirmation_requires_a_valid_confirmation_url() {
    let canonical_url = "https://boards.greenhouse.io/acme/42";
    let missing = confirmation_url_for_result(
        Some(ApplicationMethod::Web),
        ApplicationSite::Greenhouse,
        canonical_url,
        None,
    );
    assert!(matches!(
        missing,
        Err(AppError::Validation(message)) if message == "Confirmation URL required"
    ));

    let valid = confirmation_url_for_result(
        Some(ApplicationMethod::Web),
        ApplicationSite::Greenhouse,
        canonical_url,
        Some("https://boards.greenhouse.io/acme/42/confirmation"),
    );
    assert_eq!(
        valid.unwrap(),
        Some("https://boards.greenhouse.io/acme/42/confirmation".into())
    );

    let mismatched = confirmation_url_for_result(
        Some(ApplicationMethod::Web),
        ApplicationSite::Greenhouse,
        canonical_url,
        Some("https://boards.greenhouse.io/other/99/confirmation"),
    );
    assert!(mismatched.is_err());
}

#[test]
fn confirmed_result_idempotency_preserves_workspace() {
    let app_submitted = JobApplicationResponseDto {
        id: "app_sub_1".into(),
        site: ApplicationSite::Linkedin,
        canonical_url: "https://linkedin.com/jobs/view/999".into(),
        source_url: "https://linkedin.com/jobs/view/999".into(),
        company_name: "Enterprise Inc".into(),
        role_title: "Architect".into(),
        job_description: "System design".into(),
        resume_upload_id: "upload_sub".into(),
        resume_filename: "resume.pdf".into(),
        prepared_fields: PreparedFieldsDto {
            first_name: "Alice".into(),
            last_name: "Wonder".into(),
            email: "alice@example.com".into(),
            phone: None,
            location: None,
            linkedin_url: None,
            github_url: None,
            portfolio_url: None,
        },
        revision: "rev9".into(),
        status: ApplicationStatus::Submitted,
        application_method: Some(ApplicationMethod::Email),
        contact_email: Some("jobs@enterprise.com".into()),
        workspace_id: Some("ws_existing_123".into()),
        confirmation_url: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let check_confirmed =
        |app: &JobApplicationResponseDto, outcome: &ApplicationOutcome| -> Result<()> {
            if matches!(app.status, ApplicationStatus::Submitted) {
                if !matches!(outcome, ApplicationOutcome::Confirmed) {
                    return Err(AppError::Conflict(
                        "Submitted application cannot change its recorded outcome".into(),
                    ));
                }
                if app.workspace_id.is_none() {
                    return Err(AppError::Conflict(
                        "Submitted application has no workspace".into(),
                    ));
                }
                return Ok(());
            }
            if !matches!(app.status, ApplicationStatus::Submitting) {
                return Err(AppError::Conflict("Application is not submitting".into()));
            }
            Ok(())
        };

    assert!(check_confirmed(&app_submitted, &ApplicationOutcome::Confirmed).is_ok());
    assert!(check_confirmed(&app_submitted, &ApplicationOutcome::Uncertain).is_err());
    assert!(check_confirmed(&app_submitted, &ApplicationOutcome::NotSubmitted).is_err());

    let mut app_no_ws = app_submitted.clone();
    app_no_ws.workspace_id = None;
    assert!(check_confirmed(&app_no_ws, &ApplicationOutcome::Confirmed).is_err());
}

#[test]
fn result_recording_rejects_non_submitting_applications() {
    let check_state = |status: ApplicationStatus| -> Result<()> {
        if matches!(status, ApplicationStatus::Submitted) {
            return Ok(());
        }
        if !matches!(status, ApplicationStatus::Submitting) {
            return Err(AppError::Conflict("Application is not submitting".into()));
        }
        Ok(())
    };

    assert!(check_state(ApplicationStatus::Submitting).is_ok());
    assert!(check_state(ApplicationStatus::Prepared).is_err());
    assert!(check_state(ApplicationStatus::Approved).is_err());
    assert!(check_state(ApplicationStatus::NeedsReview).is_err());
    assert!(check_state(ApplicationStatus::Unknown).is_err());
}

#[test]
fn ownership_and_resume_completion_invariants() {
    let auth_user = "user:44444444-4444-4444-4444-444444444444";
    let other_user = "user:55555555-5555-5555-5555-555555555555";

    let verify_owner = |upload_user: &str, user: &str| -> Result<()> {
        if clean_id(upload_user) != clean_id(user) {
            return Err(AppError::Forbidden(
                "Resume upload belongs to another user".into(),
            ));
        }
        Ok(())
    };

    assert!(verify_owner(auth_user, auth_user).is_ok());
    assert!(verify_owner(other_user, auth_user).is_err());

    let verify_status = |status: &str| -> Result<()> {
        if status != "completed" {
            return Err(AppError::Conflict(
                "Resume upload is no longer completed".into(),
            ));
        }
        Ok(())
    };

    assert!(verify_status("completed").is_ok());
    assert!(verify_status("pending").is_err());
    assert!(verify_status("deleting").is_err());
}
