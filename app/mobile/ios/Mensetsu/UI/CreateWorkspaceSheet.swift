
import SwiftUI
import MensetsuContracts
import UniformTypeIdentifiers

public struct CreateWorkspaceSheet: View {
    private enum ResumeSource: String, CaseIterable {
        case text = "Text"
        case pdf = "PDF"
    }

    @Environment(\.presentationMode) var presentationMode
    var onCreated: (WorkspaceResponseDto) -> Void

    @State private var companyName = ""
    @State private var roleTitle = ""
    @State private var resumeText = ""
    @State private var jobDescription = ""
    @State private var resumeSource: ResumeSource = .text
    @State private var selectedPDF: URL?
    @State private var isImportingPDF = false

    @State private var isSubmitting = false
    @State private var errorMessage: String? = nil

    private let service: WorkspaceServiceProtocol

    public init(service: WorkspaceServiceProtocol = WorkspaceService(), onCreated: @escaping (WorkspaceResponseDto) -> Void) {
        self.service = service
        self.onCreated = onCreated
    }

    public var body: some View {
        NavigationView {
            Form {
                if let error = errorMessage {
                    Section {
                        HStack {
                            Image(systemName: "exclamationmark.circle.fill")
                                .foregroundColor(AppColors.roseDark)
                            Text(error)
                                .font(.system(size: 13))
                                .foregroundColor(AppColors.roseDark)
                        }
                        .padding(.vertical, 4)
                        .accessibilityIdentifier(AccessibilityID.createWorkspaceErrorBanner)
                    }
                }

                Section(header: Text("Target Information")) {
                    TextField("Company Name (e.g. Acme Corp)", text: $companyName)
                        .accessibilityIdentifier(AccessibilityID.createCompanyField)

                    TextField("Role Title (e.g. Staff Backend Engineer)", text: $roleTitle)
                        .accessibilityIdentifier(AccessibilityID.createRoleField)
                }

                Section(header: Text("Résumé")) {
                    Picker("Résumé source", selection: $resumeSource) {
                        ForEach(ResumeSource.allCases, id: \.self) { source in
                            Text(source.rawValue).tag(source)
                        }
                    }
                    .pickerStyle(.segmented)
                    .accessibilityIdentifier(AccessibilityID.createResumeSourcePicker)

                    if resumeSource == .text {
                        TextEditor(text: $resumeText)
                            .frame(minHeight: 120)
                            .accessibilityIdentifier(AccessibilityID.createResumeTextField)
                    } else {
                        Button("Choose PDF…") {
                            isImportingPDF = true
                        }
                        .disabled(isSubmitting)
                        .accessibilityIdentifier(AccessibilityID.createResumePDFButton)
                        if let selectedPDF = selectedPDF {
                            Text(selectedPDF.lastPathComponent)
                                .accessibilityIdentifier(AccessibilityID.createResumePDFName)
                        } else {
                            Text("PDF up to 5 MiB")
                                .foregroundColor(.secondary)
                        }
                    }
                }

                Section(header: Text("Job Description (Plain Text)")) {
                    TextEditor(text: $jobDescription)
                        .frame(minHeight: 120)
                        .accessibilityIdentifier(AccessibilityID.createJobDescriptionField)
                }
            }
            .navigationTitle("New Workspace")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        presentationMode.wrappedValue.dismiss()
                    }
                    .disabled(isSubmitting)
                    .accessibilityIdentifier(AccessibilityID.cancelCreateWorkspaceButton)
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button("Create") {
                        Task { await createWorkspace() }
                    }
                    .disabled(isSubmitting || companyName.trimmingCharacters(in: .whitespaces).isEmpty || roleTitle.trimmingCharacters(in: .whitespaces).isEmpty || (resumeSource == .pdf && selectedPDF == nil))
                    .accessibilityIdentifier(AccessibilityID.submitCreateWorkspaceButton)
                }
            }
            .overlay {
                if isSubmitting {
                    Color.black.opacity(0.1).ignoresSafeArea()
                    ProgressView("Creating workspace...")
                }
            }
        }
        .fileImporter(isPresented: $isImportingPDF, allowedContentTypes: [.pdf]) { result in
            switch result {
            case let .success(url):
                selectedPDF = url
                errorMessage = nil
            case let .failure(error):
                errorMessage = error.localizedDescription
            }
        }
        .accessibilityIdentifier(AccessibilityID.createWorkspaceSheet)
    }

    @MainActor
    private func createWorkspace() async {
        isSubmitting = true
        errorMessage = nil
        do {
            let company = companyName.trimmingCharacters(in: .whitespaces)
            let role = roleTitle.trimmingCharacters(in: .whitespaces)
            let description = jobDescription.trimmingCharacters(in: .whitespaces)
            let workspace: WorkspaceResponseDto
            switch resumeSource {
            case .text:
                workspace = try await service.createWorkspace(
                    companyName: company, roleTitle: role,
                    resumeText: resumeText.trimmingCharacters(in: .whitespaces),
                    jobDescription: description
                )
            case .pdf:
                guard let selectedPDF = selectedPDF else {
                    throw APIError.validation("Choose a PDF résumé.")
                }
                workspace = try await service.createWorkspaceWithPDF(
                    companyName: company, roleTitle: role,
                    jobDescription: description, fileURL: selectedPDF
                )
            }
            onCreated(workspace)
            presentationMode.wrappedValue.dismiss()
        } catch {
            errorMessage = error.localizedDescription
        }
        isSubmitting = false
    }
}
