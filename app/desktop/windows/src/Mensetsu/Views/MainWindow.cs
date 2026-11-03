using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Mensetsu.ViewModels;

namespace Mensetsu.Views
{
    public sealed partial class MainWindow : Window
    {
        public MainViewModel ViewModel { get; }

        public MainWindow(MainViewModel viewModel)
        {
            ViewModel = viewModel;
            Title = "Mensetsu - Native Studio";

            var contentControl = new ContentControl
            {
                HorizontalContentAlignment = HorizontalAlignment.Stretch,
                VerticalContentAlignment = VerticalAlignment.Stretch
            };

            ViewModel.PropertyChanged += (s, e) =>
            {
                if (e.PropertyName == nameof(MainViewModel.CurrentView))
                {
                    UpdateView(contentControl);
                }
            };

            Content = contentControl;
            UpdateView(contentControl);
        }

        private void UpdateView(ContentControl host)
        {
            switch (ViewModel.CurrentView)
            {
                case ServerSetupViewModel setupVm:
                    host.Content = new ServerSetupView { DataContext = setupVm };
                    break;
                case AuthViewModel authVm:
                    host.Content = new AuthView { DataContext = authVm };
                    break;
                case WorkspaceListViewModel listVm:
                    host.Content = new WorkspaceListView { DataContext = listVm };
                    break;
                case WorkspaceDetailViewModel detailVm:
                    host.Content = new WorkspaceDetailView { DataContext = detailVm };
                    break;
                case SettingsViewModel settingsVm:
                    host.Content = new SettingsView { DataContext = settingsVm };
                    break;
                default:
                    host.Content = new ProgressRing { IsActive = true, HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center };
                    break;
            }
        }
    }
}
