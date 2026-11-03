package org.mensetsu.app.features.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import org.mensetsu.app.core.network.EndpointValidator
import org.mensetsu.app.core.network.NetworkManager
import org.mensetsu.app.core.tags.AppTestTags

/**
 * First-launch onboarding screen that prompts the user for the self-hosted Rust API base URL.
 * Normalizes input (strips trailing slashes) and validates (absolute http(s), HTTPS for remote hosts).
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerSetupScreen(
    networkManager: NetworkManager,
    onEndpointSaved: () -> Unit
) {
    var urlInput by remember { mutableStateOf(networkManager.getBaseUrl()) }
    var errorMessage by remember { mutableStateOf<String?>(null) }

    fun submit() {
        errorMessage = null
        when (val result = EndpointValidator.validate(urlInput)) {
            is EndpointValidator.ValidationResult.Valid -> {
                networkManager.setBaseUrl(result.normalizedUrl)
                onEndpointSaved()
            }
            is EndpointValidator.ValidationResult.Invalid -> {
                errorMessage = result.reason
            }
        }
    }

    Scaffold(
        modifier = Modifier.testTag(AppTestTags.SERVER_SETUP_SCREEN),
        topBar = {
            TopAppBar(
                title = { Text("Connect to Server") }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(24.dp)
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Text(
                text = "Self-Hosted API Setup",
                style = MaterialTheme.typography.headlineLarge,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = "Enter your Rust API server base URL. Remote servers must use HTTPS. Plain HTTP is only permitted for localhost development.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            Spacer(modifier = Modifier.height(24.dp))

            if (errorMessage != null) {
                Surface(
                    color = MaterialTheme.colorScheme.errorContainer,
                    shape = MaterialTheme.shapes.small,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(
                        text = errorMessage ?: "",
                        color = MaterialTheme.colorScheme.onErrorContainer,
                        style = MaterialTheme.typography.bodyMedium,
                        modifier = Modifier
                            .padding(12.dp)
                            .testTag(AppTestTags.SERVER_URL_ERROR_MESSAGE)
                    )
                }
                Spacer(modifier = Modifier.height(16.dp))
            }

            OutlinedTextField(
                value = urlInput,
                onValueChange = {
                    urlInput = it
                    if (errorMessage != null) errorMessage = null
                },
                label = { Text("Server Base URL") },
                placeholder = { Text("https://api.example.com") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(
                    keyboardType = KeyboardType.Uri,
                    imeAction = ImeAction.Done
                ),
                keyboardActions = KeyboardActions(
                    onDone = { submit() }
                ),
                modifier = Modifier
                    .fillMaxWidth()
                    .testTag(AppTestTags.INPUT_SERVER_URL)
            )

            Spacer(modifier = Modifier.height(24.dp))

            Button(
                onClick = { submit() },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(50.dp)
                    .testTag(AppTestTags.BUTTON_SAVE_SERVER_URL)
            ) {
                Text("Connect")
            }
        }
    }
}
