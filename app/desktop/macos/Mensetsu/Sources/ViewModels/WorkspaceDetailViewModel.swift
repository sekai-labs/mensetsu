//
// WorkspaceDetailViewModel.swift
// Mensetsu macOS
//

import Foundation
import Combine
import MensetsuContracts

public enum WorkspaceDetailTab: String, CaseIterable, Identifiable {
    case overview = "overview"
    case match = "match"
    case company = "company"
    case learning = "learning"
    case questions = "questions"
    case settings = "settings"
    
    public var id: String { rawValue }
    
    public var title: String {
        switch self {
        case .overview: return "Overview"
        case .match: return "Match Analysis"
        case .company: return "Company Dossier"
        case .learning: return "Learning Path"
        case .questions: return "Question Book"
        case .settings: return "LLM Settings"
        }
    }
    
    public var icon: String {
        switch self {
        case .overview: return "doc.text.magnifyingglass"
        case .match: return "chart.bar.xaxis"
        case .company: return "building.2"
        case .learning: return "graduationcap"
        case .questions: return "questionmark.bubble"
        case .settings: return "gearshape"
        }
    }
}

@MainActor
public final class WorkspaceDetailViewModel: ObservableObject {
    public let workspaceId: String
    
    @Published public var currentTab: WorkspaceDetailTab = .overview
    @Published public var isLoading: Bool = false
    @Published public var errorMessage: String?
    
    // Loaded Entities
    @Published public var workspace: WorkspaceResponseDto?
    @Published public var match: MatchAnalysis?
    @Published public var company: CompanyResearch?
    @Published public var learningUnits: [LearningUnit] = []
    @Published public var questions: [InterviewQuestion] = []
    
    // Questions filter
    @Published public var questionDifficultyFilter: String = "all"
    
    // LLM Settings
    @Published public var llmConfigs: [LlmConfigItemDto] = []
    @Published public var editingProvider: String?
    @Published public var editingApiKey: String = ""
    @Published public var isSavingLlmKey: Bool = false
    @Published public var llmKeyErrorMessage: String?
    
    public init(workspaceId: String) {
        self.workspaceId = workspaceId
    }
    
    public var filteredQuestions: [InterviewQuestion] {
        if questionDifficultyFilter == "all" {
            return questions
        }
        return questions.filter {
            $0.difficulty.rawValue.lowercased() == questionDifficultyFilter.lowercased()
        }
    }
    
    public func loadAllData() {
        isLoading = true
        errorMessage = nil
        Task {
            do {
                async let wsReq = APIClient.shared.getWorkspace(id: self.workspaceId)
                async let matchReq = APIClient.shared.getWorkspaceMatch(id: self.workspaceId)
                async let compReq = APIClient.shared.getWorkspaceCompany(id: self.workspaceId)
                async let learnReq = APIClient.shared.getWorkspaceLearning(id: self.workspaceId)
                async let qReq = APIClient.shared.getWorkspaceQuestions(id: self.workspaceId)
                
                let ws = try await wsReq
                self.workspace = ws
                
                // Some sub-resources might 404 or return empty before completion; catch gracefully
                self.match = try? await matchReq
                self.company = try? await compReq
                self.learningUnits = (try? await learnReq) ?? []
                self.questions = (try? await qReq) ?? []
                
                self.isLoading = false
            } catch {
                self.errorMessage = error.localizedDescription
                self.isLoading = false
            }
        }
    }
    
    public func toggleLearningUnit(lessonId: String) {
        Task {
            do {
                try await APIClient.shared.toggleLearningUnitCompleted(
                    workspaceId: self.workspaceId,
                    lessonId: lessonId
                )
                // Mutate local state
                self.learningUnits = self.learningUnits.map { unit in
                    if unit.id == lessonId {
                        var updated = unit
                        updated.completed = !(unit.completed ?? false)
                        return updated
                    }
                    return unit
                }
            } catch {
                self.errorMessage = "Failed to toggle learning unit: \(error.localizedDescription)"
            }
        }
    }
    
    public func loadLlmSettings() {
        Task {
            do {
                self.llmConfigs = try await APIClient.shared.getLlmSettings()
            } catch {
                // If failed, retain existing
            }
        }
    }
    
    public func saveLlmKey(provider: String, apiKey: String) {
        guard !apiKey.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        isSavingLlmKey = true
        llmKeyErrorMessage = nil
        Task {
            do {
                let req = UpsertLlmConfigRequestDto(apiKey: apiKey, isShared: false, provider: provider)
                _ = try await APIClient.shared.upsertLlmSetting(dto: req)
                self.editingProvider = nil
                self.editingApiKey = ""
                self.isSavingLlmKey = false
                self.loadLlmSettings()
            } catch {
                self.llmKeyErrorMessage = error.localizedDescription
                self.isSavingLlmKey = false
            }
        }
    }
}
