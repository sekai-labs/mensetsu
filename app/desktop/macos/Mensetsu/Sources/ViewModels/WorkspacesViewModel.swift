//
// WorkspacesViewModel.swift
// Mensetsu macOS
//

import Foundation
import Combine
import MensetsuContracts

public enum WorkspaceFilter: String, CaseIterable {
    case all = "all"
    case ready = "ready"
    case processing = "processing"
    
    public var title: String {
        switch self {
        case .all: return "All"
        case .ready: return "Ready"
        case .processing: return "Processing"
        }
    }
}

@MainActor
public final class WorkspacesViewModel: ObservableObject {
    @Published public var workspaces: [WorkspaceResponseDto] = []
    @Published public var selectedFilter: WorkspaceFilter = .all
    @Published public var selectedWorkspaceId: String?
    @Published public var isLoading: Bool = false
    @Published public var errorMessage: String?
    
    // Creation Sheet state
    public enum ResumeInputMode: String, CaseIterable {
        case text = "text"
        case pdf = "pdf"
        
        public var title: String {
            switch self {
            case .text: return "Paste Text"
            case .pdf: return "Upload PDF"
            }
        }
    }
    
    @Published public var isShowingCreateSheet: Bool = false
    @Published public var newCompanyName: String = ""
    @Published public var newRoleTitle: String = ""
    @Published public var resumeInputMode: ResumeInputMode = .text
    @Published public var newResumeText: String = ""
    @Published public var selectedPdfUrl: URL?
    @Published public var selectedPdfFilename: String = ""
    @Published public var selectedPdfSizeBytes: UInt64 = 0
    @Published public var newJobDescription: String = ""
    @Published public var createLoading: Bool = false
    @Published public var createErrorMessage: String?
    
    private var isHoldingSecurityScope: Bool = false
    public let maxPdfSizeBytes: UInt64 = 5 * 1024 * 1024 // 5 MiB
    
    public init() {}
    
    public var filteredWorkspaces: [WorkspaceResponseDto] {
        switch selectedFilter {
        case .all:
            return workspaces
        case .ready:
            return workspaces.filter { $0.status == .ready }
        case .processing:
            return workspaces.filter { $0.status == .processing || $0.status == .pending }
        }
    }
    
    public func fetchWorkspaces() {
        isLoading = true
        errorMessage = nil
        Task {
            do {
                let list = try await APIClient.shared.listWorkspaces()
                self.workspaces = list
                self.isLoading = false
                if self.selectedWorkspaceId == nil, let first = list.first {
                    self.selectedWorkspaceId = first.id
                }
            } catch {
                self.errorMessage = error.localizedDescription
                self.isLoading = false
            }
        }
    }
    
    public func selectPdfFile(url: URL) {
        createErrorMessage = nil
        let ext = url.pathExtension.lowercased()
        guard ext == "pdf" else {
            createErrorMessage = "Only PDF documents (.pdf) are supported."
            return
        }
        
        
        let hasAccess = url.startAccessingSecurityScopedResource()
        self.isHoldingSecurityScope = hasAccess
        
        // Fast non-blocking metadata inspection (fileSize) to avoid holding 5MB in memory on main actor
        do {
            let resourceValues = try url.resourceValues(forKeys: [.fileSizeKey, .totalFileSizeKey])
            let fileSize = resourceValues.fileSize ?? resourceValues.totalFileSize ?? 0
            let size = fileSize > 0 ? UInt64(fileSize) : 0
            guard size > 0 else {
                if hasAccess {
                    url.stopAccessingSecurityScopedResource()
                    self.isHoldingSecurityScope = false
                }
                createErrorMessage = "Selected PDF file is empty."
                return
            }
            guard size <= maxPdfSizeBytes else {
                if hasAccess {
                    url.stopAccessingSecurityScopedResource()
                    self.isHoldingSecurityScope = false
                }
                createErrorMessage = "PDF file size exceeds maximum limit of 5 MiB (\(size) bytes)."
                return
            }
            
            if isHoldingSecurityScope, let oldUrl = selectedPdfUrl {
                oldUrl.stopAccessingSecurityScopedResource()
            }
            isHoldingSecurityScope = hasAccess
            self.selectedPdfUrl = url
            self.selectedPdfFilename = url.lastPathComponent
            self.selectedPdfSizeBytes = size
        } catch {
            if hasAccess {
                url.stopAccessingSecurityScopedResource()
                self.isHoldingSecurityScope = false
            }
            self.createErrorMessage = "Failed to read selected file: \(error.localizedDescription)"
        }
    }
    
