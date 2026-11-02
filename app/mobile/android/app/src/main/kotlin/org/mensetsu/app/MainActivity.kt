package org.mensetsu.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import org.mensetsu.app.core.theme.DarkColorScheme
import org.mensetsu.app.core.theme.LightColorScheme
import org.mensetsu.app.core.theme.AppTypography
import org.mensetsu.app.features.auth.AuthScreen
import org.mensetsu.app.features.settings.LlmSettingsScreen
import org.mensetsu.app.features.workspaces.WorkspaceDetailScreen
import org.mensetsu.app.features.workspaces.WorkspaceListScreen

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val app = application as MensetsuApplication
        val networkManager = app.networkManager

        setContent {
            val isDark = isSystemInDarkTheme()
            MaterialTheme(
                colorScheme = if (isDark) DarkColorScheme else LightColorScheme,
                typography = AppTypography
            ) {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    AppNavigation(networkManager = networkManager)
                }
            }
        }
    }
}

sealed class Screen {
    data object Auth : Screen()
    data object WorkspaceList : Screen()
    data class WorkspaceDetail(val workspaceId: String) : Screen()
    data object Settings : Screen()
}

@Composable
fun AppNavigation(networkManager: org.mensetsu.app.core.network.NetworkManager) {
    // Determine initial screen based on whether an access or refresh token exists
    var currentScreen by remember {
        mutableStateOf<Screen>(
            if (networkManager.getAccessToken() != null || networkManager.getRefreshToken() != null) {
                Screen.WorkspaceList
            } else {
                Screen.Auth
            }
        )
    }

    when (val screen = currentScreen) {
        is Screen.Auth -> {
            AuthScreen(
                networkManager = networkManager,
                onAuthSuccess = {
                    currentScreen = Screen.WorkspaceList
                }
            )
        }
        is Screen.WorkspaceList -> {
            WorkspaceListScreen(
                networkManager = networkManager,
                onNavigateToDetail = { id ->
                    currentScreen = Screen.WorkspaceDetail(id)
                },
                onNavigateToSettings = {
                    currentScreen = Screen.Settings
                },
                onLogout = {
                    currentScreen = Screen.Auth
                }
            )
        }
        is Screen.WorkspaceDetail -> {
            WorkspaceDetailScreen(
                workspaceId = screen.workspaceId,
                networkManager = networkManager,
                onBack = {
                    currentScreen = Screen.WorkspaceList
                }
            )
        }
        is Screen.Settings -> {
            LlmSettingsScreen(
                networkManager = networkManager,
                onBack = {
                    currentScreen = Screen.WorkspaceList
                }
            )
        }
    }
}
