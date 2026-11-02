using System;
using System.Threading.Tasks;
using Mensetsu.Services;

namespace Mensetsu.ViewModels
{
    public class AuthViewModel : ViewModelBase
    {
        private readonly IMensetsuApiService _apiService;
        private readonly IAuthSession _session;

        private bool _isRegisterMode;
        private string _name = string.Empty;
        private string _email = string.Empty;
        private string _password = string.Empty;

        private bool _is2faChallenge;
        private string _twoFaCode = string.Empty;
        private string _tempToken = string.Empty;

        private bool _isLoading;
        private string? _errorMessage;

        public event Action? Authenticated;

        public AuthViewModel(IMensetsuApiService apiService, IAuthSession session)
        {
            _apiService = apiService;
            _session = session;
        }

        public bool IsRegisterMode
        {
            get => _isRegisterMode;
            set => SetProperty(ref _isRegisterMode, value);
        }

        public string Name
        {
            get => _name;
            set => SetProperty(ref _name, value);
        }

        public string Email
        {
            get => _email;
            set => SetProperty(ref _email, value);
        }

        public string Password
        {
            get => _password;
            set => SetProperty(ref _password, value);
        }

        public bool Is2faChallenge
        {
            get => _is2faChallenge;
            set => SetProperty(ref _is2faChallenge, value);
        }

        public string TwoFaCode
        {
            get => _twoFaCode;
            set => SetProperty(ref _twoFaCode, value);
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

        public async Task LoginAsync()
        {
            if (string.IsNullOrWhiteSpace(Email) || string.IsNullOrWhiteSpace(Password))
            {
                ErrorMessage = "Email and password are required.";
                return;
            }

            IsLoading = true;
            ErrorMessage = null;

            try
            {
                var response = await _apiService.LoginAsync(Email.Trim(), Password);
                if (response.Requires2fa == true && !string.IsNullOrEmpty(response.TempToken))
                {
                    _tempToken = response.TempToken;
                    Is2faChallenge = true;
                }
                else
                {
                    Authenticated?.Invoke();
                }
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
            }
            finally
            {
                IsLoading = false;
            }
        }

        public async Task RegisterAsync()
        {
            if (string.IsNullOrWhiteSpace(Name) || string.IsNullOrWhiteSpace(Email) || string.IsNullOrWhiteSpace(Password))
            {
                ErrorMessage = "All fields are required.";
                return;
            }

            IsLoading = true;
            ErrorMessage = null;

            try
            {
                await _apiService.RegisterAsync(Name.Trim(), Email.Trim(), Password);
                Authenticated?.Invoke();
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
            }
            finally
            {
                IsLoading = false;
            }
        }

        public async Task Verify2faAsync()
        {
            if (string.IsNullOrWhiteSpace(TwoFaCode))
            {
                ErrorMessage = "Verification code is required.";
                return;
            }

            IsLoading = true;
            ErrorMessage = null;

            try
            {
                await _apiService.Login2faAsync(TwoFaCode.Trim(), _tempToken);
                Is2faChallenge = false;
                Authenticated?.Invoke();
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
            }
            finally
            {
                IsLoading = false;
            }
        }
    }
}