    public func clearSelectedPdf() {
        if isHoldingSecurityScope, let url = selectedPdfUrl {
            url.stopAccessingSecurityScopedResource()
            isHoldingSecurityScope = false
        }
        self.selectedPdfUrl = nil
        self.selectedPdfFilename = ""
        self.selectedPdfSizeBytes = 0
    }
    
    public func createWorkspace() {
        guard !createLoading else { return } // Prevent duplicate submits
        createErrorMessage = nil
        let company = newCompanyName.trimmingCharacters(in: .whitespaces)
        let role = newRoleTitle.trimmingCharacters(in: .whitespaces)
        
        guard !company.isEmpty else {
            createErrorMessage = "Company name is required."
            return
        }
        guard !role.isEmpty else {
            createErrorMessage = "Role title is required."
            return
        }
        
        // Mode-specific validation
        if resumeInputMode == .pdf {
            guard let fileUrl = selectedPdfUrl, !selectedPdfFilename.isEmpty else {
                createErrorMessage = "Please select a PDF résumé file."
                return
            }
            guard selectedPdfSizeBytes > 0 else {
                createErrorMessage = "Selected PDF file is empty."
                return
            }
            guard selectedPdfSizeBytes <= maxPdfSizeBytes else {
                createErrorMessage = "PDF file size exceeds maximum limit of 5 MiB."
                return
            }
        }
        
        createLoading = true
        Task {
            do {
                var resumeUploadId: String? = nil
                
                if self.resumeInputMode == .pdf, let fileUrl = self.selectedPdfUrl {
                    let filename = self.selectedPdfFilename
                    let sizeBytes = Int64(self.selectedPdfSizeBytes)
                    
                    // Staged upload flow
                    // 1. Presign upload URL
                    let uploadInit = try await APIClient.shared.createResumeUpload(
                        filename: filename,
                        sizeBytes: sizeBytes,
                        contentType: "application/pdf"
                    )
                    
                    // 2. Binary PUT direct to S3 signed URL (no auth headers/cookies, streaming from file)
                    guard let putUrl = URL(string: uploadInit.uploadUrl) else {
                        throw APIError.serverError(400, "Invalid signed upload URL")
                    }
                    try await APIClient.shared.uploadPdfToPresignedUrl(url: putUrl, fileUrl: fileUrl)
                    
                    // 3. Complete staged upload & verify status
                    let completeRes = try await APIClient.shared.completeResumeUpload(uploadId: uploadInit.uploadId)
                    guard completeRes.status == "completed" else {
                        throw APIError.serverError(400, "Resume upload completion status is '\(completeRes.status)', expected 'completed'")
                    }
                    resumeUploadId = completeRes.uploadId
                }
                
                // 4. Create workspace
                let trimmedResumeText = self.resumeInputMode == .text ? self.newResumeText.trimmingCharacters(in: .whitespacesAndNewlines) : ""
                let trimmedJd = self.newJobDescription.trimmingCharacters(in: .whitespacesAndNewlines)
                
                let req = CreateWorkspaceRequestDto(
                    companyName: company,
                    jobDescription: trimmedJd.isEmpty ? nil : trimmedJd,
                    resumeText: (self.resumeInputMode == .text && !trimmedResumeText.isEmpty) ? trimmedResumeText : nil,
                    resumeUploadId: resumeUploadId,
                    roleTitle: role
                )
                let created = try await APIClient.shared.createWorkspace(dto: req)
                self.createLoading = false
                self.isShowingCreateSheet = false
                self.resetCreateForm()
                self.workspaces.insert(created, at: 0)
                self.selectedWorkspaceId = created.id
            } catch {
                // On failure, retain selected file for retry and display real error
                self.createErrorMessage = error.localizedDescription
                self.createLoading = false
            }
        }
    }
    
    public func resetCreateForm() {
        newCompanyName = ""
        newRoleTitle = ""
        resumeInputMode = .text
        newResumeText = ""
        clearSelectedPdf()
        newJobDescription = ""
        createErrorMessage = nil
    }
}
