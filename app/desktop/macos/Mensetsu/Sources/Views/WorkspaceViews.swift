//
// WorkspaceViews.swift
// Mensetsu macOS
//

import SwiftUI
import MensetsuContracts

public struct WorkspaceDetailView: View {
    @StateObject var viewModel: WorkspaceDetailViewModel
    
    public init(workspaceId: String) {
        _viewModel = StateObject(wrappedValue: WorkspaceDetailViewModel(workspaceId: workspaceId))
    }
    
    public var body: some View {
        VStack(spacing: 0) {
            // Tab bar
            HStack(spacing: 4) {
                ForEach(WorkspaceDetailTab.allCases) { tab in
                    Button(action: {
                        viewModel.currentTab = tab
                        if tab == .settings {
                            viewModel.loadLlmSettings()
                        }
                    }) {
                        HStack(spacing: 6) {
                            Image(systemName: tab.icon)
                            Text(tab.title)
                        }
                        .padding(.vertical, 8)
                        .padding(.horizontal, 12)
                        .background(viewModel.currentTab == tab ? Color.accentColor.opacity(0.15) : Color.clear)
                        .cornerRadius(6)
                    }
                    .buttonStyle(.plain)
                    .accessibilityIdentifier(tabIdentifier(for: tab))
                }
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(Color(NSColor.windowBackgroundColor))
            .accessibilityIdentifier(AccessibilityIdentifiers.detailTabBar)
            
            Divider()
            
            // Tab content
            if viewModel.isLoading {
                VStack(spacing: 12) {
                    ProgressView()
                    Text("Loading workspace intelligence...")
                        .font(.callout)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .accessibilityIdentifier(AccessibilityIdentifiers.detailLoadingIndicator)
            } else if let error = viewModel.errorMessage {
                VStack(spacing: 16) {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.system(size: 36))
                        .foregroundColor(.orange)
                    Text(error)
                        .font(.body)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal)
                        .accessibilityIdentifier(AccessibilityIdentifiers.detailErrorMessage)
                    Button("Retry") {
                        viewModel.loadAllData()
                    }
                    .buttonStyle(.borderedProminent)
                    .accessibilityIdentifier(AccessibilityIdentifiers.detailRetryButton)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    Group {
                        switch viewModel.currentTab {
                        case .overview:
                            OverviewTabView(viewModel: viewModel)
                        case .match:
                            MatchAnalysisTabView(viewModel: viewModel)
                        case .company:
                            CompanyDossierTabView(viewModel: viewModel)
                        case .learning:
                            LearningPathTabView(viewModel: viewModel)
                        case .questions:
                            QuestionBookTabView(viewModel: viewModel)
                        case .settings:
                            LlmSettingsTabView(viewModel: viewModel)
                        }
                    }
                    .padding(24)
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .accessibilityIdentifier(AccessibilityIdentifiers.detailContainer)
        .onAppear {
            viewModel.loadAllData()
        }
    }
    
    private func tabIdentifier(for tab: WorkspaceDetailTab) -> String {
        switch tab {
        case .overview: return AccessibilityIdentifiers.tabOverview
        case .match: return AccessibilityIdentifiers.tabMatch
        case .company: return AccessibilityIdentifiers.tabCompany
        case .learning: return AccessibilityIdentifiers.tabLearning
        case .questions: return AccessibilityIdentifiers.tabQuestions
        case .settings: return AccessibilityIdentifiers.tabLlmSettings
        }
    }
}

// MARK: - Tab Views

struct OverviewTabView: View {
    @ObservedObject var viewModel: WorkspaceDetailViewModel
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            if let ws = viewModel.workspace {
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 6) {
                        Text(ws.companyName)
                            .font(.system(size: 26, weight: .bold))
                            .accessibilityIdentifier(AccessibilityIdentifiers.overviewCompanyName)
                        Text(ws.roleTitle)
                            .font(.title3)
                            .foregroundColor(.secondary)
                            .accessibilityIdentifier(AccessibilityIdentifiers.overviewRoleTitle)
                    }
                    Spacer()
                    VStack(alignment: .trailing, spacing: 6) {
                        Text(ws.status.rawValue.uppercased())
                            .font(.caption)
                            .fontWeight(.bold)
                            .padding(.horizontal, 10)
                            .padding(.vertical, 4)
                            .background(Color.green.opacity(0.15))
                            .foregroundColor(.green)
                            .cornerRadius(4)
                            .accessibilityIdentifier(AccessibilityIdentifiers.overviewStatusBadge)
                        
                        Text("Readiness: \(ws.progressPercent)%")
                            .font(.system(.caption, design: .monospaced))
                            .foregroundColor(.secondary)
                            .accessibilityIdentifier(AccessibilityIdentifiers.overviewReadiness)
                    }
                }
                .padding(20)
                .background(Color(NSColor.controlBackgroundColor))
                .cornerRadius(8)
                
                // Metrics Summary
                HStack(spacing: 16) {
                    MetricBox(
                        title: "Match Ratio",
                        value: viewModel.match != nil ? "\(Int(round((viewModel.match?.overallMatchRatio ?? 0) * 100)))%" : "N/A",
                        icon: "chart.bar.fill"
                    )
                    MetricBox(
                        title: "Strengths Identified",
                        value: "\(viewModel.match?.strengths.count ?? 0)",
                        icon: "checkmark.seal.fill"
                    )
                    MetricBox(
                        title: "Skill Gaps",
                        value: "\(viewModel.match?.gaps.count ?? 0)",
                        icon: "exclamationmark.triangle.fill"
                    )
                    MetricBox(
                        title: "Learning Lessons",
                        value: "\(viewModel.learningUnits.filter { $0.completed == true }.count)/\(viewModel.learningUnits.count)",
                        icon: "graduationcap.fill"
                    )
                }
                
                // Job Description Preview if available
                if let jd = ws.jobDescription, !jd.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Job Description Requirements")
                            .font(.headline)
                        Text(jd)
                            .font(.body)
                            .foregroundColor(.secondary)
                            .padding(14)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Color(NSColor.controlBackgroundColor))
                            .cornerRadius(6)
                    }
                }
            } else {
                Text("No workspace data found.")
                    .foregroundColor(.secondary)
            }
        }
    }
}

