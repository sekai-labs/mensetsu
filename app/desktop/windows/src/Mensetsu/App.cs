using System;
using System.Net.Http;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Mensetsu.Services;
using Mensetsu.ViewModels;
using Mensetsu.Views;

namespace Mensetsu
{
    public partial class App : Application
    {
        public static IServiceProvider Services { get; private set; } = null!;
        public static Window? MainWindow { get; private set; }

        public App()
        {
            var services = new ServiceCollection();
            ConfigureServices(services);
            Services = services.BuildServiceProvider();
        }

        private static void ConfigureServices(IServiceCollection services)
        {
            // Token storage and session
            services.AddSingleton<ISecureTokenStore, CredentialLockerTokenStore>();
            services.AddSingleton<IAuthSession, AuthSession>();

            // Server endpoint storage & dynamic context
            services.AddSingleton<IServerEndpointStore, FileServerEndpointStore>();
            services.AddSingleton<IEndpointContext, EndpointContext>();

            // HTTP Handler with Bearer auth + serialized refresh + 1x retry
            services.AddTransient<AuthenticatedHttpClientHandler>(sp =>
            {
                var session = sp.GetRequiredService<IAuthSession>();
                var endpointContext = sp.GetRequiredService<IEndpointContext>();
                return new AuthenticatedHttpClientHandler(session, endpointContext);
            });

            // Mensetsu API Client using AuthenticatedHttpClientHandler
            services.AddHttpClient<IMensetsuApiService, MensetsuApiService>(client =>
            {
                client.DefaultRequestHeaders.Accept.Clear();
                client.DefaultRequestHeaders.Accept.Add(new System.Net.Http.Headers.MediaTypeWithQualityHeaderValue("application/json"));
            }).AddHttpMessageHandler<AuthenticatedHttpClientHandler>();

            // ViewModels
            services.AddSingleton<MainViewModel>();
        }

        protected override void OnLaunched(LaunchActivatedEventArgs args)
        {
            var mainViewModel = Services.GetRequiredService<MainViewModel>();
            MainWindow = new MainWindow(mainViewModel);
            MainWindow.Activate();

            _ = mainViewModel.InitializeAsync();
        }
    }
}
