using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Mensetsu.Contracts.Model;
using Mensetsu.Services;

namespace Mensetsu.ViewModels
{
    public class WorkspaceListViewModel : ViewModelBase
    {
        private readonly IMensetsuApiService _apiService;
        private readonly IAuthSession _session;

        private List<WorkspaceResponseDto> _allWorkspaces = new();
        private string _selectedFilter = "all";
        private bool _isLoading;
        private string? _errorMessage;

        // Create dialog state
        private bool _isCreateDialogOpen;
        private string _createCompany = string.Empty;
        private string _createRole = string.Empty;
        private string _createResumeMode = "text"; // "text" or "pdf"
        private string _createResumeText = string.Empty;
        private string? _selectedPdfPath;
        private string? _selectedPdfFilename;
        private byte[]? _selectedPdfBytes;
        private string _createJobDescription = string.Empty;
        private bool _isCreating;
        private string? _createErrorMessage;
        public ObservableCollection<WorkspaceResponseDto> FilteredWorkspaces { get; } = new();

        public event Action<string>? OpenWorkspaceRequested;
        public event Action? OpenSettingsRequested;
        public event Action? LoggedOut;

        public WorkspaceListViewModel(IMensetsuApiService apiService, IAuthSession session)
        {
            _apiService = apiService;
            _session = session;
        }

        public string SelectedFilter
        {
            get => _selectedFilter;
            set
            {
                if (SetProperty(ref _selectedFilter, value))
                {
                    ApplyFilter();
                }
            }
        }

        public bool IsLoading
        {
            get => _isLoading;
            set => SetProperty(ref _isLoading, value);
        }

        public string? ErrorMessage
        {
            get => _errorMessage;
            set => SetProperty(ref _errorMessage, value);
        }

        public bool IsCreateDialogOpen
        {
            get => _isCreateDialogOpen;
            set => SetProperty(ref _isCreateDialogOpen, value);
        }

        public string CreateCompany
        {
            get => _createCompany;
            set => SetProperty(ref _createCompany, value);
        }

        public string CreateRole
        {
            get => _createRole;
            set => SetProperty(ref _createRole, value);
        }
        public string CreateResumeMode
        {
            get => _createResumeMode;
            set => SetProperty(ref _createResumeMode, value);
        }

        public string CreateResumeText
        {
            get => _createResumeText;
            set => SetProperty(ref _createResumeText, value);
        }

        public string? SelectedPdfPath
        {
            get => _selectedPdfPath;
            set => SetProperty(ref _selectedPdfPath, value);
        }

        public string? SelectedPdfFilename
        {
            get => _selectedPdfFilename;
            set => SetProperty(ref _selectedPdfFilename, value);
        }

        public byte[]? SelectedPdfBytes
        {
            get => _selectedPdfBytes;
            set => SetProperty(ref _selectedPdfBytes, value);
        }

        public string CreateJobDescription
        {
            get => _createJobDescription;
            set => SetProperty(ref _createJobDescription, value);
        }

        public bool IsCreating
        {
            get => _isCreating;
            set => SetProperty(ref _isCreating, value);
        }

        public string? CreateErrorMessage
        {
            get => _createErrorMessage;
            set => SetProperty(ref _createErrorMessage, value);
        }

        public async Task LoadWorkspacesAsync()
        {
            IsLoading = true;
            ErrorMessage = null;

            try
            {
                _allWorkspaces = await _apiService.ListWorkspacesAsync();
                ApplyFilter();
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
                _allWorkspaces.Clear();
                FilteredWorkspaces.Clear();
            }
            finally
            {
                IsLoading = false;
            }
        }

        private void ApplyFilter()
        {
            FilteredWorkspaces.Clear();
            var filtered = _allWorkspaces.Where(w =>
            {
                if (_selectedFilter.Equals("all", StringComparison.OrdinalIgnoreCase)) return true;
                var statusStr = w.Status.ToString().ToLowerInvariant();
                return statusStr.Equals(_selectedFilter.ToLowerInvariant());
            });

            foreach (var ws in filtered)
            {
                FilteredWorkspaces.Add(ws);
            }
        }