struct MetricBox: View {
    let title: String
    let value: String
    let icon: String
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(title)
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Image(systemName: icon)
                    .foregroundColor(.accentColor)
            }
            Text(value)
                .font(.title2)
                .fontWeight(.bold)
        }
        .padding(16)
        .frame(maxWidth: .infinity)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct MatchAnalysisTabView: View {
    @ObservedObject var viewModel: WorkspaceDetailViewModel
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            if let match = viewModel.match {
                HStack {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("CV & JD Match Analysis")
                            .font(.title2)
                            .fontWeight(.bold)
                        Text("Multi-dimensional alignment evaluated against job description requirements.")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    Text("\(Int(round(match.overallMatchRatio * 100)))% Match")
                        .font(.system(.title3, design: .monospaced))
                        .fontWeight(.bold)
                        .padding(.horizontal, 14)
                        .padding(.vertical, 8)
                        .background(Color.accentColor.opacity(0.15))
                        .foregroundColor(.accentColor)
                        .cornerRadius(6)
                        .accessibilityIdentifier(AccessibilityIdentifiers.matchRatioBadge)
                }
                
                // Strengths
                VStack(alignment: .leading, spacing: 12) {
                    Text("Verified Strengths (\(match.strengths.count))")
                        .font(.headline)
                        .foregroundColor(.green)
                    
                    VStack(spacing: 8) {
                        ForEach(match.strengths, id: \.skill) { s in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(s.skill)
                                    .font(.subheadline)
                                    .fontWeight(.semibold)
                                Text(s.evidence)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            .padding(12)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Color(NSColor.controlBackgroundColor))
                            .cornerRadius(6)
                        }
                    }
                    .accessibilityIdentifier(AccessibilityIdentifiers.matchStrengthsList)
                }
                
                // Gaps
                VStack(alignment: .leading, spacing: 12) {
                    Text("Skill Gaps (\(match.gaps.count))")
                        .font(.headline)
                        .foregroundColor(.orange)
                    
                    VStack(spacing: 8) {
                        ForEach(match.gaps, id: \.skill) { g in
                            VStack(alignment: .leading, spacing: 4) {
                                HStack {
                                    Text(g.skill)
                                        .font(.subheadline)
                                        .fontWeight(.semibold)
                                    Spacer()
                                    Text(g.severity)
                                        .font(.caption2)
                                        .padding(.horizontal, 6)
                                        .padding(.vertical, 2)
                                        .background(Color.orange.opacity(0.2))
                                        .foregroundColor(.orange)
                                        .cornerRadius(4)
                                }
                                Text(g.remediation)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            .padding(12)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Color(NSColor.controlBackgroundColor))
                            .cornerRadius(6)
                        }
                    }
                    .accessibilityIdentifier(AccessibilityIdentifiers.matchGapsList)
                }
                
                // Risks
                if !match.risks.isEmpty {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Risk Signals")
                            .font(.headline)
                            .foregroundColor(.red)
                        
                        VStack(spacing: 8) {
                            ForEach(match.risks, id: \.self) { risk in
                                Text("• \(risk)")
                                    .font(.subheadline)
                                    .foregroundColor(.secondary)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }
                        .padding(12)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(6)
                        .accessibilityIdentifier(AccessibilityIdentifiers.matchRisksList)
                    }
                }
            } else {
                Text("Match analysis not yet ready. Once processing finishes, verified match signals will appear here.")
                    .font(.body)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(40)
            }
        }
    }
}

