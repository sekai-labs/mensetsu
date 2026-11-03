//
// MainSplitView.swift
// Mensetsu macOS
//

import SwiftUI
import UniformTypeIdentifiers
import MensetsuContracts
public struct MainSplitView: View {
    @ObservedObject var authViewModel: AuthViewModel
    @StateObject var workspacesViewModel = WorkspacesViewModel()
    
    public init(authViewModel: AuthViewModel) {
        self.authViewModel = authViewModel
    }
    
    public var body: some View {
        NavigationSplitView {
            // Sidebar
            VStack(spacing: 0) {
                // Header / User info & Logout
                HStack {
                    Image(systemName: "person.crop.circle.fill")
                        .resizable()
                        .frame(width: 24, height: 24)
                        .foregroundColor(.accentColor)
                    
                    VStack(alignment: .leading, spacing: 2) {
                        Text(authViewModel.currentUser?.name ?? "User")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .lineLimit(1)
                        Text(authViewModel.currentUser?.email ?? "")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                    }
                    Spacer()
                    
                    Button(action: {
                        authViewModel.logout()
                    }) {
                        Image(systemName: "rectangle.portrait.and.arrow.right")
                            .font(.system(size: 14))
                    }
                    .buttonStyle(.plain)
                    .help("Log out")
                    .accessibilityIdentifier(AccessibilityIdentifiers.buttonLogout)
                }
                .padding(12)
                .background(Color(NSColor.controlBackgroundColor))
                
                Divider()
                
                // Filters Segment
                Picker("Filter", selection: $workspacesViewModel.selectedFilter) {
                    Text("All").tag(WorkspaceFilter.all)
                        .accessibilityIdentifier(AccessibilityIdentifiers.filterAll)
                    Text("Ready").tag(WorkspaceFilter.ready)
                        .accessibilityIdentifier(AccessibilityIdentifiers.filterReady)
                    Text("Processing").tag(WorkspaceFilter.processing)
                        .accessibilityIdentifier(AccessibilityIdentifiers.filterProcessing)
                }
                .pickerStyle(SegmentedPickerStyle())
                .padding(10)
                .accessibilityIdentifier(AccessibilityIdentifiers.filterSegment)
                
                // List of Workspaces
                if workspacesViewModel.isLoading {
                    Spacer()
                    ProgressView()
                    Spacer()
                } else if let error = workspacesViewModel.errorMessage {
                    Spacer()
                    VStack(spacing: 8) {
                        Text("Failed to load workspaces")
                            .font(.caption)
                            .foregroundColor(.red)
                        Button("Retry") {
                            workspacesViewModel.fetchWorkspaces()
                        }
                        .controlSize(.small)
                    }
                    Spacer()
                } else if workspacesViewModel.filteredWorkspaces.isEmpty {
                    Spacer()
                    Text("No workspaces yet.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                } else {
                    List(selection: $workspacesViewModel.selectedWorkspaceId) {
                        ForEach(workspacesViewModel.filteredWorkspaces, id: \.id) { ws in
                            NavigationLink(value: ws.id) {
                                HStack(alignment: .center, spacing: 10) {
                                    VStack(alignment: .leading, spacing: 3) {
                                        Text(ws.companyName)
                                            .font(.body)
                                            .fontWeight(.semibold)
                                            .lineLimit(1)
                                        Text(ws.roleTitle)
                                            .font(.caption)
                                            .foregroundColor(.secondary)
                                            .lineLimit(1)
                                    }
                                    Spacer()
                                    Text(ws.status.rawValue.uppercased())
                                        .font(.system(size: 9, weight: .bold))
                                        .padding(.horizontal, 6)
                                        .padding(.vertical, 2)
                                        .background(statusColor(ws.status).opacity(0.15))
                                        .foregroundColor(statusColor(ws.status))
                                        .cornerRadius(4)
                                        .accessibilityIdentifier("\(AccessibilityIdentifiers.statusBadgePrefix)\(ws.id)")
                                }
                                .padding(.vertical, 4)
                            }
                            .accessibilityIdentifier("\(AccessibilityIdentifiers.workspaceRowPrefix)\(ws.id)")
                        }
                    }
                    .listStyle(SidebarListStyle())
                    .accessibilityIdentifier(AccessibilityIdentifiers.workspaceList)
                }
                
                Divider()
                
                // Bottom Toolbar: New Workspace & Settings
                HStack {
                    Button(action: {
                        workspacesViewModel.isShowingCreateSheet = true
                    }) {
                        Label("New Workspace", systemImage: "plus.circle.fill")
                            .fontWeight(.medium)
                    }
                    .buttonStyle(.plain)
                    .accessibilityIdentifier(AccessibilityIdentifiers.buttonNewWorkspace)
                    
                    Spacer()

                    Button(action: {
                        authViewModel.isShowingSettings = true
                    }) {
                        Image(systemName: "gearshape")
                            .font(.system(size: 14))
                    }
                    .buttonStyle(.plain)
                    .help("Settings")
                    .accessibilityIdentifier(AccessibilityIdentifiers.buttonSettings)
                }
                .padding(12)
                .background(Color(NSColor.controlBackgroundColor))
            }
            .frame(minWidth: 260)
            .accessibilityIdentifier(AccessibilityIdentifiers.sidebar)
        } detail: {
            if let selectedId = workspacesViewModel.selectedWorkspaceId {
                WorkspaceDetailView(workspaceId: selectedId)
                    .id(selectedId)
            } else {
                VStack(spacing: 12) {
                    Image(systemName: "briefcase")
                        .font(.system(size: 48))
                        .foregroundColor(.secondary)
                    Text("Select or create a workspace to view interview preparation details.")
                        .font(.body)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .frame(minWidth: 800, minHeight: 600)
        .accessibilityIdentifier(AccessibilityIdentifiers.rootSplitView)
        .onAppear {
            workspacesViewModel.fetchWorkspaces()
        }
        .sheet(isPresented: $workspacesViewModel.isShowingCreateSheet) {
            NewWorkspaceSheet(viewModel: workspacesViewModel)
        }
        .sheet(isPresented: $authViewModel.isShowingSettings) {
            SettingsView(authViewModel: authViewModel)
        }
    }
    
    private func statusColor(_ status: WorkspaceStatus) -> Color {
        switch status {
        case .ready: return .green
        case .processing, .pending: return .blue
        case .failed: return .red
        }
    }
}

// MARK: - New Workspace Sheet (Accepting company, role, resume_text, and job_description)

struct NewWorkspaceSheet: View {
    @ObservedObject var viewModel: WorkspacesViewModel
    
    var body: some View {
        VStack(spacing: 20) {
            VStack(alignment: .leading, spacing: 4) {
                Text("New Preparation Workspace")
                    .font(.title2)
                    .fontWeight(.bold)
                Text("Create an evidence-backed interview research workspace.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
            VStack(alignment: .leading, spacing: 14) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Target Company *")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    TextField("e.g. Acme Corp, ScaleVector", text: $viewModel.newCompanyName)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .accessibilityIdentifier(AccessibilityIdentifiers.inputTargetCompany)
                }
                
                VStack(alignment: .leading, spacing: 4) {
                    Text("Role Title *")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    TextField("e.g. Senior Backend Engineer", text: $viewModel.newRoleTitle)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .accessibilityIdentifier(AccessibilityIdentifiers.inputRoleTitle)
                }
                
                VStack(alignment: .leading, spacing: 4) {
                    Text("Job Description (JD)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    TextEditor(text: $viewModel.newJobDescription)
                        .font(.system(.body, design: .monospaced))
                        .frame(height: 80)
                        .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.secondary.opacity(0.3)))
                        .accessibilityIdentifier(AccessibilityIdentifiers.inputJobDescription)
                }
                
                // Résumé Input: Text vs Upload PDF
                VStack(alignment: .leading, spacing: 6) {
                    HStack {
                        Text("Résumé Source")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Spacer()
                        Picker("", selection: $viewModel.resumeInputMode) {
                            Text("Text").tag(WorkspacesViewModel.ResumeInputMode.text)
                                .accessibilityIdentifier(AccessibilityIdentifiers.tabResumeText)
                            Text("PDF").tag(WorkspacesViewModel.ResumeInputMode.pdf)
                                .accessibilityIdentifier(AccessibilityIdentifiers.tabResumePdf)
                        }
                        .pickerStyle(SegmentedPickerStyle())
                        .frame(width: 140)
                        .accessibilityIdentifier(AccessibilityIdentifiers.pickerResumeMode)
                        .disabled(viewModel.createLoading)
                    }
                    
                    if viewModel.resumeInputMode == .text {
                        TextEditor(text: $viewModel.newResumeText)
                            .font(.system(.body, design: .monospaced))
                            .frame(height: 80)
                            .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.secondary.opacity(0.3)))
                            .accessibilityIdentifier(AccessibilityIdentifiers.inputResumeText)
                    } else {
                        VStack(alignment: .leading, spacing: 8) {
                            HStack(spacing: 10) {
                                Button(action: {
                                    openPdfFilePicker()
                                }) {
                                    HStack(spacing: 6) {
                                        Image(systemName: "doc.badge.plus")
                                        Text(viewModel.selectedPdfFilename.isEmpty ? "Select PDF Résumé..." : "Change PDF...")
                                    }
                                }
                                .buttonStyle(.bordered)
                                .accessibilityIdentifier(AccessibilityIdentifiers.buttonSelectPdf)
                                .disabled(viewModel.createLoading)
                                
                                if !viewModel.selectedPdfFilename.isEmpty {
                                    Button(action: {
                                        viewModel.clearSelectedPdf()
                                    }) {
                                        Image(systemName: "xmark.circle.fill")
                                            .foregroundColor(.secondary)
                                    }
                                    .buttonStyle(.plain)
                                    .accessibilityIdentifier(AccessibilityIdentifiers.buttonClearPdf)
                                    .disabled(viewModel.createLoading)
                                }
                                
                                Spacer()
                            }
                            
                            if !viewModel.selectedPdfFilename.isEmpty {
                                HStack(spacing: 8) {
                                    Image(systemName: "doc.text.fill")
                                        .foregroundColor(.accentColor)
                                    Text("\(viewModel.selectedPdfFilename) (\(formattedFileSize(viewModel.selectedPdfSizeBytes)))")
                                        .font(.callout)
                                        .lineLimit(1)
                                        .truncationMode(.middle)
                                }
                                .padding(8)
                                .background(Color.accentColor.opacity(0.08))
                                .cornerRadius(6)
                                .accessibilityIdentifier(AccessibilityIdentifiers.labelSelectedPdf)
                            } else {
                                Text("Select a PDF résumé document (max 5 MiB).")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                        .padding(10)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(RoundedRectangle(cornerRadius: 6).stroke(Color.secondary.opacity(0.3)))
                    }
                }
            }
            
            if let error = viewModel.createErrorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
                    .accessibilityIdentifier(AccessibilityIdentifiers.createErrorMessage)
            }
            
            HStack(spacing: 12) {
                Button("Cancel") {
                    viewModel.isShowingCreateSheet = false
                    viewModel.resetCreateForm()
                }
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier(AccessibilityIdentifiers.buttonCancelCreateWorkspace)
                .disabled(viewModel.createLoading)
                
                Spacer()
                
                Button(action: {
                    viewModel.createWorkspace()
                }) {
                    HStack {
                        if viewModel.createLoading {
                            ProgressView()
                                .controlSize(.small)
                        }
                        Text("Create Workspace")
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(viewModel.createLoading || viewModel.newCompanyName.isEmpty || viewModel.newRoleTitle.isEmpty)
                .keyboardShortcut(.defaultAction)
                .accessibilityIdentifier(AccessibilityIdentifiers.buttonCreateWorkspaceSubmit)
            }
        }
        .padding(24)
        .frame(width: 480)
        .accessibilityIdentifier(AccessibilityIdentifiers.newWorkspaceSheet)
    }
    
    private func openPdfFilePicker() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        if #available(macOS 12.0, *) {
            if let pdfType = UTType(filenameExtension: "pdf") {
                panel.allowedContentTypes = [pdfType]
            }
        } else {
            panel.allowedFileTypes = ["pdf"]
        }
        panel.prompt = "Choose PDF"
        panel.message = "Select a PDF résumé document (up to 5 MiB)"
        
        if panel.runModal() == .OK, let url = panel.url {
            viewModel.selectPdfFile(url: url)
        }
    }
    
    private func formattedFileSize(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useBytes, .useKB, .useMB]
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }
}
