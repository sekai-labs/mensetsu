//
// WorkspaceDetailView.swift
// Mensetsu
//

import SwiftUI
import MensetsuContracts

public struct WorkspaceDetailView: View {
    public let workspaceId: String

    @StateObject private var viewModel: WorkspaceDetailViewModel
    @State private var selectedTab: DetailTab = .overview

    public enum DetailTab: String, CaseIterable, Identifiable {
        case overview = "Overview"
        case match = "Match"
        case company = "Company"
        case learning = "Learning"
        case questions = "Questions"

        public var id: String { rawValue }
    }

    public init(workspaceId: String) {
        self.workspaceId = workspaceId
        _viewModel = StateObject(wrappedValue: WorkspaceDetailViewModel(workspaceId: workspaceId))
    }

    public var body: some View {
        VStack(spacing: 0) {
            // Tab Picker
            Picker("Tabs", selection: $selectedTab) {
                ForEach(DetailTab.allCases) { tab in
                    Text(tab.rawValue).tag(tab)
                }
            }
            .pickerStyle(SegmentedPickerStyle())
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(Color(UIColor.systemBackground))
            .accessibilityIdentifier(AccessibilityID.detailTabBar)

            // Content Area
            if viewModel.isLoading && viewModel.workspace == nil {
                Spacer()
                ProgressView("Loading workspace...")
                    .accessibilityIdentifier(AccessibilityID.detailLoadingIndicator)
                Spacer()
            } else if let error = viewModel.errorMessage, viewModel.workspace == nil {
                Spacer()
                VStack(spacing: 12) {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.system(size: 40))
                        .foregroundColor(AppColors.rose)
                    Text("Error Loading Workspace")
                        .font(.headline)
                    Text(error)
                        .font(.subheadline)
                        .foregroundColor(AppColors.slate500)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal, 32)
                    Button("Retry") {
                        Task { await viewModel.loadAllData() }
                    }
                    .buttonStyle(.borderedProminent)
                }
                .accessibilityIdentifier(AccessibilityID.detailErrorBanner)
                Spacer()
            } else {
                TabView(selection: $selectedTab) {
                    overviewView
                        .tag(DetailTab.overview)
                        .accessibilityIdentifier(AccessibilityID.tabOverview)

                    matchView
                        .tag(DetailTab.match)
                        .accessibilityIdentifier(AccessibilityID.tabMatch)

                    companyView
                        .tag(DetailTab.company)
                        .accessibilityIdentifier(AccessibilityID.tabCompany)

                    learningView
                        .tag(DetailTab.learning)
                        .accessibilityIdentifier(AccessibilityID.tabLearning)

                    questionsView
                        .tag(DetailTab.questions)
                        .accessibilityIdentifier(AccessibilityID.tabQuestions)
                }
                .tabViewStyle(PageTabViewStyle(indexDisplayMode: .never))
            }
        }
        .navigationTitle(viewModel.workspace?.companyName ?? "Workspace")
        .navigationBarTitleDisplayMode(.inline)
        .task {
            await viewModel.loadAllData()
        }
        .accessibilityIdentifier(AccessibilityID.workspaceDetailScreen)
    }

    // MARK: - Overview
    private var overviewView: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let ws = viewModel.workspace {
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Text(ws.companyName)
                                .font(.system(size: 24, weight: .bold))
                                .accessibilityIdentifier(AccessibilityID.overviewCompanyName)
                            Spacer()
                            Text(ws.status.rawValue.uppercased())
                                .font(.system(size: 11, weight: .bold))
                                .foregroundColor(AppColors.emeraldDark)
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(AppColors.emeraldLight)
                                .cornerRadius(4)
                                .accessibilityIdentifier(AccessibilityID.overviewStatusBadge)
                        }

                        Text(ws.roleTitle)
                            .font(.system(size: 16))
                            .foregroundColor(AppColors.slate600)
                            .accessibilityIdentifier(AccessibilityID.overviewRoleTitle)

                        Divider()

                        Text("Readiness: \(ws.progressPercent)%")
                            .font(.system(size: 14, weight: .medium, design: .monospaced))
                            .foregroundColor(AppColors.slate500)

                        Text("Phase: \(ws.currentPhase.rawValue)")
                            .font(.system(size: 13))
                            .foregroundColor(AppColors.slate500)
                    }
                    .padding(16)
                    .background(Color(UIColor.secondarySystemGroupedBackground))
                    .cornerRadius(12)

                    // Metrics Grid
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                        metricCard(
                            title: "Match Ratio",
                            value: viewModel.match != nil ? "\(Int((viewModel.match!.overallMatchRatio) * 100))%" : "—",
                            subtitle: "Alignment score",
                            color: AppColors.primary
                        )
                        .accessibilityIdentifier(AccessibilityID.overviewMatchMetric)

                        metricCard(
                            title: "Skill Gaps",
                            value: "\(viewModel.match?.gaps.count ?? 0)",
                            subtitle: "Identified to bridge",
                            color: AppColors.amberDark
                        )
                        .accessibilityIdentifier(AccessibilityID.overviewGapsMetric)

                        metricCard(
                            title: "Interview Questions",
                            value: "\(viewModel.questions.count)",
                            subtitle: "Curated rubrics",
                            color: AppColors.emeraldDark
                        )
                        .accessibilityIdentifier(AccessibilityID.overviewQuestionsMetric)

                        metricCard(
                            title: "Learning Lessons",
                            value: "\(viewModel.lessons.count)",
                            subtitle: "\(viewModel.lessons.filter { $0.completed == true }.count) completed",
                            color: AppColors.primaryDark
                        )
                    }
                }
            }
            .padding(16)
        }
    }

    private func metricCard(title: String, value: String, subtitle: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(AppColors.slate500)
            Text(value)
                .font(.system(size: 26, weight: .bold, design: .rounded))
                .foregroundColor(color)
            Text(subtitle)
                .font(.system(size: 11))
                .foregroundColor(AppColors.slate400)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(Color(UIColor.secondarySystemGroupedBackground))
        .cornerRadius(10)
    }

    // MARK: - Match Tab
    private var matchView: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                if let match = viewModel.match {
                    // Overall Match Card
                    HStack {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Overall Fit Ratio")
                                .font(.headline)
                            Text("Synthesized from résumé claims and job description requirements.")
                                .font(.caption)
                                .foregroundColor(AppColors.slate500)
                        }
                        Spacer()
                        Text("\(Int(match.overallMatchRatio * 100))%")
                            .font(.system(size: 32, weight: .bold, design: .rounded))
                            .foregroundColor(AppColors.primary)
                            .accessibilityIdentifier(AccessibilityID.matchRatioText)
                    }
                    .padding(16)
                    .background(Color(UIColor.secondarySystemGroupedBackground))
                    .cornerRadius(10)

                    // Strengths
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Demonstrated Strengths")
                            .font(.headline)
                            .foregroundColor(AppColors.emeraldDark)

                        if match.strengths.isEmpty {
                            Text("No strengths recorded.")
                                .font(.subheadline)
                                .foregroundColor(AppColors.slate500)
                        } else {
                            ForEach(match.strengths, id: \.skill) { s in
                                VStack(alignment: .leading, spacing: 6) {
                                    HStack {
                                        Text(s.skill)
                                            .font(.system(size: 15, weight: .semibold))
                                        Spacer()
                                        Text(s.relevance.uppercased())
                                            .font(.system(size: 10, weight: .bold))
                                            .foregroundColor(AppColors.emeraldDark)
                                            .padding(.horizontal, 6)
                                            .padding(.vertical, 2)
                                            .background(AppColors.emeraldLight)
                                            .cornerRadius(4)
                                    }
                                    if !s.evidenceQuote.isEmpty {
                                        Text("\"\(s.evidenceQuote)\"")
                                            .font(.system(size: 13, design: .serif))
                                            .foregroundColor(AppColors.slate600)
                                            .padding(8)
                                            .background(Color(UIColor.tertiarySystemGroupedBackground))
                                            .cornerRadius(6)
                                    }
                                }
                                .padding(12)
                                .background(Color(UIColor.secondarySystemGroupedBackground))
                                .cornerRadius(8)
                            }
                        }
                    }
                    .accessibilityIdentifier(AccessibilityID.matchStrengthsList)

                    // Gaps
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Identified Skill Gaps")
                            .font(.headline)
                            .foregroundColor(AppColors.amberDark)

                        if match.gaps.isEmpty {
                            Text("No gaps identified.")
                                .font(.subheadline)
                                .foregroundColor(AppColors.slate500)
                        } else {
                            ForEach(match.gaps, id: \.skill) { g in
                                VStack(alignment: .leading, spacing: 6) {
                                    HStack {
                                        Text(g.skill)
                                            .font(.system(size: 15, weight: .semibold))
                                        Spacer()
                                        Text(g.importance.uppercased())
                                            .font(.system(size: 10, weight: .bold))
                                            .foregroundColor(AppColors.amberDark)
                                            .padding(.horizontal, 6)
                                            .padding(.vertical, 2)
                                            .background(AppColors.amberLight)
                                            .cornerRadius(4)
                                    }
                                    if !g.recommendedFocus.isEmpty {
                                        Text(g.recommendedFocus)
                                            .font(.system(size: 13))
                                            .foregroundColor(AppColors.slate600)
                                    }
                                }
                                .padding(12)
                                .background(Color(UIColor.secondarySystemGroupedBackground))
                                .cornerRadius(8)
                            }
                        }
                    }
                    .accessibilityIdentifier(AccessibilityID.matchGapsList)

                    // Risks
                    if !match.risks.isEmpty {
                        VStack(alignment: .leading, spacing: 10) {
                            Text("Potential Interview Risks")
                                .font(.headline)
                                .foregroundColor(AppColors.roseDark)

                            ForEach(match.risks, id: \.self) { r in
                                Text("• \(r)")
                                    .font(.system(size: 14))
                                    .foregroundColor(AppColors.slate700)
                            }
                        }
                        .padding(12)
                        .background(Color(UIColor.secondarySystemGroupedBackground))
                        .cornerRadius(8)
                        .accessibilityIdentifier(AccessibilityID.matchRisksList)
                    }
                } else {
                    Text("No match analysis available.")
                        .foregroundColor(AppColors.slate500)
                }
            }
            .padding(16)
        }
    }

    // MARK: - Company Tab
    private var companyView: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let comp = viewModel.company {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Executive Overview")
                            .font(.headline)
                        Text(comp.overview)
                            .font(.system(size: 14))
                            .foregroundColor(AppColors.slate700)
                    }
                    .padding(14)
                    .background(Color(UIColor.secondarySystemGroupedBackground))
                    .cornerRadius(8)
                    .accessibilityIdentifier(AccessibilityID.companyOverviewText)

                    VStack(alignment: .leading, spacing: 8) {
                        Text("Verified Tech Stack")
                            .font(.headline)

                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 100))], spacing: 8) {
                            ForEach(comp.technologyStack, id: \.self) { tech in
                                Text(tech)
                                    .font(.system(size: 12, design: .monospaced))
                                    .padding(.horizontal, 10)
                                    .padding(.vertical, 6)
                                    .background(Color(UIColor.tertiarySystemGroupedBackground))
                                    .cornerRadius(6)
                            }
                        }
                    }
                    .padding(14)
                    .background(Color(UIColor.secondarySystemGroupedBackground))
                    .cornerRadius(8)
                    .accessibilityIdentifier(AccessibilityID.companyTechStackList)

                    if !comp.engineeringCulture.isEmpty {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Engineering Culture")
                                .font(.headline)
                            ForEach(comp.engineeringCulture, id: \.self) { item in
                                Text("• \(item)")
                                    .font(.system(size: 14))
                                    .foregroundColor(AppColors.slate700)
                            }
                        }
                        .padding(14)
                        .background(Color(UIColor.secondarySystemGroupedBackground))
                        .cornerRadius(8)
                        .accessibilityIdentifier(AccessibilityID.companyCultureList)
                    }

                    if !comp.sources.isEmpty {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Source Citations")
                                .font(.headline)
                            ForEach(comp.sources, id: \.url) { src in
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(src.title)
                                        .font(.system(size: 13, weight: .medium))
                                    Text(src.url)
                                        .font(.system(size: 11))
                                        .foregroundColor(AppColors.primary)
                                }
                                .padding(.vertical, 4)
                            }
                        }
                        .padding(14)
                        .background(Color(UIColor.secondarySystemGroupedBackground))
                        .cornerRadius(8)
                        .accessibilityIdentifier(AccessibilityID.companyCitationsList)
                    }
                } else {
                    Text("No company dossier available.")
                        .foregroundColor(AppColors.slate500)
                }
            }
            .padding(16)
        }
    }

    // MARK: - Learning Tab
    private var learningView: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                if viewModel.lessons.isEmpty {
                    Text("No learning units generated yet.")
                        .foregroundColor(AppColors.slate500)
                        .padding(16)
                } else {
                    ForEach(viewModel.lessons, id: \.id) { lesson in
                        HStack(alignment: .top, spacing: 12) {
                            Button(action: {
                                Task { await viewModel.toggleLesson(lessonId: lesson.id) }
                            }) {
                                Image(systemName: lesson.completed == true ? "checkmark.circle.fill" : "circle")
                                    .font(.system(size: 22))
                                    .foregroundColor(lesson.completed == true ? AppColors.emerald : AppColors.slate400)
                            }
                            .accessibilityIdentifier(AccessibilityID.lessonToggle(id: lesson.id))

                            VStack(alignment: .leading, spacing: 4) {
                                Text(lesson.title)
                                    .font(.system(size: 15, weight: .semibold))
                                    .foregroundColor(AppColors.slate900)
                                Text(lesson.summary)
                                    .font(.system(size: 13))
                                    .foregroundColor(AppColors.slate600)
                                HStack {
                                    Text("Skill: \(lesson.skill)")
                                        .font(.system(size: 11))
                                        .foregroundColor(AppColors.primary)
                                    Spacer()
                                    Text("~\(lesson.estimatedMinutes) min")
                                        .font(.system(size: 11, design: .monospaced))
                                        .foregroundColor(AppColors.slate400)
                                }
                            }
                        }
                        .padding(14)
                        .background(Color(UIColor.secondarySystemGroupedBackground))
                        .cornerRadius(10)
                        .accessibilityIdentifier(AccessibilityID.lessonRow(id: lesson.id))
                    }
                }
            }
            .padding(16)
        }
        .accessibilityIdentifier(AccessibilityID.learningUnitsList)
    }

    // MARK: - Questions Tab
    @State private var selectedDifficulty: String = "all"

    private var filteredQuestions: [InterviewQuestion] {
        viewModel.questions.filter { q in
            if selectedDifficulty == "all" { return true }
            return q.difficulty.rawValue.lowercased() == selectedDifficulty.lowercased()
        }
    }

    private var questionsView: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                Picker("Difficulty", selection: $selectedDifficulty) {
                    Text("All Difficulties").tag("all")
                    Text("Easy").tag("easy")
                    Text("Medium").tag("medium")
                    Text("Hard").tag("hard")
                }
                .pickerStyle(SegmentedPickerStyle())
                .accessibilityIdentifier(AccessibilityID.questionsDifficultyPicker)

                if filteredQuestions.isEmpty {
                    Text("No interview questions matching filter.")
                        .foregroundColor(AppColors.slate500)
                        .padding(16)
                } else {
                    ForEach(filteredQuestions, id: \.id) { q in
                        VStack(alignment: .leading, spacing: 8) {
                            HStack {
                                Text(q.category.rawValue.uppercased())
                                    .font(.system(size: 10, weight: .bold))
                                    .foregroundColor(AppColors.primary)
                                Spacer()
                                Text(q.difficulty.rawValue.uppercased())
                                    .font(.system(size: 10, weight: .bold))
                                    .foregroundColor(difficultyColor(q.difficulty.rawValue))
                            }

                            Text(q.question)
                                .font(.system(size: 15, weight: .semibold))

                            Text("Intent: \(q.interviewerIntent)")
                                .font(.system(size: 12))
                                .foregroundColor(AppColors.slate500)

                            if !q.strongAnswerRubric.isEmpty {
                                Divider()
                                Text("Strong Answer Rubric:")
                                    .font(.system(size: 12, weight: .medium))
                                    .foregroundColor(AppColors.emeraldDark)
                                ForEach(q.strongAnswerRubric, id: \.self) { item in
                                    Text("✓ \(item)")
                                        .font(.system(size: 12))
                                        .foregroundColor(AppColors.slate700)
                                }
                            }
                        }
                        .padding(14)
                        .background(Color(UIColor.secondarySystemGroupedBackground))
                        .cornerRadius(10)
                        .accessibilityIdentifier(AccessibilityID.questionRow(id: q.id))
                    }
                }
            }
            .padding(16)
        }
        .accessibilityIdentifier(AccessibilityID.questionsList)
    }

    private func difficultyColor(_ diff: String) -> Color {
        switch diff.lowercased() {
        case "easy": return AppColors.emeraldDark
        case "medium": return AppColors.amberDark
        case "hard": return AppColors.roseDark
        default: return AppColors.slate500
        }
    }
}