struct CompanyDossierTabView: View {
    @ObservedObject var viewModel: WorkspaceDetailViewModel
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            if let company = viewModel.company {
                VStack(alignment: .leading, spacing: 4) {
                    Text(company.companyName)
                        .font(.title2)
                        .fontWeight(.bold)
                    Text("Verified company technical intelligence and citation provenance.")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                
                // Overview
                VStack(alignment: .leading, spacing: 8) {
                    Text("Executive Technical Summary")
                        .font(.headline)
                    Text(company.overview)
                        .font(.body)
                        .foregroundColor(.secondary)
                }
                .padding(16)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(NSColor.controlBackgroundColor))
                .cornerRadius(8)
                .accessibilityIdentifier(AccessibilityIdentifiers.companyOverview)
                
                // Tech Stack
                VStack(alignment: .leading, spacing: 10) {
                    Text("Verified Engineering Tech Stack")
                        .font(.headline)
                    
                    FlowLayout(spacing: 8) {
                        ForEach(company.technologyStack, id: \.self) { tech in
                            Text(tech)
                                .font(.system(.caption, design: .monospaced))
                                .padding(.horizontal, 10)
                                .padding(.vertical, 5)
                                .background(Color(NSColor.windowBackgroundColor))
                                .cornerRadius(4)
                        }
                    }
                }
                .padding(16)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(NSColor.controlBackgroundColor))
                .cornerRadius(8)
                .accessibilityIdentifier(AccessibilityIdentifiers.companyTechStack)
                
                // Sources & Citations
                if !company.sources.isEmpty {
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Grounding Sources (\(company.sources.count))")
                            .font(.headline)
                        
                        VStack(spacing: 8) {
                            ForEach(company.sources, id: \.url) { source in
                                HStack {
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(source.title)
                                            .font(.subheadline)
                                            .fontWeight(.medium)
                                        Text(source.url)
                                            .font(.caption2)
                                            .foregroundColor(.accentColor)
                                    }
                                    Spacer()
                                }
                                .padding(10)
                                .background(Color(NSColor.windowBackgroundColor))
                                .cornerRadius(6)
                            }
                        }
                    }
                    .padding(16)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(NSColor.controlBackgroundColor))
                    .cornerRadius(8)
                    .accessibilityIdentifier(AccessibilityIdentifiers.companyCitationsList)
                }
            } else {
                Text("Company dossier not yet ready. Verified technical research will appear here.")
                    .font(.body)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(40)
            }
        }
    }
}

