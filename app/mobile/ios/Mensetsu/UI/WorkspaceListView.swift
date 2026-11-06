
import SwiftUI
import MensetsuContracts

public struct WorkspaceListView: View {
    @ObservedObject var authViewModel: AuthViewModel
    @StateObject private var viewModel = WorkspaceListViewModel()
    @State private var showingCreateSheet = false
    @State private var showingSettingsSheet = false
    @State private var selectedFilter: FilterOption = .all

    public enum FilterOption: String, CaseIterable, Identifiable {
        case all = "All"
        case ready = "Ready"
        case processing = "Processing"

        public var id: String { rawValue }
    }

    public init(authViewModel: AuthViewModel) {
        self.authViewModel = authViewModel
    }

    public var body: some View {
        NavigationView {
            ZStack {
                Color(UIColor.systemGroupedBackground).ignoresSafeArea()

                VStack(spacing: 0) {
                    filterSegment
                        .padding(.horizontal, 16)
                        .padding(.vertical, 8)
                        .background(Color(UIColor.systemBackground))

                    if viewModel.isLoading && viewModel.workspaces.isEmpty {
                        Spacer()
                        ProgressView("Loading workspaces...")
                            .accessibilityIdentifier(AccessibilityID.workspaceLoadingIndicator)
                        Spacer()
                    } else if let error = viewModel.errorMessage, viewModel.workspaces.isEmpty {
                        Spacer()
                        VStack(spacing: 12) {
                            Image(systemName: "exclamationmark.triangle")
                                .font(.system(size: 40))
                                .foregroundColor(AppColors.rose)
                            Text("Failed to load workspaces")
                                .font(.headline)
                            Text(error)
                                .font(.subheadline)
                                .foregroundColor(AppColors.slate500)
                                .multilineTextAlignment(.center)
                                .padding(.horizontal, 32)
                            Button("Retry") {
                                Task { await viewModel.loadWorkspaces() }
                            }
                            .buttonStyle(.borderedProminent)
                        }
                        .accessibilityIdentifier(AccessibilityID.workspaceErrorBanner)
                        Spacer()
                    } else if filteredWorkspaces.isEmpty {
                        Spacer()
                        VStack(spacing: 12) {
                            Image(systemName: "folder.badge.plus")
                                .font(.system(size: 48))
                                .foregroundColor(AppColors.slate400)
                            Text("No workspaces found")
                                .font(.headline)
                                .foregroundColor(AppColors.slate700)
                            Text("Create your first interview preparation workspace.")
                                .font(.subheadline)
                                .foregroundColor(AppColors.slate500)
                            Button("Create Workspace") {
                                showingCreateSheet = true
                            }
                            .buttonStyle(.borderedProminent)
                        }
                        Spacer()
                    } else {
                        List {
                            ForEach(filteredWorkspaces, id: \.id) { ws in
                                NavigationLink(destination: WorkspaceDetailView(workspaceId: ws.id)) {
                                    workspaceRow(ws)
                                }
                                .accessibilityIdentifier(AccessibilityID.workspaceRow(id: ws.id))
                            }
                        }
                        .listStyle(InsetGroupedListStyle())
                        .refreshable {
                            await viewModel.loadWorkspaces()
                        }
                        .accessibilityIdentifier(AccessibilityID.workspaceList)
                    }
                }
            }
            .navigationTitle("Workspaces")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button(action: { showingSettingsSheet = true }) {
                        Image(systemName: "gearshape")
                    }
                    .accessibilityIdentifier(AccessibilityID.openSettingsButton)
                }

                ToolbarItemGroup(placement: .navigationBarTrailing) {
                    Button(action: { showingCreateSheet = true }) {
                        Image(systemName: "plus")
                    }
                    .accessibilityIdentifier(AccessibilityID.openCreateWorkspaceButton)

                    Button(action: {
                        Task { await authViewModel.logout() }
                    }) {
                        Image(systemName: "rectangle.portrait.and.arrow.right")
                            .foregroundColor(AppColors.rose)
                    }
                    .accessibilityIdentifier(AccessibilityID.logoutButton)
                }
            }
            .sheet(isPresented: $showingCreateSheet) {
                CreateWorkspaceSheet(onCreated: { newWs in
                    showingCreateSheet = false
                    Task { await viewModel.loadWorkspaces() }
                })
            }
            .sheet(isPresented: $showingSettingsSheet) {
                SettingsView(authViewModel: authViewModel)
            }
            .task {
                await viewModel.loadWorkspaces()
            }
        }
        .accessibilityIdentifier(AccessibilityID.workspaceListScreen)
    }

    private var filterSegment: some View {
        Picker("Filter", selection: $selectedFilter) {
            ForEach(FilterOption.allCases) { filter in
                Text(filter.rawValue).tag(filter)
            }
        }
        .pickerStyle(SegmentedPickerStyle())
        .accessibilityIdentifier(AccessibilityID.workspaceFilterSegment)
    }

    private var filteredWorkspaces: [WorkspaceResponseDto] {
        viewModel.workspaces.filter { ws in
            switch selectedFilter {
            case .all:
                return true
            case .ready:
                return ws.status.rawValue.lowercased() == "ready" || ws.status.rawValue.lowercased() == "completed"
            case .processing:
                return ws.status.rawValue.lowercased() != "ready" && ws.status.rawValue.lowercased() != "completed"
            }
        }
    }

    private func workspaceRow(_ ws: WorkspaceResponseDto) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(ws.companyName)
                    .font(.system(size: 17, weight: .semibold))
                    .foregroundColor(AppColors.slate900)
                Spacer()
                statusBadge(ws.status.rawValue)
            }

            Text(ws.roleTitle)
                .font(.system(size: 14))
                .foregroundColor(AppColors.slate600)

            HStack {
                Text("Phase: \(ws.currentPhase.rawValue)")
                    .font(.system(size: 12))
                    .foregroundColor(AppColors.slate500)
                Spacer()
                Text("\(ws.progressPercent)% ready")
                    .font(.system(size: 12, weight: .medium, design: .monospaced))
                    .foregroundColor(AppColors.primary)
            }
        }
        .padding(.vertical, 4)
    }

    private func statusBadge(_ status: String) -> some View {
        let isReady = status.lowercased() == "ready" || status.lowercased() == "completed"
        return Text(status.uppercased())
            .font(.system(size: 11, weight: .bold))
            .foregroundColor(isReady ? AppColors.emeraldDark : AppColors.amberDark)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(isReady ? AppColors.emeraldLight : AppColors.amberLight)
            .cornerRadius(4)
    }
}

@MainActor
public final class WorkspaceListViewModel: ObservableObject {
    @Published public var workspaces: [WorkspaceResponseDto] = []
    @Published public var isLoading: Bool = false
    @Published public var errorMessage: String? = nil

    private let service: WorkspaceServiceProtocol

    public init(service: WorkspaceServiceProtocol = WorkspaceService()) {
        self.service = service
    }

    public func loadWorkspaces() async {
        isLoading = true
        errorMessage = nil
        do {
            workspaces = try await service.listWorkspaces()
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
