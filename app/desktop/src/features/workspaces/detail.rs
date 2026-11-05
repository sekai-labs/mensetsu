use dioxus::prelude::*;
use mensetsu_contracts::models::{InterviewQuestion, LearningUnit, MatchAnalysis};
use crate::services::api::ApiService;
use crate::ui::test_ids::test_ids;

#[derive(Props, Clone, PartialEq)]
pub struct WorkspaceDetailProps {
    pub api: ApiService,
    pub workspace_id: String,
    pub workspace_title: String,
    pub on_back: EventHandler<()>,
}

#[component]
pub fn WorkspaceDetailView(props: WorkspaceDetailProps) -> Element {
    let mut active_tab = use_signal(|| "overview".to_string());
    let mut error_msg = use_signal(|| Option::<String>::None);

    // Tab data
    let mut match_data = use_signal(|| Option::<MatchAnalysis>::None);
    let mut company_data = use_signal(|| Option::<serde_json::Value>::None);
    let mut learning_data = use_signal(|| Option::<Vec<LearningUnit>>::None);
    let mut questions_data = use_signal(|| Option::<Vec<InterviewQuestion>>::None);
    let mut questions_difficulty = use_signal(|| "all".to_string());

    // Loaders
    let load_details = {
        let api = props.api.clone();
        let ws_id = props.workspace_id.clone();
        move || {
            error_msg.set(None);
            let api1 = api.clone();
            let id1 = ws_id.clone();
            spawn(async move {
                match api1.get_workspace_match(&id1).await {
                    Ok(m) => match_data.set(Some(m)),
                    Err(e) => error_msg.set(Some(format!("Failed to load match: {e}"))),
                }
            });

            let api2 = api.clone();
            let id2 = ws_id.clone();
            spawn(async move {
                match api2.get_workspace_company(&id2).await {
                    Ok(c) => company_data.set(Some(c)),
                    Err(e) => error_msg.set(Some(format!("Failed to load company: {e}"))),
                }
            });

            let api3 = api.clone();
            let id3 = ws_id.clone();
            spawn(async move {
                match api3.get_workspace_learning(&id3).await {
                    Ok(l) => learning_data.set(Some(l)),
                    Err(e) => error_msg.set(Some(format!("Failed to load learning: {e}"))),
                }
            });

            let api4 = api.clone();
            let id4 = ws_id.clone();
            spawn(async move {
                match api4.get_workspace_questions(&id4).await {
                    Ok(q) => questions_data.set(Some(q)),
                    Err(e) => error_msg.set(Some(format!("Failed to load questions: {e}"))),
                }
            });
        }
    };

    use_effect(move || {
        let mut load_fn = load_details.clone();
        load_fn();
    });

    let handle_toggle_lesson = {
        let api = props.api.clone();
        let ws_id = props.workspace_id.clone();
        move |lesson_id: String| {
            let api = api.clone();
            let ws_id = ws_id.clone();
            spawn(async move {
                if let Ok(_) = api.toggle_learning_unit(&ws_id, &lesson_id).await {
                    if let Ok(updated) = api.get_workspace_learning(&ws_id).await {
                        learning_data.set(Some(updated));
                    }
                }
            });
        }
    };

    let cur_tab = active_tab.read().clone();
    let current_diff = questions_difficulty.read().clone();
    let filtered_questions: Vec<InterviewQuestion> = questions_data
        .read()
        .as_ref()
        .map(|questions| {
            questions
                .iter()
                .filter(|q| {
                    if current_diff == "all" {
                        true
                    } else {
                        let q_diff = format!("{:?}", q.difficulty).to_lowercase();
                        q_diff.contains(&current_diff)
                    }
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    rsx! {
        div {
            class: "app-layout",
            id: "{test_ids::DETAIL_VIEW}",
            "data-testid": "{test_ids::DETAIL_VIEW}",
            // Header
            header { class: "app-header",
                div { style: "display: flex; align-items: center; gap: 16px;",
                    button {
                        class: "secondary",
                        id: "{test_ids::DETAIL_BACK_BUTTON}",
                        "data-testid": "{test_ids::DETAIL_BACK_BUTTON}",
                        onclick: move |_| props.on_back.call(()),
                        "← Back"
                    }
                    h1 {
                        class: "title-2",
                        id: "{test_ids::DETAIL_TITLE_LABEL}",
                        "data-testid": "{test_ids::DETAIL_TITLE_LABEL}",
                        "{props.workspace_title}"
                    }
                }
            }

            // Tabs bar
            div { style: "padding: 0 24px; background-color: var(--surface-color);",
                div { class: "tabs",
                    button {
                        class: if cur_tab == "overview" { "tab-btn active" } else { "tab-btn" },
                        id: "{test_ids::DETAIL_TAB_OVERVIEW}",
                        "data-testid": "{test_ids::DETAIL_TAB_OVERVIEW}",
                        onclick: move |_| active_tab.set("overview".to_string()),
                        "Overview"
                    }
                    button {
                        class: if cur_tab == "match" { "tab-btn active" } else { "tab-btn" },
                        id: "{test_ids::DETAIL_TAB_MATCH}",
                        "data-testid": "{test_ids::DETAIL_TAB_MATCH}",
                        onclick: move |_| active_tab.set("match".to_string()),
                        "Match"
                    }
                    button {
                        class: if cur_tab == "company" { "tab-btn active" } else { "tab-btn" },
                        id: "{test_ids::DETAIL_TAB_COMPANY}",
                        "data-testid": "{test_ids::DETAIL_TAB_COMPANY}",
                        onclick: move |_| active_tab.set("company".to_string()),
                        "Company"
                    }
                    button {
                        class: if cur_tab == "learning" { "tab-btn active" } else { "tab-btn" },
                        id: "{test_ids::DETAIL_TAB_LEARNING}",
                        "data-testid": "{test_ids::DETAIL_TAB_LEARNING}",
                        onclick: move |_| active_tab.set("learning".to_string()),
                        "Learning"
                    }
                    button {
                        class: if cur_tab == "questions" { "tab-btn active" } else { "tab-btn" },
                        id: "{test_ids::DETAIL_TAB_QUESTIONS}",
                        "data-testid": "{test_ids::DETAIL_TAB_QUESTIONS}",
                        onclick: move |_| active_tab.set("questions".to_string()),
                        "Questions"
                    }
                }
            }

            // Tab Content
            main { class: "app-content",
                if let Some(err) = error_msg.read().as_ref() {
                    div {
                        class: "error-banner",
                        id: "{test_ids::DETAIL_ERROR_LABEL}",
                        "data-testid": "{test_ids::DETAIL_ERROR_LABEL}",
                        "{err}"
                    }
                }

                // Tab 1: Overview
                if cur_tab == "overview" {
                    div { id: "{test_ids::DETAIL_TAB_OVERVIEW}",
                        h2 { class: "title-2", style: "margin-bottom: 12px;", "Workspace Overview" }
                        p { class: "dim-label", style: "margin-bottom: 16px;", "Workspace ID: {props.workspace_id}" }
                        div { class: "card", style: "max-width: 600px;",
                            h3 { style: "margin-bottom: 8px;", "{props.workspace_title}" }
                            p { class: "dim-label", "Target analysis, match score, company dossier, learning path, and practice questions are organized in the tabs above." }
                        }
                    }
                }

                // Tab 2: Match
                if cur_tab == "match" {
                    div { id: "{test_ids::DETAIL_TAB_MATCH}",
                        if let Some(m) = match_data.read().as_ref() {
                            div {
                                div {
                                    id: "{test_ids::MATCH_RATIO_LABEL}",
                                    "data-testid": "{test_ids::MATCH_RATIO_LABEL}",
                                    class: "title-2",
                                    style: "margin-bottom: 16px;",
                                    "Overall Match: {(m.overall_match_ratio * 100.0) as i64}%"
                                }

                                h3 { style: "margin-top: 16px; margin-bottom: 8px;", "Strengths:" }
                                div {
                                    id: "{test_ids::MATCH_STRENGTHS_LIST}",
                                    "data-testid": "{test_ids::MATCH_STRENGTHS_LIST}",
                                    class: "list-box",
                                    for s in &m.strengths {
                                        div { class: "list-item", "✓ {s.skill} — {s.evidence_quote}" }
                                    }
                                }

                                h3 { style: "margin-top: 20px; margin-bottom: 8px;", "Gaps:" }
                                div {
                                    id: "{test_ids::MATCH_GAPS_LIST}",
                                    "data-testid": "{test_ids::MATCH_GAPS_LIST}",
                                    class: "list-box",
                                    for g in &m.gaps {
                                        div { class: "list-item", "• {g.skill} — {g.requirement_quote}" }
                                    }
                                }

                                h3 { style: "margin-top: 20px; margin-bottom: 8px;", "Risks:" }
                                div {
                                    id: "{test_ids::MATCH_RISKS_LIST}",
                                    "data-testid": "{test_ids::MATCH_RISKS_LIST}",
                                    class: "list-box",
                                    for r in &m.risks {
                                        div { class: "list-item", "⚠ {r}" }
                                    }
                                }
                            }
                        } else {
                            p { class: "dim-label", "Loading match analysis..." }
                        }
                    }
                }

                // Tab 3: Company
                if cur_tab == "company" {
                    div { id: "{test_ids::DETAIL_TAB_COMPANY}",
                        if let Some(c) = company_data.read().as_ref() {
                            div {
                                div {
                                    id: "{test_ids::COMPANY_SUMMARY_LABEL}",
                                    "data-testid": "{test_ids::COMPANY_SUMMARY_LABEL}",
                                    style: "margin-bottom: 16px; font-size: 15px;",
                                    "{c.get(\"overview\").and_then(|v| v.as_str()).unwrap_or(\"No company overview available\")}"
                                }

                                h3 { style: "margin-top: 16px; margin-bottom: 8px;", "Technology Stack:" }
                                div {
                                    id: "{test_ids::COMPANY_TECH_STACK}",
                                    "data-testid": "{test_ids::COMPANY_TECH_STACK}",
                                    class: "list-box",
                                    if let Some(tech) = c.get("tech_stack").and_then(|v| v.as_array()) {
                                        for t in tech {
                                            div { class: "list-item", "{t.as_str().unwrap_or_default()}" }
                                        }
                                    }
                                }

                                h3 { style: "margin-top: 20px; margin-bottom: 8px;", "Sources & Citations:" }
                                div {
                                    id: "{test_ids::COMPANY_CITATIONS_LIST}",
                                    "data-testid": "{test_ids::COMPANY_CITATIONS_LIST}",
                                    class: "list-box",
                                    if let Some(citations) = c.get("citations").and_then(|v| v.as_array()) {
                                        for cite in citations {
                                            div { class: "list-item", "{cite.as_str().unwrap_or_default()}" }
                                        }
                                    }
                                }
                            }
                        } else {
                            p { class: "dim-label", "Loading company research..." }
                        }
                    }
                }

                // Tab 4: Learning
                if cur_tab == "learning" {
                    div { id: "{test_ids::DETAIL_TAB_LEARNING}",
                        if let Some(units) = learning_data.read().as_ref() {
                            div {
                                id: "{test_ids::LEARNING_LESSON_LIST}",
                                "data-testid": "{test_ids::LEARNING_LESSON_LIST}",
                                class: "list-box",
                                for unit in units {
                                    div {
                                        class: "list-item",
                                        key: "{unit.id}",
                                        div {
                                            strong { style: "display: block; font-size: 15px;", "{unit.title}" }
                                            p { class: "dim-label", "{unit.summary}" }
                                        }
                                        button {
                                            class: if unit.completed.unwrap_or(false) { "primary" } else { "secondary" },
                                            id: "{test_ids::LEARNING_TOGGLE_PREFIX}{unit.id}",
                                            "data-testid": "{test_ids::LEARNING_TOGGLE_PREFIX}{unit.id}",
                                            onclick: {
                                                let toggle_fn = handle_toggle_lesson.clone();
                                                let unit_id = unit.id.clone();
                                                move |_| toggle_fn(unit_id.clone())
                                            },
                                            if unit.completed.unwrap_or(false) { "Completed ✓" } else { "Mark Done" }
                                        }
                                    }
                                }
                            }
                        } else {
                            p { class: "dim-label", "Loading learning path..." }
                        }
                    }
                }

                // Tab 5: Questions
                if cur_tab == "questions" {
                    div { id: "{test_ids::DETAIL_TAB_QUESTIONS}",
                        div { style: "display: flex; gap: 8px; margin-bottom: 16px; align-items: center;",
                            label { class: "dim-label", "Filter by difficulty:" }
                            select {
                                id: "{test_ids::QUESTIONS_FILTER_COMBO}",
                                "data-testid": "{test_ids::QUESTIONS_FILTER_COMBO}",
                                style: "max-width: 180px;",
                                value: "{questions_difficulty}",
                                onchange: move |evt| questions_difficulty.set(evt.value().clone()),
                                option { value: "all", "All Difficulties" }
                                option { value: "beginner", "Beginner" }
                                option { value: "intermediate", "Intermediate" }
                                option { value: "advanced", "Advanced" }
                            }
                        }

                        if questions_data.read().is_some() {
                            div {
                                id: "{test_ids::QUESTIONS_LIST}",
                                "data-testid": "{test_ids::QUESTIONS_LIST}",
                                class: "list-box",
                                for q in filtered_questions {
                                    div {
                                        class: "list-item",
                                        key: "{q.id}",
                                        div {
                                            strong { style: "display: block; font-size: 15px; margin-bottom: 4px;", "{q.question}" }
                                            p { class: "dim-label", "[{q.category:?}] Intent: {q.interviewer_intent}" }
                                        }
                                        span {
                                            style: "font-size: 12px; padding: 4px 8px; border-radius: 4px; background-color: var(--surface-border);",
                                            "{q.difficulty:?}"
                                        }
                                    }
                                }
                            }
                        } else {
                            p { class: "dim-label", "Loading questions..." }
                        }
                    }
                }
            }
        }
    }
}