struct LearningPathTabView: View {
    @ObservedObject var viewModel: WorkspaceDetailViewModel
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Personalized Learning Units")
                    .font(.title2)
                    .fontWeight(.bold)
                Text("Targeted technical prep modules covering verified gaps.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
            
            if viewModel.learningUnits.isEmpty {
                Text("No learning units generated yet.")
                    .foregroundColor(.secondary)
                    .padding(40)
                    .frame(maxWidth: .infinity, alignment: .center)
            } else {
                VStack(spacing: 12) {
                    ForEach(viewModel.learningUnits, id: \.id) { unit in
                        HStack(alignment: .top, spacing: 14) {
                            Button(action: {
                                viewModel.toggleLearningUnit(lessonId: unit.id)
                            }) {
                                Image(systemName: (unit.completed ?? false) ? "checkmark.circle.fill" : "circle")
                                    .font(.system(size: 20))
                                    .foregroundColor((unit.completed ?? false) ? .green : .secondary)
                            }
                            .buttonStyle(.plain)
                            .accessibilityIdentifier("\(AccessibilityIdentifiers.learningTogglePrefix)\(unit.id)")
                            
                            VStack(alignment: .leading, spacing: 6) {
                                HStack {
                                    Text(unit.title)
                                        .font(.headline)
                                    Spacer()
                                    Text("\(unit.estimatedMinutes) min")
                                        .font(.caption2)
                                        .foregroundColor(.secondary)
                                }
                                Text(unit.summary)
                                    .font(.subheadline)
                                    .foregroundColor(.secondary)
                                
                                if !unit.contentMarkdown.isEmpty {
                                    Text(unit.contentMarkdown)
                                        .font(.caption)
                                        .foregroundColor(.primary)
                                        .lineLimit(4)
                                        .padding(.top, 4)
                                }
                            }
                        }
                        .padding(16)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(8)
                        .accessibilityIdentifier("\(AccessibilityIdentifiers.learningUnitRowPrefix)\(unit.id)")
                    }
                }
                .accessibilityIdentifier(AccessibilityIdentifiers.learningUnitsList)
            }
        }
    }
}

struct QuestionBookTabView: View {
    @ObservedObject var viewModel: WorkspaceDetailViewModel
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Curated Interview Question Book")
                        .font(.title2)
                        .fontWeight(.bold)
                    Text("High-leverage questions with interviewer intent, rubrics, and traps.")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                Spacer()
                Picker("Difficulty", selection: $viewModel.questionDifficultyFilter) {
                    Text("All Difficulties").tag("all")
                    Text("Easy").tag("easy")
                    Text("Medium").tag("medium")
                    Text("Hard").tag("hard")
                }
                .frame(width: 150)
                .accessibilityIdentifier(AccessibilityIdentifiers.questionsDifficultyPicker)
            }
            
            if viewModel.filteredQuestions.isEmpty {
                Text("No interview questions matching current criteria.")
                    .foregroundColor(.secondary)
                    .padding(40)
                    .frame(maxWidth: .infinity, alignment: .center)
            } else {
                VStack(spacing: 14) {
                    ForEach(viewModel.filteredQuestions, id: \.id) { q in
                        VStack(alignment: .leading, spacing: 10) {
                            HStack {
                                Text(q.category.rawValue.uppercased())
                                    .font(.caption2)
                                    .fontWeight(.bold)
                                    .padding(.horizontal, 6)
                                    .padding(.vertical, 2)
                                    .background(Color.accentColor.opacity(0.15))
                                    .foregroundColor(.accentColor)
                                    .cornerRadius(4)
                                
                                Text(q.difficulty.rawValue.capitalized)
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                                
                                Spacer()
                            }
                            
                            Text(q.question)
                                .font(.headline)
                            
                            VStack(alignment: .leading, spacing: 4) {
                                Text("Interviewer Intent:")
                                    .font(.caption)
                                    .fontWeight(.semibold)
                                Text(q.interviewerIntent)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            
                            if !q.strongAnswerRubric.isEmpty {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text("Strong Answer Rubric:")
                                        .font(.caption)
                                        .fontWeight(.semibold)
                                    ForEach(q.strongAnswerRubric, id: \.self) { rubric in
                                        Text("• \(rubric)")
                                            .font(.caption)
                                            .foregroundColor(.secondary)
                                    }
                                }
                            }
                        }
                        .padding(16)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(8)
                        .accessibilityIdentifier("\(AccessibilityIdentifiers.questionRowPrefix)\(q.id)")
                    }
                }
                .accessibilityIdentifier(AccessibilityIdentifiers.questionsList)
            }
        }
    }
}