        public void OpenCreateDialog()
        {
            CreateCompany = string.Empty;
            CreateRole = string.Empty;
            CreateResumeMode = "text";
            CreateResumeText = string.Empty;
            SelectedPdfPath = null;
            SelectedPdfFilename = null;
            SelectedPdfBytes = null;
            CreateJobDescription = string.Empty;
            CreateErrorMessage = null;
            IsCreateDialogOpen = true;
        }

        public void SetSelectedPdf(string path, string filename, byte[] bytes)
        {
            SelectedPdfPath = path;
            SelectedPdfFilename = filename;
            SelectedPdfBytes = bytes;
            CreateErrorMessage = null;
        }

        public void ClearSelectedPdf()
        {
            SelectedPdfPath = null;
            SelectedPdfFilename = null;
            SelectedPdfBytes = null;
        }

        public void CloseCreateDialog()
        {
            IsCreateDialogOpen = false;
            CreateErrorMessage = null;
        }

        public async Task SubmitCreateWorkspaceAsync()
        {
            // Prevent duplicate submits
            if (IsCreating) return;

            if (string.IsNullOrWhiteSpace(CreateCompany) || string.IsNullOrWhiteSpace(CreateRole))
            {
                CreateErrorMessage = "Company name and role title are required.";
                return;
            }

            string? resumeText = null;
            string? resumeUploadId = null;

            if (CreateResumeMode == "pdf")
            {
                if (SelectedPdfBytes == null || SelectedPdfBytes.Length == 0 || string.IsNullOrEmpty(SelectedPdfFilename))
                {
                    CreateErrorMessage = "Please select a PDF file.";
                    return;
                }

                if (SelectedPdfBytes.Length > 5 * 1024 * 1024)
                {
                    CreateErrorMessage = "PDF file size must not exceed 5 MiB.";
                    return;
                }

                if (!SelectedPdfFilename.EndsWith(".pdf", StringComparison.OrdinalIgnoreCase))
                {
                    CreateErrorMessage = "File must have a .pdf extension.";
                    return;
                }
            }
            else
            {
                resumeText = CreateResumeText?.Trim() ?? string.Empty;
            }

            IsCreating = true;
            CreateErrorMessage = null;

            try
            {
                if (CreateResumeMode == "pdf" && SelectedPdfBytes != null)
                {
                    // 1. Presign request
                    var presign = await _apiService.CreateResumeUploadAsync(
                        filename: SelectedPdfFilename!,
                        sizeBytes: SelectedPdfBytes.Length,
                        contentType: "application/pdf"
                    );

                    // 2. Direct binary PUT to returned signed URL (without host rewrite, unauthed)
                    await _apiService.UploadResumePdfBytesAsync(
                        uploadUrl: presign.UploadUrl,
                        pdfBytes: SelectedPdfBytes
                    );

                    // 3. Complete notification
                    var completed = await _apiService.CompleteResumeUploadAsync(presign.UploadId);
                    resumeUploadId = completed.UploadId;
                }

                // 4. Create workspace
                var created = await _apiService.CreateWorkspaceAsync(
                    companyName: CreateCompany.Trim(),
                    roleTitle: CreateRole.Trim(),
                    resumeText: resumeText,
                    resumeUploadId: resumeUploadId,
                    jobDescription: CreateJobDescription?.Trim() ?? string.Empty
                );

                IsCreateDialogOpen = false;
                await LoadWorkspacesAsync();
                OpenWorkspaceRequested?.Invoke(created.Id);
            }
            catch (Exception ex)
            {
                // Retain selected file and error for retry
                CreateErrorMessage = ex.Message;
            }
            finally
            {
                IsCreating = false;
            }
        }

        public void SelectWorkspace(WorkspaceResponseDto ws)
        {
            OpenWorkspaceRequested?.Invoke(ws.Id);
        }

        public void OpenSettings()
        {
            OpenSettingsRequested?.Invoke();
        }

        public async Task LogoutAsync()
        {
            await _apiService.LogoutAsync();
            LoggedOut?.Invoke();
        }
    }
}
