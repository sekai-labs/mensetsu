use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, DropDown, Label, ListBox, ListBoxRow, Notebook, Orientation,
    ScrolledWindow, StringList,
};
use std::rc::Rc;
use crate::api_service::ApiService;
use crate::test_ids::ids;

pub struct WorkspaceDetailView {
    pub container: Box,
}

impl WorkspaceDetailView {
    pub fn new<FBack>(
        api: ApiService,
        workspace_id: String,
        workspace_title: String,
        on_back: FBack,
    ) -> Self
    where
        FBack: Fn() + 'static,
    {
        let container = Box::new(Orientation::Vertical, 12);
        container.set_widget_name(ids::DETAIL_VIEW);
        container.set_margin_start(24);
        container.set_margin_end(24);
        container.set_margin_top(16);
        container.set_margin_bottom(16);

        // Header with Back button and Workspace title
        let header = Box::new(Orientation::Horizontal, 12);
        let back_btn = Button::with_label("← Back");
        back_btn.set_widget_name(ids::DETAIL_BACK_BUTTON);

        let title_label = Label::new(Some(&workspace_title));
        title_label.set_widget_name(ids::DETAIL_TITLE_LABEL);
        title_label.add_css_class("title-2");
        title_label.set_hexpand(true);
        title_label.set_halign(gtk4::Align::Start);

        header.append(&back_btn);
        header.append(&title_label);

        back_btn.connect_clicked(move |_| {
            on_back();
        });

        // Error / Status banner
        let error_label = Label::new(None);
        error_label.set_widget_name(ids::DETAIL_ERROR_LABEL);
        error_label.add_css_class("error");
        error_label.set_visible(false);

        // Notebook with tabs
        let notebook = Notebook::new();
        notebook.set_vexpand(true);

        // Tab 1: Overview
        let overview_box = Box::new(Orientation::Vertical, 16);
        overview_box.set_widget_name(ids::DETAIL_TAB_OVERVIEW);
        overview_box.set_margin_start(16);
        overview_box.set_margin_end(16);
        overview_box.set_margin_top(16);
        let overview_summary = Label::new(Some("Workspace Overview & Status"));
        overview_summary.add_css_class("title-3");
        let overview_details = Label::new(Some(&format!("Workspace ID: {workspace_id}")));
        overview_box.append(&overview_summary);
        overview_box.append(&overview_details);
        let tab1_label = Label::new(Some("Overview"));
        notebook.append_page(&overview_box, Some(&tab1_label));

        // Tab 2: Match Analysis
        let match_box = Box::new(Orientation::Vertical, 12);
        match_box.set_widget_name(ids::DETAIL_TAB_MATCH);
        match_box.set_margin_start(16);
        match_box.set_margin_end(16);
        match_box.set_margin_top(16);

        let match_ratio_lbl = Label::new(Some("Overall Match: Loading..."));
        match_ratio_lbl.set_widget_name(ids::MATCH_RATIO_LABEL);
        match_ratio_lbl.add_css_class("title-3");
        match_ratio_lbl.set_halign(gtk4::Align::Start);
        match_box.append(&match_ratio_lbl);

        let match_scroll = ScrolledWindow::new();
        match_scroll.set_vexpand(true);
        let match_content = Box::new(Orientation::Vertical, 12);

        let strengths_title = Label::new(Some("Strengths:"));
        strengths_title.set_halign(gtk4::Align::Start);
        strengths_title.add_css_class("heading");
        let strengths_list = ListBox::new();
        strengths_list.set_widget_name(ids::MATCH_STRENGTHS_LIST);
        strengths_list.add_css_class("boxed-list");

        let gaps_title = Label::new(Some("Gaps:"));
        gaps_title.set_halign(gtk4::Align::Start);
        gaps_title.add_css_class("heading");
        let gaps_list = ListBox::new();
        gaps_list.set_widget_name(ids::MATCH_GAPS_LIST);
        gaps_list.add_css_class("boxed-list");

        let risks_title = Label::new(Some("Risks:"));
        risks_title.set_halign(gtk4::Align::Start);
        risks_title.add_css_class("heading");
        let risks_list = ListBox::new();
        risks_list.set_widget_name(ids::MATCH_RISKS_LIST);
        risks_list.add_css_class("boxed-list");

        match_content.append(&strengths_title);
        match_content.append(&strengths_list);
        match_content.append(&gaps_title);
        match_content.append(&gaps_list);
        match_content.append(&risks_title);
        match_content.append(&risks_list);
        match_scroll.set_child(Some(&match_content));
        match_box.append(&match_scroll);

        let tab2_label = Label::new(Some("Match"));
        notebook.append_page(&match_box, Some(&tab2_label));

        // Tab 3: Company Technical Intelligence
        let company_box = Box::new(Orientation::Vertical, 12);
        company_box.set_widget_name(ids::DETAIL_TAB_COMPANY);
        company_box.set_margin_start(16);
        company_box.set_margin_end(16);
        company_box.set_margin_top(16);

        let comp_summary_lbl = Label::new(Some("Company Overview: Loading..."));
        comp_summary_lbl.set_widget_name(ids::COMPANY_SUMMARY_LABEL);
        comp_summary_lbl.set_wrap(true);
        comp_summary_lbl.set_halign(gtk4::Align::Start);

        let comp_tech_lbl = Label::new(Some("Tech Stack: Loading..."));
        comp_tech_lbl.set_widget_name(ids::COMPANY_TECH_STACK);
        comp_tech_lbl.set_wrap(true);
        comp_tech_lbl.set_halign(gtk4::Align::Start);

        let citations_title = Label::new(Some("Sources & Citations:"));
        citations_title.set_halign(gtk4::Align::Start);
        citations_title.add_css_class("heading");

        let citations_list = ListBox::new();
        citations_list.set_widget_name(ids::COMPANY_CITATIONS_LIST);
        citations_list.add_css_class("boxed-list");

        company_box.append(&comp_summary_lbl);
        company_box.append(&comp_tech_lbl);
        company_box.append(&citations_title);
        company_box.append(&citations_list);

        let tab3_label = Label::new(Some("Company"));
        notebook.append_page(&company_box, Some(&tab3_label));

        // Tab 4: Learning Path
        let learning_box = Box::new(Orientation::Vertical, 12);
        learning_box.set_widget_name(ids::DETAIL_TAB_LEARNING);
        learning_box.set_margin_start(16);
        learning_box.set_margin_end(16);
        learning_box.set_margin_top(16);

        let learning_scroll = ScrolledWindow::new();
        learning_scroll.set_vexpand(true);
        let learning_list = ListBox::new();
        learning_list.set_widget_name(ids::LEARNING_LESSON_LIST);
        learning_list.add_css_class("boxed-list");
        learning_scroll.set_child(Some(&learning_list));
        learning_box.append(&learning_scroll);

        let tab4_label = Label::new(Some("Learning"));
        notebook.append_page(&learning_box, Some(&tab4_label));

        // Tab 5: Question Book
        let questions_box = Box::new(Orientation::Vertical, 12);
        questions_box.set_widget_name(ids::DETAIL_TAB_QUESTIONS);
        questions_box.set_margin_start(16);
        questions_box.set_margin_end(16);
        questions_box.set_margin_top(16);

        let diff_model = StringList::new(&["All Difficulties", "Easy", "Medium", "Hard"]);
        let filter_combo = DropDown::new(Some(diff_model), Option::<gtk4::Expression>::None);
        filter_combo.set_widget_name(ids::QUESTIONS_FILTER_COMBO);

        let q_scroll = ScrolledWindow::new();
        q_scroll.set_vexpand(true);
        let questions_list = ListBox::new();
        questions_list.set_widget_name(ids::QUESTIONS_LIST);
        questions_list.add_css_class("boxed-list");
        q_scroll.set_child(Some(&questions_list));

        questions_box.append(&filter_combo);
        questions_box.append(&q_scroll);

        let tab5_label = Label::new(Some("Questions"));
        notebook.append_page(&questions_box, Some(&tab5_label));

        container.append(&header);
        container.append(&error_label);
        container.append(&notebook);

        // Fetch Detail Data
        {
            let api = api.clone();
            let ws_id = workspace_id.clone();
            let err_lbl = error_label.clone();
            let ratio_lbl = match_ratio_lbl.clone();
            let s_list = strengths_list.clone();
            let g_list = gaps_list.clone();
            let r_list = risks_list.clone();
            let c_sum = comp_summary_lbl.clone();
            let c_tech = comp_tech_lbl.clone();
            let c_cite = citations_list.clone();
            let l_list = learning_list.clone();
            let q_list = questions_list.clone();
            let filter_combo = filter_combo.clone();

            glib::MainContext::default().spawn_local(async move {
                // 1. Match analysis
                match api.get_workspace_match(&ws_id).await {
                    Ok(match_data) => {
                        let pct = (match_data.overall_match_ratio * 100.0).round() as i32;
                        ratio_lbl.set_text(&format!("Overall Match: {}%", pct));

                        for s in match_data.strengths {
                            let row = ListBoxRow::new();
                            let l = Label::new(Some(&format!("✓ {} — {}", s.skill, s.evidence_quote)));
                            l.set_halign(gtk4::Align::Start);
                            l.set_wrap(true);
                            row.set_child(Some(&l));
                            s_list.append(&row);
                        }
                        for g in match_data.gaps {
                            let row = ListBoxRow::new();
                            let l = Label::new(Some(&format!("• {} — {}", g.skill, g.requirement_quote)));
                            l.set_halign(gtk4::Align::Start);
                            l.set_wrap(true);
                            row.set_child(Some(&l));
                            g_list.append(&row);
                        }
                        for r in match_data.risks {
                            let row = ListBoxRow::new();
                            let l = Label::new(Some(&format!("⚠ {}", r)));
                            l.set_halign(gtk4::Align::Start);
                            l.set_wrap(true);
                            row.set_child(Some(&l));
                            r_list.append(&row);
                        }
                    }
                    Err(e) => {
                        ratio_lbl.set_text("Match: Not ready or error");
                        err_lbl.set_text(&format!("Match data: {e}"));
                        err_lbl.set_visible(true);
                    }
                }

                // 2. Company research
                match api.get_workspace_company(&ws_id).await {
                    Ok(comp) => {
                        c_sum.set_text(comp["overview"].as_str().unwrap_or("Company research pending"));
                        let stack = comp["technology_stack"].as_array().into_iter().flatten()
                            .filter_map(|item| item.as_str()).collect::<Vec<_>>();
                        if stack.is_empty() {
                            c_tech.set_text("Tech stack: Research pending");
                        } else {
                            c_tech.set_text(&format!("Tech stack: {}", stack.join(", ")));
                        }
                        for src in comp["sources"].as_array().into_iter().flatten() {
                            let title = src["title"].as_str().unwrap_or("Untitled source");
                            let url = src["source_url"].as_str().or_else(|| src["url"].as_str()).unwrap_or("");
                            let row = ListBoxRow::new();
                            let label = Label::new(Some(&format!("{title}: {url}")));
                            label.set_halign(gtk4::Align::Start);
                            row.set_child(Some(&label));
                            c_cite.append(&row);
                        }
                    }
                    Err(e) => {
                        c_sum.set_text("Company research: Not available");
                        err_lbl.set_text(&format!("Company data: {e}"));
                        err_lbl.set_visible(true);
                    }
                }

                // 3. Learning path
                match api.get_workspace_learning(&ws_id).await {
                    Ok(lessons) => {
                        for lesson in lessons {
                            let row = ListBoxRow::new();
                            let b = Box::new(Orientation::Horizontal, 12);
                            let toggle = CheckButton::new();
                            let lesson_id = lesson.id.clone();
                            toggle.set_widget_name(&format!("{}{}", ids::LEARNING_TOGGLE_PREFIX, lesson_id));
                            toggle.set_active(lesson.completed.unwrap_or(false));

                            let title = Label::new(Some(&lesson.title));
                            title.set_hexpand(true);
                            title.set_halign(gtk4::Align::Start);

                            b.append(&toggle);
                            b.append(&title);
                            row.set_child(Some(&b));

                            let api_toggle = api.clone();
                            let ws_id_toggle = ws_id.clone();
                            let lid = lesson_id.clone();
                            toggle.connect_toggled(move |_| {
                                let api_toggle = api_toggle.clone();
                                let ws_id_toggle = ws_id_toggle.clone();
                                let lid = lid.clone();
                                glib::MainContext::default().spawn_local(async move {
                                    let _ = api_toggle.toggle_learning_unit(&ws_id_toggle, &lid).await;
                                });
                            });

                            l_list.append(&row);
                        }
                    }
                    Err(e) => {
                        err_lbl.set_text(&format!("Learning data: {e}"));
                        err_lbl.set_visible(true);
                    }
                }

                // 4. Questions
                match api.get_workspace_questions(&ws_id).await {
                    Ok(all_q) => {
                        let all_q = Rc::new(all_q);
                        let populate_q = {
                            let q_list = q_list.clone();
                            let all_q = all_q.clone();
                            let filter_combo = filter_combo.clone();

                            Rc::new(move || {
                                while let Some(c) = q_list.first_child() {
                                    q_list.remove(&c);
                                }
                                let selected_idx = filter_combo.selected();
                                for q in all_q.iter() {
                                    let diff_str = format!("{:?}", q.difficulty);
                                    let matches_filter = match selected_idx {
                                        1 => diff_str.to_lowercase().contains("easy"),
                                        2 => diff_str.to_lowercase().contains("medium"),
                                        3 => diff_str.to_lowercase().contains("hard"),
                                        _ => true,
                                    };
                                    if !matches_filter {
                                        continue;
                                    }

                                    let row = ListBoxRow::new();
                                    let vb = Box::new(Orientation::Vertical, 4);
                                    vb.set_margin_start(8);
                                    vb.set_margin_end(8);
                                    vb.set_margin_top(8);
                                    vb.set_margin_bottom(8);

                                    let q_label = Label::new(Some(&q.question));
                                    q_label.set_halign(gtk4::Align::Start);
                                    q_label.set_wrap(true);
                                    q_label.add_css_class("heading");

                                    let cat_label = Label::new(Some(&format!("[{:?}] Intent: {}", q.category, q.interviewer_intent)));
                                    cat_label.set_halign(gtk4::Align::Start);
                                    cat_label.add_css_class("dim-label");

                                    vb.append(&q_label);
                                    vb.append(&cat_label);
                                    row.set_child(Some(&vb));
                                    q_list.append(&row);
                                }
                            })
                        };

                        populate_q();
                        let pop = populate_q.clone();
                        filter_combo.connect_selected_notify(move |_| {
                            pop();
                        });
                    }
                    Err(e) => {
                        err_lbl.set_text(&format!("Questions data: {e}"));
                        err_lbl.set_visible(true);
                    }
                }
            });
        }

        Self { container }
    }
}