struct LlmSettingsTabView: View {
    @ObservedObject var viewModel: WorkspaceDetailViewModel
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            VStack(alignment: .leading, spacing: 4) {
                Text("LLM Provider Configuration")
                    .font(.title2)
                    .fontWeight(.bold)
                Text("Configure custom API keys for underlying LLM reasoning providers.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
            
            if viewModel.llmConfigs.isEmpty {
                Text("Loading provider settings...")
                    .foregroundColor(.secondary)
                    .padding(20)
            } else {
                VStack(spacing: 12) {
                    ForEach(viewModel.llmConfigs, id: \.provider) { config in
                        HStack {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(config.provider.uppercased())
                                    .font(.headline)
                                Text(config.hasKey ? "Key configured (\(config.isShared ? "Shared" : "Personal"))" : "No key provided")
                                    .font(.caption)
                                    .foregroundColor(config.hasKey ? .green : .secondary)
                            }
                            Spacer()
                            Button("Configure") {
                                viewModel.editingProvider = config.provider
                                viewModel.editingApiKey = ""
                            }
                            .buttonStyle(.bordered)
                            .accessibilityIdentifier("\(AccessibilityIdentifiers.llmConfigureKeyPrefix)\(config.provider)")
                        }
                        .padding(16)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(8)
                        .accessibilityIdentifier("\(AccessibilityIdentifiers.llmProviderRowPrefix)\(config.provider)")
                    }
                }
                .accessibilityIdentifier(AccessibilityIdentifiers.llmProvidersList)
            }
        }
        .sheet(isPresented: Binding(
            get: { viewModel.editingProvider != nil },
            set: { if !$0 { viewModel.editingProvider = nil } }
        )) {
            VStack(spacing: 16) {
                Text("Configure \(viewModel.editingProvider?.uppercased() ?? "") Key")
                    .font(.headline)
                
                SecureField("Enter API Key (sk-...)", text: $viewModel.editingApiKey)
                    .textFieldStyle(RoundedBorderTextFieldStyle())
                    .accessibilityIdentifier(AccessibilityIdentifiers.inputLlmApiKey)
                
                if let error = viewModel.llmKeyErrorMessage {
                    Text(error)
                        .font(.caption)
                        .foregroundColor(.red)
                }
                
                HStack(spacing: 12) {
                    Button("Cancel") {
                        viewModel.editingProvider = nil
                    }
                    .accessibilityIdentifier(AccessibilityIdentifiers.buttonCancelLlmKey)
                    
                    Button("Save") {
                        if let prov = viewModel.editingProvider {
                            viewModel.saveLlmKey(provider: prov, apiKey: viewModel.editingApiKey)
                        }
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(viewModel.editingApiKey.isEmpty || viewModel.isSavingLlmKey)
                    .accessibilityIdentifier(AccessibilityIdentifiers.buttonSaveLlmKey)
                }
            }
            .padding(24)
            .frame(width: 380)
            .accessibilityIdentifier(AccessibilityIdentifiers.llmKeySheet)
        }
    }
}

// FlowLayout helper for SwiftUI
struct FlowLayout: Layout {
    var spacing: CGFloat = 8
    
    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = proposal.width ?? 0
        var height: CGFloat = 0
        var currentX: CGFloat = 0
        var currentY: CGFloat = 0
        var lineHeight: CGFloat = 0
        
        for view in subviews {
            let size = view.sizeThatFits(.unspecified)
            if currentX + size.width > width && currentX > 0 {
                currentX = 0
                currentY += lineHeight + spacing
                lineHeight = 0
            }
            lineHeight = max(lineHeight, size.height)
            currentX += size.width + spacing
        }
        height = currentY + lineHeight
        return CGSize(width: width, height: height)
    }
    
    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        var currentX = bounds.minX
        var currentY = bounds.minY
        var lineHeight: CGFloat = 0
        
        for view in subviews {
            let size = view.sizeThatFits(.unspecified)
            if currentX + size.width > bounds.maxX && currentX > bounds.minX {
                currentX = bounds.minX
                currentY += lineHeight + spacing
                lineHeight = 0
            }
            view.place(at: CGPoint(x: currentX, y: currentY), proposal: ProposedViewSize(size))
            lineHeight = max(lineHeight, size.height)
            currentX += size.width + spacing
        }
    }
}