@MainActor
public final class WorkspaceDetailViewModel: ObservableObject {
    public let workspaceId: String

    @Published public var workspace: WorkspaceResponseDto?
    @Published public var match: MatchAnalysis?
    @Published public var company: CompanyResearch?
    @Published public var questions: [InterviewQuestion] = []
    @Published public var lessons: [LearningUnit] = []

    @Published public var isLoading: Bool = false
    @Published public var errorMessage: String? = nil

    private let service: WorkspaceServiceProtocol

    public init(workspaceId: String, service: WorkspaceServiceProtocol = WorkspaceService()) {
        self.workspaceId = workspaceId
        self.service = service
    }

    public func loadAllData() async {
        isLoading = true
        errorMessage = nil
        do {
            async let wsFetch = service.getWorkspace(id: workspaceId)
            async let matchFetch = service.getWorkspaceMatch(id: workspaceId)
            async let companyFetch = service.getWorkspaceCompany(id: workspaceId)
            async let questionsFetch = service.getWorkspaceQuestions(id: workspaceId)
            async let lessonsFetch = service.getWorkspaceLearning(id: workspaceId)

            let (ws, m, c, q, l) = try await (wsFetch, matchFetch, companyFetch, questionsFetch, lessonsFetch)
            self.workspace = ws
            self.match = m
            self.company = c
            self.questions = q
            self.lessons = l
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    public func toggleLesson(lessonId: String) async {
        do {
            try await service.toggleLearningUnit(id: workspaceId, lessonId: lessonId)
            // Toggle local state
            lessons = lessons.map { lesson in
                if lesson.id == lessonId {
                    var updated = lesson
                    updated.completed = !(lesson.completed ?? false)
                    return updated
                }
                return lesson
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
